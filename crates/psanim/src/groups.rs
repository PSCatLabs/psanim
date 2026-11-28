//! Whether the program has a transparency group open: the one place psanim
//! asks.
//!
//! PSCat's `begingroup` paints on a layer of its own, and while a group is
//! open `swap_canvas` exchanges the page *below* that layer. psanim draws
//! its own layers (`faded`, `blurred`, the motions, the pixel effects) by
//! swapping the canvas, so it must know whether a group is open before it
//! swaps, and whether the group it ran a body in is still the innermost one
//! when the body ends.
//!
//! **Pending PSCatLabs/pscat#1494.** PSCat 0.8.0 (the pinned `d385971`)
//! gives a host no supported way to ask either question: the group stack is
//! a private field of `Gfx`, and `Interp::gfx()` is `#[doc(hidden)]`. An
//! earlier version of this branch counted `begingroup` / `endgroup` by
//! wrapping them in the prelude; that cannot be made safe (a pattern's
//! `endgroup` closes nothing at the page's level, and `//begingroup` must
//! stay an operator), so it was removed. Until the interpreter answers,
//! these report what psanim knows for sure: nothing, which is how psanim
//! behaved before groups existed. Every caller is written against the
//! final meaning, and the tests that need a real answer are `#[ignore]`d
//! with the issue number.

use pscat::Interp;

/// Identifies one open transparency group for as long as it is open: two
/// answers are equal only if they name the same `begingroup`.
// Made by `innermost_group` once pscat#1494 lands.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GroupId(pub u64);

/// How many transparency groups are open at the page's level, the level
/// `swap_canvas` exchanges below: what the program paints goes to a group's
/// layer when this is above 0. Counted as the interpreter counts them: a
/// group opened by any route counts, and one that `endgroup`, `erasepage`,
/// `showpage`, a `restore` or an error ended does not.
///
/// Pending pscat#1494: always 0 until then.
pub(crate) fn groups_open(_interp: &Interp) -> usize {
    0
}

/// The innermost open group at the page's level, if any.
///
/// Pending pscat#1494: always `None` until then.
pub(crate) fn innermost_group(_interp: &Interp) -> Option<GroupId> {
    None
}
