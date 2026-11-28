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
//! PSCat answers both questions (`Interp::open_group_count` and
//! `innermost_group_serial`, PSCatLabs/pscat#1494). An earlier version of
//! this branch counted `begingroup` / `endgroup` by wrapping them in the
//! prelude; that cannot be made safe (a pattern's `endgroup` closes nothing
//! at the page's level, and `//begingroup` must stay an operator), so it was
//! removed.
//!
//! What PSCat counts: groups at the page's level, opened by any route. A
//! group is closed by `endgroup`, `erasepage`, `showpage`, `copypage`, a
//! `restore` past it, the end of a run, an uncaught error, a limit or the
//! interrupt flag. A *caught* error does not close one: `{ ... begingroup
//! ... err } stopped` leaves it open, as it leaves a `gsave` open. A group
//! opened under `nulldevice` is counted though it has no layer.

use pscat::Interp;

/// Identifies one open transparency group for as long as it is open: two
/// answers are equal only if they name the same `begingroup`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GroupId(pub u64);

/// How many transparency groups are open at the page's level, the level
/// `swap_canvas` exchanges below: what the program paints goes to a group's
/// layer when this is above 0. Counted as the interpreter counts them: a
/// group opened by any route counts, and one that `endgroup`, `erasepage`,
/// `showpage`, a `restore` or an uncaught error ended does not.
pub(crate) fn groups_open(interp: &Interp) -> usize {
    interp.open_group_count()
}

/// The innermost open group at the page's level, if any.
pub(crate) fn innermost_group(interp: &Interp) -> Option<GroupId> {
    interp.innermost_group_serial().map(GroupId)
}
