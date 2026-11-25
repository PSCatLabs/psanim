//! Playlists: which scenes are on the list at a given time.
//!
//! A playlist cycles through its scenes in rounds. With no schedule a round
//! is the whole list and the loop has a fixed length. With a schedule each
//! scene says on which days and between which times it is on the list, and
//! the set is chosen when a round starts, so a scene is never cut off
//! halfway. That makes the start of round `r` depend on the sets of the
//! rounds before it; this module finds the round that contains a time `t`
//! without playing them all (a stretch of rounds with the same set is
//! skipped arithmetically), so a frame stays a pure function of `t`.
//!
//! The PostScript half (`prelude.ps`, "playlists") lays out the round and
//! draws it through `reel`; this half only answers "which round, which set".

use pscat::Interp;
use pscat::PsError;
use pscat::host::Value;

use crate::ops::{Shared, fail, reg};

const DAY: f64 = 86_400.0;
const WEEK: f64 = 7.0 * DAY;
/// Scenes in one playlist: the set is a bit mask in an integer.
pub const MAX_SCENES: usize = 60;

/// What the solver needs to know of one scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// Seconds the scene takes in a round: its hold and its transition out.
    pub slot: f64,
    /// Days it is on the list, bit 0 Monday to bit 6 Sunday.
    pub days: u32,
    /// Time of day it is on the list from and until, in seconds. From after
    /// until means a window that crosses midnight.
    pub from: f64,
    pub to: f64,
}

impl Slot {
    fn restricted(&self) -> bool {
        self.days & 127 != 127 || self.from != 0.0 || self.to != DAY
    }

    fn active(&self, day: u32, tod: f64) -> bool {
        if self.days >> day & 1 == 0 {
            return false;
        }
        match self.from.partial_cmp(&self.to) {
            Some(std::cmp::Ordering::Less) => tod >= self.from && tod < self.to,
            Some(std::cmp::Ordering::Greater) => tod >= self.from || tod < self.to,
            _ => true,
        }
    }
}

/// The round that contains a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Round {
    /// Playlist time at which the round starts.
    pub start: f64,
    /// The scenes in it (bit i for scene i); 0 means nothing is scheduled.
    pub mask: u64,
    /// The sets of the round before and the round after: the previous
    /// round's last scene hands over to this one's first, and this round's
    /// last hands over to the next one's first. The first round has itself
    /// before it, so a playlist without a schedule loops seamlessly.
    pub prev: u64,
    pub next: u64,
}

fn mask_at(slots: &[Slot], week: f64) -> u64 {
    let week = week.rem_euclid(WEEK);
    let day = ((week / DAY).floor() as u32).min(6);
    let tod = week - f64::from(day) * DAY;
    let mut m = 0;
    for (i, s) in slots.iter().enumerate() {
        if s.active(day, tod) {
            m |= 1 << i;
        }
    }
    m
}

/// The next time of day after `tod` at which any set could change, in
/// seconds from `tod`; infinite when no scene is scheduled.
fn until_change(slots: &[Slot], tod: f64) -> f64 {
    if !slots.iter().any(Slot::restricted) {
        return f64::INFINITY;
    }
    let mut next = DAY; // midnight: the day, and so the days that apply, changes
    for s in slots.iter().filter(|s| s.restricted()) {
        for edge in [s.from, s.to] {
            if edge > tod + 1e-9 && edge < next {
                next = edge;
            }
        }
    }
    next - tod
}

/// Find the round that contains playlist time `t`, where time 0 is `start`
/// seconds after Monday 00:00.
pub fn solve(slots: &[Slot], start: f64, t: f64) -> Result<Round, String> {
    let t = t.max(0.0);
    let mut pos = 0.0;
    let mut last: Option<u64> = None;
    for _ in 0..1_000_000 {
        let week = (start + pos).rem_euclid(WEEK);
        let day = ((week / DAY).floor() as u32).min(6);
        let tod = week - f64::from(day) * DAY;
        let mask = mask_at(slots, week);
        let change = until_change(slots, tod);
        // A round, and how many in a row start before the set can change.
        let (len, count) = if mask == 0 {
            (change, 1.0)
        } else {
            let len: f64 = slots
                .iter()
                .enumerate()
                .filter(|(i, _)| mask >> i & 1 == 1)
                .map(|(_, s)| s.slot)
                .sum();
            (len, ((change / len) - 1e-9).ceil().max(1.0))
        };
        if !len.is_finite() || len <= 0.0 {
            return Err("a playlist with nothing scheduled and no end".into());
        }
        let hit = ((t - pos) / len).floor();
        if hit < count {
            let start_of_round = pos + hit * len;
            return Ok(Round {
                start: start_of_round,
                mask,
                prev: if hit > 0.0 {
                    mask
                } else {
                    last.unwrap_or(mask)
                },
                next: mask_at(slots, start + start_of_round + len),
            });
        }
        pos += count * len;
        last = Some(mask);
    }
    Err("the playlist's schedule did not settle".into())
}

fn slots_from(v: &Value) -> Option<Vec<Slot>> {
    let Value::Array(items) = v else { return None };
    items
        .iter()
        .map(|item| {
            let n = item.as_numbers()?;
            let [slot, days, from, to] = n[..] else {
                return None;
            };
            Some(Slot {
                slot,
                days: days as u32,
                from,
                to,
            })
        })
        .collect()
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // [ [slot days from to] ... ] start t .plsolve  ->  round-start mask prev next
    reg(interp, ctx, ".plsolve", |i, ctx| {
        let t = i.pop_f64()?;
        let start = i.pop_f64()?;
        let Some(slots) = slots_from(&i.pop_value()?) else {
            return fail(
                ctx,
                PsError::Typecheck,
                "playlist: the schedule is an array of [slot days from to]",
            );
        };
        if slots.len() > MAX_SCENES {
            return fail(
                ctx,
                PsError::Rangecheck,
                format!("a playlist holds at most {MAX_SCENES} scenes"),
            );
        }
        match solve(&slots, start, t) {
            Ok(r) => {
                i.push_real(r.start);
                i.push_int(r.mask as i64);
                i.push_int(r.prev as i64);
                i.push_int(r.next as i64);
                Ok(())
            }
            Err(msg) => fail(ctx, PsError::Rangecheck, format!("playlist: {msg}")),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: u32 = 127;

    fn always(slot: f64) -> Slot {
        Slot {
            slot,
            days: ALL,
            from: 0.0,
            to: DAY,
        }
    }

    #[test]
    fn an_unscheduled_list_is_one_fixed_loop() {
        let s = [always(10.0), always(5.0)];
        let r = solve(&s, 0.0, 47.0).unwrap();
        assert_eq!((r.start, r.mask, r.prev, r.next), (45.0, 3, 3, 3));
        // And far into a long run, in one step.
        let r = solve(&s, 0.0, 1.0e9 + 7.0).unwrap();
        assert_eq!(r.start, 1.0e9 + 5.0);
        assert_eq!(r.mask, 3);
    }

    #[test]
    fn a_window_puts_a_scene_on_the_list_only_inside_it() {
        // Scene 1 is on the list from 09:00 to 17:00.
        let s = [
            always(100.0),
            Slot {
                slot: 100.0,
                days: ALL,
                from: 9.0 * 3600.0,
                to: 17.0 * 3600.0,
            },
        ];
        // Starting at 08:00, the first round has only scene 0.
        let r = solve(&s, 8.0 * 3600.0, 50.0).unwrap();
        assert_eq!((r.mask, r.start), (1, 0.0));
        // Rounds of 100 s start at 0, 100, ...; the first to start at or
        // after 09:00 (3600 s in) has both.
        let r = solve(&s, 8.0 * 3600.0, 3650.0).unwrap();
        assert_eq!((r.mask, r.start, r.prev), (3, 3600.0, 1));
        // The round before the change hands over to the one after it.
        let r = solve(&s, 8.0 * 3600.0, 3590.0).unwrap();
        assert_eq!((r.mask, r.next), (1, 3));
        // After 17:00 it goes again.
        let r = solve(&s, 8.0 * 3600.0, 9.0 * 3600.0 + 50.0).unwrap();
        assert_eq!(r.mask, 1);
    }

    #[test]
    fn days_and_nothing_scheduled() {
        // Only on Saturday (bit 5): the rest of the week is an empty round
        // that lasts until the day changes.
        let s = [Slot {
            slot: 60.0,
            days: 1 << 5,
            from: 0.0,
            to: DAY,
        }];
        let r = solve(&s, 0.0, 3.0 * DAY).unwrap();
        assert_eq!(r.mask, 0);
        let r = solve(&s, 0.0, 5.0 * DAY + 30.0).unwrap();
        assert_eq!((r.mask, r.prev, r.start), (1, 0, 5.0 * DAY));
        // Sunday again is empty.
        let r = solve(&s, 0.0, 6.0 * DAY + 100.0).unwrap();
        assert_eq!(r.mask, 0);
    }

    #[test]
    fn a_window_may_cross_midnight() {
        let s = [Slot {
            slot: 10.0,
            days: ALL,
            from: 22.0 * 3600.0,
            to: 6.0 * 3600.0,
        }];
        assert_eq!(solve(&s, 23.0 * 3600.0, 0.0).unwrap().mask, 1);
        assert_eq!(solve(&s, 3.0 * 3600.0, 0.0).unwrap().mask, 1);
        assert_eq!(solve(&s, 12.0 * 3600.0, 0.0).unwrap().mask, 0);
    }

    #[test]
    fn the_solution_does_not_depend_on_how_it_is_reached() {
        // Rounds found one at a time agree with the jump over a long stretch.
        let s = [
            always(7.0),
            Slot {
                slot: 11.0,
                days: ALL,
                from: 1.0 * 3600.0,
                to: 2.0 * 3600.0,
            },
        ];
        let mut t = 0.0;
        let mut starts = vec![];
        while t < 4.0 * 3600.0 {
            let r = solve(&s, 0.0, t).unwrap();
            starts.push(r.start);
            t = r.start + if r.mask == 3 { 18.0 } else { 7.0 };
        }
        // Every round starts where the one before ended.
        for w in starts.windows(2) {
            assert!(w[1] > w[0]);
        }
        let r = solve(&s, 0.0, 3.0 * 3600.0 + 1.0).unwrap();
        assert!(starts.contains(&r.start));
    }
}
