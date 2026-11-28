//! Errors, shaped for a reader who will fix the program: what failed, where
//! in the movie, which command, and when there is one, what to do about it.

use std::fmt;

/// Which part of a movie's life an error came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Before anything ran: bad options, an unusable declaration.
    Load,
    /// The program's top level, run once when the movie is loaded.
    Setup,
    /// Drawing a frame.
    Frame {
        /// The frame's index.
        index: u64,
        /// Its time in seconds.
        time: f64,
    },
    /// Advancing a stateful movie's simulation by one tick.
    Step {
        /// The tick's index: tick `n` advances the state from `n / rate`.
        tick: u64,
        /// The time the tick starts at, in seconds.
        time: f64,
    },
}

impl Phase {
    /// `load`, `setup`, `frame` or `step`.
    pub fn name(&self) -> &'static str {
        match self {
            Phase::Load => "load",
            Phase::Setup => "setup",
            Phase::Frame { .. } => "frame",
            Phase::Step { .. } => "step",
        }
    }
}

/// An error from loading or rendering a movie.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    /// Where in the movie's life it happened.
    pub phase: Phase,
    /// A short machine-readable kind: a PostScript error name
    /// (`undefined`, `typecheck`, `limitexceeded`, `interrupt`, ...) or one
    /// of psanim's own (`declaration`, `options`, `unclosedlayer`,
    /// `unclosedgroup`, `groupnesting`; the last two only once PSCat
    /// reports open transparency groups, PSCatLabs/pscat#1494).
    pub kind: String,
    /// The offending PostScript command, when the interpreter knows it.
    pub command: Option<String>,
    /// The 1-based line of the program's top level the error is attributed
    /// to. Only meaningful for [`Phase::Setup`]: a frame is drawn by calling
    /// a procedure defined earlier, and the interpreter does not track which
    /// line a procedure's body came from.
    pub line: Option<usize>,
    /// The `run` file `line` is in, when not the program itself.
    pub file: Option<String>,
    /// Which limit was passed, for `limitexceeded`: `steps`, `time`, `memory`.
    pub limit: Option<&'static str>,
    /// One sentence saying what went wrong.
    pub message: String,
    /// What to try, when psanim can tell.
    pub hint: Option<String>,
    /// The top of the operand stack when the error happened, topmost last,
    /// as short printed forms: often enough to see which value was wrong.
    pub stack: Vec<String>,
    /// The program's procedures that were running, innermost first: where
    /// in the program's own code the error happened.
    pub backtrace: Vec<TraceLine>,
    /// What the frame printed (`==`, `print`) before it failed, when it
    /// was a frame that failed: often the clue. The movie keeps it too.
    pub printed: crate::engine::Printed,
}

/// One procedure that was running when an error happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceLine {
    /// The procedure's name; `None` for one with no name of its own (a
    /// loop body, the body given to `during`).
    pub procedure: Option<String>,
    /// The element that was running, marked `>>> like this <<<`, among its
    /// neighbours in the procedure's body.
    pub excerpt: String,
}

impl Error {
    pub(crate) fn new(phase: Phase, kind: &str, message: impl Into<String>) -> Error {
        Error {
            phase,
            kind: kind.to_string(),
            command: None,
            line: None,
            file: None,
            limit: None,
            message: message.into(),
            hint: None,
            stack: Vec::new(),
            backtrace: Vec::new(),
            printed: crate::engine::Printed::default(),
        }
    }

    pub(crate) fn hint(mut self, hint: impl Into<String>) -> Error {
        self.hint = Some(hint.into());
        self
    }

    /// The error as a JSON object, for tools.
    pub fn to_json(&self) -> serde_json::Value {
        let mut o = serde_json::json!({
            "phase": self.phase.name(),
            "kind": self.kind,
            "message": self.message,
        });
        let m = o.as_object_mut().expect("an object");
        match self.phase {
            Phase::Frame { index, time } => {
                m.insert("frame".into(), index.into());
                m.insert("time".into(), time.into());
            }
            Phase::Step { tick, time } => {
                m.insert("tick".into(), tick.into());
                m.insert("time".into(), time.into());
            }
            _ => {}
        }
        if let Some(c) = &self.command {
            m.insert("command".into(), c.clone().into());
        }
        if let Some(l) = self.line {
            m.insert("line".into(), l.into());
        }
        if let Some(f) = &self.file {
            m.insert("file".into(), f.clone().into());
        }
        if let Some(l) = self.limit {
            m.insert("limit".into(), l.into());
        }
        if let Some(h) = &self.hint {
            m.insert("hint".into(), h.clone().into());
        }
        if !self.stack.is_empty() {
            m.insert("stack".into(), self.stack.clone().into());
        }
        if !self.backtrace.is_empty() {
            m.insert(
                "backtrace".into(),
                self.backtrace
                    .iter()
                    .map(|t| serde_json::json!({"procedure": t.procedure, "excerpt": t.excerpt}))
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
        if !self.printed.text.is_empty() {
            m.insert("printed".into(), self.printed.text.clone().into());
            if self.printed.truncated {
                m.insert("printed_truncated".into(), true.into());
            }
        }
        o
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.phase {
            Phase::Load => write!(f, "error: ")?,
            Phase::Setup => write!(f, "error in setup: ")?,
            Phase::Frame { index, time } => write!(f, "error in frame {index} (t={time:.3}s): ")?,
            Phase::Step { tick, time } => write!(f, "error in step {tick} (t={time:.3}s): ")?,
        }
        write!(f, "{}", self.message)?;
        if let Some(line) = self.line {
            match &self.file {
                Some(file) => write!(f, " [line {line} of {file}]")?,
                None => write!(f, " [line {line}]")?,
            }
        }
        for t in &self.backtrace {
            match &t.procedure {
                Some(name) => write!(f, "\n  in {name}: {}", t.excerpt)?,
                None => write!(f, "\n  in {{...}}: {}", t.excerpt)?,
            }
        }
        if !self.stack.is_empty() {
            write!(f, "\n  operand stack (top last): {}", self.stack.join(" "))?;
        }
        if let Some(h) = &self.hint {
            write!(f, "\n  hint: {h}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

/// The closest of `candidates` to `name`, if one is within a small edit
/// distance: for "did you mean".
pub(crate) fn closest<'a>(
    name: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    let limit = (name.len() / 3).clamp(1, 3);
    candidates
        .filter(|c| *c != name)
        .map(|c| (edit_distance(name, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != cb)).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_finds_near_misses_only() {
        let names = ["during", "delayed", "tween", "between", "now"];
        assert_eq!(closest("durring", names.iter().copied()), Some("during"));
        assert_eq!(closest("twen", names.iter().copied()), Some("tween"));
        assert_eq!(closest("zzzzzz", names.iter().copied()), None);
        assert_eq!(closest("now", names.iter().copied()), None);
    }

    #[test]
    fn display_and_json_carry_the_same_facts() {
        let mut e = Error::new(
            Phase::Frame {
                index: 12,
                time: 0.4,
            },
            "typecheck",
            "typecheck in `add`",
        );
        e.command = Some("add".into());
        e.stack = vec!["(a)".into(), "3".into()];
        let e = e.hint("add needs two numbers");
        let text = e.to_string();
        assert!(
            text.contains("frame 12") && text.contains("hint: add needs") && text.contains("(a) 3")
        );
        let j = e.to_json();
        assert_eq!(j["phase"], "frame");
        assert_eq!(j["frame"], 12);
        assert_eq!(j["command"], "add");
        assert_eq!(j["stack"][1], "3");
    }
}
