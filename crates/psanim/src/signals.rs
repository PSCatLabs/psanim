//! External inputs: time-varying signals and discrete events.
//!
//! A signal is a number that varies with time and is not computed by the
//! animation: a mouse coordinate, an audio level, a value read from a data
//! file. An event track is a list of moments: beats, key presses, cues.
//! A program reads them with `signal`, `sinceevent` and `eventcount`.
//!
//! For an offline render the host loads them from data, so the render is
//! reproducible. For a live host (the browser player) the host overwrites
//! the current value of a signal before each frame. The program cannot tell
//! the difference, which is the point.

use std::collections::BTreeMap;

/// How a sampled signal is read between its samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Interpolation {
    /// Straight lines between samples.
    #[default]
    Linear,
    /// Each sample holds until the next.
    Step,
}

/// A sampled signal: `(time, value)` pairs in time order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Track {
    pub samples: Vec<(f64, f64)>,
    pub interpolation: Interpolation,
}

impl Track {
    /// A signal that has one value at all times.
    pub fn constant(value: f64) -> Track {
        Track {
            samples: vec![(0.0, value)],
            interpolation: Interpolation::Step,
        }
    }

    /// The signal's value at time `t`. Before the first sample it is the
    /// first value, after the last the last; an empty track reads 0.
    pub fn at(&self, t: f64) -> f64 {
        let s = &self.samples;
        let Some(first) = s.first() else { return 0.0 };
        if t <= first.0 {
            return first.1;
        }
        // The first sample strictly after t.
        let i = s.partition_point(|(st, _)| *st <= t);
        if i >= s.len() {
            return s[s.len() - 1].1;
        }
        let (a, b) = (s[i - 1], s[i]);
        match self.interpolation {
            Interpolation::Step => a.1,
            Interpolation::Linear => {
                let span = b.0 - a.0;
                if span <= 0.0 {
                    b.1
                } else {
                    a.1 + (b.1 - a.1) * ((t - a.0) / span)
                }
            }
        }
    }
}

/// The signals and event tracks a movie can read, by name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Signals {
    pub tracks: BTreeMap<String, Track>,
    /// Event times in seconds, ascending.
    pub events: BTreeMap<String, Vec<f64>>,
}

impl Signals {
    /// Set `name` to a constant: what a live host does before each frame.
    pub fn set(&mut self, name: &str, value: f64) {
        self.tracks.insert(name.to_string(), Track::constant(value));
    }

    pub fn value(&self, name: &str, t: f64) -> Option<f64> {
        self.tracks.get(name).map(|tr| tr.at(t))
    }

    /// Seconds since the most recent event at or before `t`, and how many
    /// events have happened by then. `None` when there has been none.
    pub fn last_event(&self, name: &str, t: f64) -> (Option<f64>, usize) {
        let Some(times) = self.events.get(name) else {
            return (None, 0);
        };
        let n = times.partition_point(|e| *e <= t);
        (n.checked_sub(1).map(|i| t - times[i]), n)
    }

    /// Read signals from CSV text: a header row naming the columns, the
    /// first of which is time in seconds; each other column is a signal.
    /// Empty cells are skipped, so signals can be sampled at different
    /// times. A column whose header ends in `!` is an event track instead:
    /// every row with a non-empty cell in it is an event at that row's time.
    ///
    /// ```
    /// let s = psanim::Signals::from_csv("t,level,beat!\n0,0.0,\n0.5,1.0,x\n1,0.5,\n").unwrap();
    /// assert_eq!(s.value("level", 0.25), Some(0.5));
    /// assert_eq!(s.last_event("beat", 0.75), (Some(0.25), 1));
    /// ```
    pub fn from_csv(text: &str) -> Result<Signals, String> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#'));
        let header: Vec<&str> = lines
            .next()
            .ok_or("signal file is empty")?
            .split(',')
            .map(str::trim)
            .collect();
        if header.len() < 2 {
            return Err("signal file needs a time column and at least one signal column".into());
        }
        let mut out = Signals::default();
        for (n, line) in lines.enumerate() {
            let cells: Vec<&str> = line.split(',').map(str::trim).collect();
            let t: f64 = cells[0]
                .parse()
                .map_err(|_| format!("row {}: time '{}' is not a number", n + 2, cells[0]))?;
            for (name, cell) in header.iter().zip(&cells).skip(1) {
                if cell.is_empty() {
                    continue;
                }
                if let Some(event) = name.strip_suffix('!') {
                    out.events.entry(event.to_string()).or_default().push(t);
                } else {
                    let v: f64 = cell
                        .parse()
                        .map_err(|_| format!("row {}: '{cell}' in column '{name}' is not a number", n + 2))?;
                    out.tracks.entry(name.to_string()).or_default().samples.push((t, v));
                }
            }
        }
        for track in out.tracks.values_mut() {
            track.samples.sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        for events in out.events.values_mut() {
            events.sort_by(f64::total_cmp);
        }
        Ok(out)
    }

    /// Merge `other` into this set; its entries replace same-named ones.
    pub fn extend(&mut self, other: Signals) {
        self.tracks.extend(other.tracks);
        self.events.extend(other.events);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_interpolates_and_clamps() {
        let tr = Track { samples: vec![(1.0, 10.0), (2.0, 20.0), (4.0, 0.0)], ..Default::default() };
        assert_eq!(tr.at(0.0), 10.0);
        assert_eq!(tr.at(1.0), 10.0);
        assert_eq!(tr.at(1.5), 15.0);
        assert_eq!(tr.at(2.0), 20.0);
        assert_eq!(tr.at(3.0), 10.0);
        assert_eq!(tr.at(9.0), 0.0);
        let step = Track { interpolation: Interpolation::Step, ..tr };
        assert_eq!(step.at(1.99), 10.0);
        assert_eq!(step.at(2.0), 20.0);
        assert_eq!(Track::default().at(1.0), 0.0);
        assert_eq!(Track::constant(3.5).at(-100.0), 3.5);
    }

    #[test]
    fn events_count_and_age() {
        let mut s = Signals::default();
        s.events.insert("beat".into(), vec![0.5, 1.0, 1.5]);
        assert_eq!(s.last_event("beat", 0.2), (None, 0));
        assert_eq!(s.last_event("beat", 0.5), (Some(0.0), 1));
        assert_eq!(s.last_event("beat", 1.25), (Some(0.25), 2));
        assert_eq!(s.last_event("nope", 1.0), (None, 0));
    }

    #[test]
    fn csv_errors_name_the_row() {
        assert!(Signals::from_csv("").is_err());
        assert!(Signals::from_csv("t\n0\n").is_err());
        let e = Signals::from_csv("t,a\n0,1\nx,2\n").unwrap_err();
        assert!(e.contains("row 3"), "{e}");
        let e = Signals::from_csv("t,a\n0,one\n").unwrap_err();
        assert!(e.contains("column 'a'"), "{e}");
    }

    #[test]
    fn csv_sparse_columns_and_unsorted_rows() {
        let s = Signals::from_csv("t, a, b\n2, 4,\n0, 0, 7\n# comment\n1, , 9\n").unwrap();
        assert_eq!(s.value("a", 1.0), Some(2.0));
        assert_eq!(s.value("b", 0.5), Some(8.0));
        assert_eq!(s.value("c", 0.5), None);
    }
}
