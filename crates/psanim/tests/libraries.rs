//! The expressive libraries, end to end: what their words promise, and
//! how they compose with each other and with the engine's time model.

use psanim::{Frame, Movie, Options};

const HEAD: &str = "<< /Size [200 120] /Duration 4 /FPS 10 /Background [0 0 0] >> animation
/pkstage use /pkmotion use /pktype use /pkdiagram use /pkcode use /pkchart use /pkink use /pkfigure use /pkpixel use\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

/// What `expr` prints (with `==`) when run in the scene at time `t`.
fn eval_at(setup: &str, expr: &str, t: f64) -> String {
    let mut m = load(&format!("{setup}\n/scene {{ {expr} }} def"));
    m.take_output();
    m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

fn nums(text: &str) -> Vec<f64> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
        .filter_map(|s| s.parse().ok())
        .collect()
}

fn pixels(m: &mut Movie, t: f64) -> Vec<u8> {
    m.render_at(t)
        .unwrap_or_else(|e| panic!("{e}"))
        .premultiplied
        .to_vec()
}

/// The box of pixels that are not black, as (x0, y0, x1, y1), and how many.
fn ink(f: &Frame) -> Option<(u32, u32, u32, u32, usize)> {
    let mut b: Option<(u32, u32, u32, u32, usize)> = None;
    for y in 0..f.height {
        for x in 0..f.width {
            let o = ((y * f.width + x) * 4) as usize;
            if f.premultiplied[o..o + 3].iter().any(|c| *c > 8) {
                b = Some(match b {
                    None => (x, y, x + 1, y + 1, 1),
                    Some((x0, y0, x1, y1, n)) => {
                        (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1), n + 1)
                    }
                });
            }
        }
    }
    b
}

fn ink_at(m: &mut Movie, t: f64) -> Option<(u32, u32, u32, u32, usize)> {
    ink(&m.render_at(t).unwrap_or_else(|e| panic!("{e}")))
}

const BLOCK: &str = "/blk { 1 1 1 setrgbcolor 60 40 80 40 rectfill } def";

// --- motions ---------------------------------------------------------------

#[test]
fn an_entrance_is_absent_before_moving_during_and_exactly_the_drawing_after() {
    let mut plain = load(&format!("{BLOCK} /scene {{ blk }} def"));
    let rest = pixels(&mut plain, 0.0);
    for name in [
        "fade",
        "rise",
        "fall",
        "slide-left",
        "slide-right",
        "pop",
        "grow",
        "zoom",
        "drop",
        "focus",
        "spin",
        "flip",
        "unfold",
        "unroll",
        "wipe-right",
        "wipe-left",
        "wipe-up",
        "wipe-down",
        "mask-up",
        "mask-down",
        "mask-right",
    ] {
        let mut m = load(&format!(
            "{BLOCK} /scene {{ 1 [/{name} 1] {{ blk }} enters }} def"
        ));
        assert!(
            ink_at(&mut m, 0.5).is_none(),
            "{name}: visible before its time"
        );
        assert!(ink_at(&mut m, 1.0).is_none(), "{name}: visible at u = 0");
        let mid = pixels(&mut m, 1.4);
        assert!(mid != rest, "{name}: already at rest part-way");
        assert!(mid.iter().any(|b| *b > 8), "{name}: nothing drawn part-way");
        assert_eq!(
            pixels(&mut m, 2.1),
            rest,
            "{name}: not the plain drawing once done"
        );
    }
}

#[test]
fn an_emphasis_rests_before_and_after_and_moves_between() {
    let mut plain = load(&format!("{BLOCK} /scene {{ blk }} def"));
    let rest = pixels(&mut plain, 0.0);
    for name in [
        "pulse", "shake", "wobble", "jump", "squash", "blink", "tada", "dim",
    ] {
        let mut m = load(&format!(
            "{BLOCK} /scene {{ 1 [/{name} 1] {{ blk }} emphasized }} def"
        ));
        assert_eq!(pixels(&mut m, 0.5), rest, "{name}: moves before its time");
        assert!(pixels(&mut m, 1.4) != rest, "{name}: does nothing");
        assert_eq!(pixels(&mut m, 2.2), rest, "{name}: does not come back");
    }
}

#[test]
fn a_motion_measures_the_drawing_so_scale_is_about_its_own_centre() {
    // The block's centre is (100, 60) in canvas units; pixels are y down.
    let mut m = load(&format!(
        "{BLOCK} /scene {{ 0.5 [/grow 1 /linear] {{ blk }} motion }} def"
    ));
    let (x0, y0, x1, y1, _) = ink_at(&mut m, 0.0).unwrap();
    assert!((f64::from(x0 + x1) / 2.0 - 100.0).abs() <= 1.0, "{x0} {x1}");
    assert!((f64::from(y0 + y1) / 2.0 - 60.0).abs() <= 1.0, "{y0} {y1}");
    assert!(
        (38..=42).contains(&(x1 - x0)),
        "half of 80 wide: {}",
        x1 - x0
    );
    // /unfold grows from the bottom edge: the bottom stays put.
    let mut m = load(&format!(
        "{BLOCK} /scene {{ 0.5 /unfold {{ blk }} motion }} def"
    ));
    let (_, y0, _, y1, _) = ink_at(&mut m, 0.0).unwrap();
    assert_eq!(y1, 80, "the bottom edge (canvas y = 40) must not move");
    assert!((18..=22).contains(&(y1 - y0)));
}

#[test]
fn a_masked_motion_never_leaves_the_drawings_box_and_a_wipe_shows_part_of_it() {
    let mut m = load(&format!(
        "{BLOCK} /scene {{ 0 [/mask-up 2 /linear] {{ blk }} enters }} def"
    ));
    for t in [0.3, 0.8, 1.4] {
        let (x0, y0, x1, y1, _) = ink_at(&mut m, t).unwrap();
        assert!(
            x0 >= 60 && x1 <= 140 && y0 >= 40 && y1 <= 80,
            "t={t}: {x0} {y0} {x1} {y1}"
        );
    }
    let mut m = load(&format!(
        "{BLOCK} /scene {{ 0.25 /wipe-right {{ blk }} motion }} def"
    ));
    let (x0, _, x1, _, _) = ink_at(&mut m, 0.0).unwrap();
    assert_eq!((x0, x1), (60, 80));
}

#[test]
fn leaving_is_entering_run_backward() {
    let mut a = load(&format!(
        "{BLOCK} /scene {{ 0 2 {{ [/rise 1] {{ blk }} entering }} during }} def"
    ));
    let mut b = load(&format!(
        "{BLOCK} /scene {{ 0 2 {{ [/rise 1] {{ blk }} leaving }} during }} def"
    ));
    assert_eq!(pixels(&mut a, 0.3), pixels(&mut b, 1.7));
    assert_eq!(pixels(&mut a, 0.6), pixels(&mut b, 1.4));
    // onstage and staged: there in the middle, gone at both ends.
    for body in [
        "1 3 /pop /fade { blk } onstage",
        "1 3 { /pop /fade { blk } staged } during",
    ] {
        let mut m = load(&format!("{BLOCK} /scene {{ {body} }} def"));
        assert!(ink_at(&mut m, 0.5).is_none(), "{body}");
        assert_eq!(ink_at(&mut m, 2.0).map(|b| b.4), Some(80 * 40), "{body}");
        assert!(ink_at(&mut m, 3.2).is_none(), "{body}");
    }
}

#[test]
fn a_programs_own_motion_is_a_dictionary_from_u_and_bad_ones_are_explained() {
    let mut m = load(&format!(
        "{BLOCK} motions /sideways {{ 1 exch sub 30 mul /dx exch 2 array astore aload pop 2 dict begin def currentdict end }} put
         /scene {{ 0 [/sideways 2 /linear] {{ blk }} enters }} def"
    ));
    let (x0, _, _, _, _) = ink_at(&mut m, 1.0).unwrap();
    assert_eq!(x0, 75, "half-way: 15 units right of rest");
    let mut m = load(&format!(
        "{BLOCK} /scene {{ 0 /nosuch {{ blk }} enters }} def"
    ));
    let e = m.render_at(0.2).map(|_| ()).expect_err("unknown motion");
    assert!(
        e.to_string().contains("/fade") && e.to_string().contains("motions"),
        "{e}"
    );
    let mut m = load(&format!(
        "{BLOCK} motions /typo {{ pop << /scael 2 >> }} put /scene {{ 0 /typo {{ blk }} enters }} def"
    ));
    let e = m.render_at(0.2).map(|_| ()).expect_err("bad key");
    assert!(e.to_string().contains("/scael"), "{e}");
}

#[test]
fn swapped_and_echoed_and_rank() {
    let mut m = load(
        "/scene { 2 [/fade 0.5] { 1 0 0 setrgbcolor 0 0 100 120 rectfill } { 0 1 0 setrgbcolor 100 0 100 120 rectfill } swapped } def",
    );
    let (_, _, x1, _, _) = ink_at(&mut m, 1.0).unwrap();
    assert_eq!(x1, 100, "A alone before t");
    let (x0, _, _, _, _) = ink_at(&mut m, 3.0).unwrap();
    assert_eq!(x0, 100, "B alone after t");
    // echoed: the moving dot leaves fainter copies behind it.
    let mut m =
        load("/scene { 3 0.2 { 1 1 1 setrgbcolor now 40 mul 20 add 60 5 disc } echoed } def");
    let (x0, _, x1, _, _) = ink_at(&mut m, 2.0).unwrap();
    assert!(x1 - x0 > 30, "a trail, not one dot: {x0}..{x1}");
    assert_eq!(
        nums(&eval_at("", "[ 0 1 4 { 5 /center rank } for ] ==", 0.0)),
        [2.0, 1.0, 0.0, 1.0, 2.0]
    );
    assert_eq!(
        nums(&eval_at("", "[ 0 1 3 { 4 /reverse rank } for ] ==", 0.0)),
        [3.0, 2.0, 1.0, 0.0]
    );
}

// --- time: scripts, storyboards, cascades -----------------------------------

#[test]
fn a_script_runs_steps_one_after_another_and_nests() {
    let body = "{ 1 { (a) print now == } then  0.5 wait  1 { (b) print now == } thenfor
                  { 2 { (c) print now == } then } script
                  cursor == } script cursor ==";
    // t = 0.4: step a is live, and c (its script starts its own cursor at
    // 0); the cursors are 2.5 inside, 0 outside.
    let out = eval_at("", body, 0.4);
    assert!(out.starts_with('a') && !out.contains('b'), "{out}");
    assert_eq!(
        nums(&out.replace(['a', 'b', 'c'], " ")),
        [0.4, 0.4, 2.5, 0.0]
    );
    // t = 2.0: a (clock 2.0), b in its window (1.5..2.5: clock 0.5), c (nested, from 0).
    let out = eval_at("", body, 2.0);
    assert_eq!(
        nums(&out.replace(['a', 'b', 'c'], " ")),
        [2.0, 0.5, 2.0, 2.5, 0.0]
    );
    // t = 3: b is over.
    assert!(!eval_at("", body, 3.0).contains('b'));
}

#[test]
fn a_storyboard_names_beats_and_places_bodies_in_them() {
    let src = "[ /one 1 /two 2 /three 1 ] storyboard
        << /Size [20 20] /Duration storylength /FPS 10 >> animation
        /scene { /two beat == /two beatspan == == storylength ==
                 /two { (in) print now == span == } inbeat
                 0 1 { /three { (from) print now == } frombeat } during } def";
    let mut m = Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.meta().duration, 4.0);
    m.take_output();
    m.render_at(1.5).unwrap();
    let out = m.take_output();
    assert_eq!(
        nums(&out.replace("in", " ")),
        [1.0, 3.0, 1.0, 4.0, 0.5, 2.0],
        "{out}"
    );
    // frombeat ignores the enclosing scope: at t = 3.5 the `during` is closed.
    m.render_at(0.5).unwrap();
    assert!(!m.take_output().contains("from"));
    let mut m = Movie::load(
        "[ /a 1 ] storyboard << /Size [20 20] /Duration 1 >> animation /scene { /b beat pop } def",
        &Options::default(),
    )
    .unwrap();
    let e = m.render_at(0.0).map(|_| ()).expect_err("no such beat");
    assert!(e.to_string().contains("no beat /b"), "{e}");
}

#[test]
fn cascade_hands_each_item_its_index_and_a_later_clock() {
    let out = eval_at("", "[ 10 20 30 ] 0.5 { exch == == now == } cascade", 2.0);
    assert_eq!(nums(&out), [10.0, 0.0, 2.0, 20.0, 1.0, 1.5, 30.0, 2.0, 1.0]);
    // The body sees its caller's operands.
    assert_eq!(
        nums(&eval_at(
            "",
            "7 [ 1 2 ] 0 { pop pop dup == } cascade pop",
            0.0
        )),
        [7.0, 7.0]
    );
}

// --- space: boxes, viewports, the camera -------------------------------------

#[test]
fn boxes_cut_up_cover_their_parent_and_interpolate() {
    let b = "[ 10 20 180 90 ]";
    let out = eval_at(
        "",
        &format!("{b} 3 6 cols == {b} 2 10 rows == {b} 3 2 0 cells length =="),
        0.0,
    );
    let n = nums(&out);
    assert_eq!(n[..4], [10.0, 20.0, 56.0, 90.0]);
    assert_eq!(n[8..12], [134.0, 20.0, 56.0, 90.0]);
    assert_eq!(
        n[12..16],
        [10.0, 70.0, 180.0, 40.0],
        "rows run top to bottom"
    );
    assert_eq!(n[16..20], [10.0, 20.0, 180.0, 40.0]);
    assert_eq!(n[20], 6.0);
    let n = nums(&eval_at(
        "",
        &format!("{b} 2 2 [ 20 10 ] cells 3 get =="),
        0.0,
    ));
    assert_eq!(n, [110.0, 20.0, 80.0, 40.0], "a gap across and a gap down");
    let out = eval_at(
        "",
        &format!(
            "{b} 5 inset == {b} /ne boxpoint == == {b} 40 20 /c placed == 100 60 40 20 /c boxat == {b} 0.25 splitx == == {b} 0.25 splity == =="
        ),
        0.0,
    );
    assert_eq!(
        nums(&out),
        [
            15.0, 25.0, 170.0, 80.0, 110.0, 190.0, 80.0, 55.0, 40.0, 20.0, 80.0, 50.0, 40.0, 20.0,
            55.0, 20.0, 135.0, 90.0, 10.0, 20.0, 45.0, 90.0, 10.0, 20.0, 180.0, 67.5, 10.0, 87.5,
            180.0, 22.5
        ]
    );
    // A box is numbers: tweens and keyframes work on it.
    let out = eval_at("", "[ 0 [0 0 10 10]  2 [100 50 30 20] ] keyframes ==", 1.0);
    assert_eq!(nums(&out), [50.0, 25.0, 20.0, 15.0]);
}

#[test]
fn within_rebases_the_canvas_and_clips_and_nests() {
    let body = "[ 20 10 100 60 ] { width == height == canvasbox ==
                  [ 10 10 50 20 ] { width == } within  width ==
                  1 1 1 setrgbcolor -50 -50 500 500 rectfill } within width ==";
    let mut m = load(&format!("/scene {{ {body} }} def"));
    m.take_output();
    let f = m.render_at(0.0).unwrap();
    assert_eq!(
        ink(&f).map(|b| (b.0, b.1, b.2, b.3)),
        Some((20, 50, 120, 110)),
        "clipped to the box"
    );
    let out = m.take_output();
    assert_eq!(
        nums(&out),
        [100.0, 60.0, 0.0, 0.0, 100.0, 60.0, 50.0, 100.0, 200.0]
    );
    // fitted: a 400x400 scene in a 100x60 box is scaled to 60x60 and centred.
    let mut m = load(
        "/scene { [ 20 10 100 60 ] 400 400 { 1 1 1 setrgbcolor 0 0 width height rectfill } fitted } def",
    );
    assert_eq!(
        ink_at(&mut m, 0.0).map(|b| (b.0, b.1, b.2, b.3)),
        Some((40, 50, 100, 110))
    );
}

#[test]
fn lookat_fills_the_frame_with_a_box_and_visiblebox_reports_it() {
    let mut m = load(
        "/scene { gsave [ 50 30 20 12 ] lookat visiblebox == 1 1 1 setrgbcolor 50 30 20 12 rectfill grestore } def",
    );
    m.take_output();
    let f = m.render_at(0.0).unwrap();
    assert_eq!(ink(&f).map(|b| b.4), Some(200 * 120));
    let v = nums(&m.take_output());
    for (got, want) in v.iter().zip([50.0, 30.0, 20.0, 12.0]) {
        assert!((got - want).abs() < 1e-6, "{v:?}");
    }
    // spotlight washes everything outside the box toward the ground.
    let mut m =
        load("/scene { 1 1 1 setrgbcolor 0 0 200 120 rectfill [ 80 40 40 40 ] 1 spotlight } def");
    let f = m.render_at(0.0).unwrap();
    let at = |x: u32, y: u32| f.premultiplied[((y * f.width + x) * 4) as usize];
    assert_eq!(at(100, 60), 255);
    assert!(
        at(10, 10) < 20,
        "midnight's ground is near black: {}",
        at(10, 10)
    );
}

#[test]
fn themes_set_colour_and_type_by_role() {
    let out = eval_at(
        "/paper usetheme << /accent [0.5 0.25 1] >> usetheme",
        "/accent ink currentrgbcolor == == ==  /bg rgbof == == ==  [0.1 0.2 0.3] rgbof == == ==
         /mono 20 face fontsize ==  /accent /bg 0.5 mixink length ==",
        0.0,
    );
    let n = nums(&out);
    assert_eq!(n[..3], [1.0, 0.25, 0.5]);
    assert_eq!(n[3..6], [0.92, 0.95, 0.965], "paper's ground");
    assert!((n[9] - 20.0).abs() < 1e-3 && n[10] == 3.0, "{n:?}");
    let e = Movie::load(
        &format!("{HEAD} /nosuch usetheme /scene {{ }} def"),
        &Options::default(),
    )
    .map(|_| ())
    .expect_err("unknown theme");
    assert!(e.to_string().contains("rangecheck"), "{e}");
}

// --- type ---------------------------------------------------------------------

#[test]
fn text_is_measured_wrapped_and_fitted() {
    let setup = "/t (the quick brown fox jumps over the lazy dog again and again) def";
    let out = eval_at(
        setup,
        "/Helvetica 10 face t 90 wraplines dup length == { stringwidth pop 90 le == } forall",
        0.0,
    );
    assert!(out.starts_with('4') || out.starts_with('5'), "{out}");
    assert!(
        !out.contains("false"),
        "a line is wider than the measure: {out}"
    );
    // Words survive wrapping in order.
    let out = eval_at(
        setup,
        "/Helvetica 10 face t 90 wraplines { print ( ) print } forall",
        0.0,
    );
    assert_eq!(
        out,
        "the quick brown fox jumps over the lazy dog again and again"
    );
    // fitface fills the width (or the height, whichever binds).
    let out = eval_at(
        "",
        "(WIDE WORDS) [ 0 0 150 100 ] /Helvetica-Bold fitface (WIDE WORDS) stringwidth pop ==",
        0.0,
    );
    assert!((nums(&out)[0] - 150.0).abs() < 0.5, "{out}");
    let out = eval_at(
        "",
        "(I) [ 0 0 150 36 ] /Helvetica-Bold fitface fontsize ==",
        0.0,
    );
    assert!((nums(&out)[0] - 50.0).abs() < 0.01, "{out}");
    let out = eval_at(
        "",
        "/Helvetica 10 face 20 30 moveto (alpha beta) (beta) partbox == currentpoint == ==
         (a\\nb) [ 0 0 100 100 ] /center textlayout length ==",
        0.0,
    );
    let n = nums(&out);
    assert!(
        (n[0] - 47.25).abs() < 0.2,
        "the box starts after `alpha `: {n:?}"
    );
    assert_eq!(
        n[4..],
        [30.0, 20.0, 2.0],
        "the current point is kept; \\n breaks"
    );
}

#[test]
fn kinetic_text_ends_as_plain_text_and_starts_empty() {
    let font = "/Helvetica-Bold 24 face 1 1 1 setrgbcolor 20 50 moveto";
    let mut plain = load(&format!("/scene {{ {font} (many words here) show }} def"));
    let rest = pixels(&mut plain, 0.0);
    for unit in ["glyphs", "words", "whole"] {
        let mut m = load(&format!(
            "/scene {{ {font} (many words here) /{unit} 0.05 [/mask-up 0.4] kinetic }} def"
        ));
        assert!(ink_at(&mut m, 0.0).is_none(), "{unit}");
        let part = ink_at(&mut m, 0.3).map_or(0, |b| b.4);
        let all = ink_at(&mut m, 3.0).map_or(0, |b| b.4);
        assert!(part > 0 && all > 0, "{unit}: {part} of {all}");
        assert_eq!(pixels(&mut m, 3.0), rest, "{unit}");
    }
    let mut m = load(
        "/scene { /Helvetica 9 face 1 1 1 setrgbcolor
           (one two three four five six seven eight nine ten) [ 10 10 100 100 ] /left /lines 0.5 /fade kineticparagraph } def",
    );
    let (a, b) = (ink_at(&mut m, 0.3).unwrap(), ink_at(&mut m, 3.9).unwrap());
    assert!(a.3 < b.3, "later lines are lower on the page: {a:?} {b:?}");
}

#[test]
fn numbers_and_strings_format() {
    assert_eq!(
        eval_at("", "1234567.891 1 fmtnum print", 0.0),
        "1,234,567.9"
    );
    assert_eq!(
        eval_at("", "2026 0 fmtfixed print (a) (bc) strcat print", 0.0),
        "2026abc"
    );
    assert_eq!(
        eval_at(
            "",
            "(a  b\\nc) splitwords length == (a\\n\\nb) splitlines length ==",
            0.0
        ),
        "3\n3"
    );
    assert_eq!(
        nums(&eval_at("", "0 100 5 niceticks == [ 3 9 5 ] ranks ==", 0.0)),
        [0.0, 20.0, 40.0, 60.0, 80.0, 100.0, 2.0, 0.0, 1.0]
    );
}

#[test]
fn captions_show_in_their_time_only() {
    let mut m = load("/scene { [ [1 2 (hello there)] ] canvasbox << /Size 12 >> captions } def");
    assert!(ink_at(&mut m, 0.5).is_none());
    assert!(ink_at(&mut m, 1.5).is_some());
    assert!(ink_at(&mut m, 2.5).is_none());
}

// --- credits ---------------------------------------------------------------------

/// The columns (x) that hold ink in a frame.
fn ink_columns(f: &Frame) -> Vec<u32> {
    (0..f.width)
        .filter(|&x| {
            (0..f.height).any(|y| {
                let o = ((y * f.width + x) * 4) as usize;
                f.premultiplied[o..o + 3].iter().any(|c| *c > 8)
            })
        })
        .collect()
}

#[test]
fn credits_set_roles_and_names_on_either_side_of_a_gutter() {
    let mut m = load(
        "/scene { [ [/pair (Role) (Name)] ] << /Size 12 /Gutter 40 /Fade 0 >> 73 creditsat } def",
    );
    let cols = ink_columns(&m.render_at(0.0).unwrap());
    let left: Vec<u32> = cols.iter().copied().filter(|x| *x < 100).collect();
    let right: Vec<u32> = cols.iter().copied().filter(|x| *x >= 100).collect();
    assert!(!left.is_empty() && !right.is_empty(), "{cols:?}");
    assert!(
        *left.last().unwrap() <= 80,
        "role ends at the gutter: {left:?}"
    );
    assert!(
        *left.last().unwrap() >= 76,
        "role is right-aligned: {left:?}"
    );
    assert!(
        *right.first().unwrap() >= 119,
        "name starts after it: {right:?}"
    );
    assert!(
        *right.first().unwrap() <= 123,
        "name is left-aligned: {right:?}"
    );
}

#[test]
fn credits_timing_comes_from_the_duration_or_the_speed_and_ends_on_the_final_card() {
    let setup = "/body [ [/heading (Cast)] [/pair (Role) (Name)] [/space 2] [/name (Someone)] ] def
        /opts << /Duration 6 /Cards [ [[/name (Hello)]] ] /CardTime 2
                 /Final [ [/heading (END)] ] >> def";
    // cards, then the roll's own duration
    assert_eq!(
        nums(&eval_at(setup, "body opts creditsduration ==", 0.0)),
        [8.0]
    );
    // the speed follows: the distance covered in the roll's time
    let v = nums(&eval_at(setup, "body opts creditsspeed ==", 0.0))[0];
    let h = nums(&eval_at(setup, "body opts creditsheight ==", 0.0))[0];
    assert!(v > 0.0 && v * 6.0 > h / 2.0, "{v} {h}");
    // a given speed gives its own duration
    let d = nums(&eval_at(
        setup,
        "body << /Speed 20 /Box [0 0 200 120] /Final [ [/heading (END)] ] >> creditsduration ==",
        0.0,
    ))[0];
    assert!(
        (d * 20.0 - v * 6.0).abs() < 1e-6,
        "{d} at 20 covers what {v} does in 6 s"
    );
    let mut m = load(&format!("{setup} /scene {{ body opts credits }} def"));
    assert!(ink_at(&mut m, 0.0).is_none(), "the card fades in");
    assert!(ink_at(&mut m, 1.0).is_some(), "a card holds");
    assert!(
        ink_at(&mut m, 2.0).is_none(),
        "the roll starts below the box"
    );
    assert!(ink_at(&mut m, 5.0).is_some());
    // the final card stops where it is centred, and stays
    let end = pixels(&mut m, 8.0);
    assert_eq!(end, pixels(&mut m, 8.5));
    assert_ne!(end, pixels(&mut m, 7.0));
    let b = ink_at(&mut m, 8.0).unwrap();
    let cy = f64::from(b.1 + b.3) / 2.0;
    assert!((cy - 60.0).abs() < 3.0, "centred in the box: {b:?}");
}

#[test]
fn credits_roll_whole_pixels_so_type_does_not_shimmer() {
    // 73 units a second at 10 frames a second is 7.3 pixels a frame.
    let scene = |snap: &str| {
        load(&format!(
            "/scene {{ [ [/heading (Cast)] [/pair (Role) (Name)] [/pair (Other) (Person)] [/pair (More) (Folk)] [/name (Someone)] ]
               << /Size 9 /Fade 0 /Speed 73 /Snap {snap} >> credits }} def"
        ))
    };
    let shifted = |a: &[u8], b: &[u8], dy: usize| {
        // frame b is frame a moved up by dy rows, everywhere but the edges
        let row = 200 * 4;
        (50..88).all(|y| a[(y + dy) * row..(y + dy + 1) * row] == b[y * row..(y + 1) * row])
    };
    let mut snapped = scene("true");
    let mut loose = scene("false");
    let mut loose_exact = 0;
    for k in 10..14 {
        let t = f64::from(k) / 10.0;
        let (a, b) = (pixels(&mut snapped, t), pixels(&mut snapped, t + 0.1));
        assert!(
            (5..=9).any(|dy| shifted(&a, &b, dy)),
            "frame {k} is not the one before moved a whole number of pixels"
        );
        let (a, b) = (pixels(&mut loose, t), pixels(&mut loose, t + 0.1));
        if (5..=9).any(|dy| shifted(&a, &b, dy)) {
            loose_exact += 1;
        }
    }
    assert!(
        loose_exact < 4,
        "without snapping the type changes shape: {loose_exact}"
    );
}

#[test]
fn credits_glide_in_the_settle_tail_instead_of_hopping_a_pixel() {
    // 73 units a second slowing over 300 units: the last second or so is below a
    // pixel a frame, where snapping to whole pixels made the roll hold and hop.
    let names: String = (0..30).map(|i| format!("[/name (Person {i})] ")).collect();
    let opts = |snap: &str| {
        format!(
            "<< /Size 9 /Fade 0 /Speed 73 /Settle 300 /Snap {snap} /Final [ [/heading (END)] ] >>"
        )
    };
    let src = format!(
        "{}\n/scene {{ [ {names} ] {} credits }} def",
        HEAD.replace("/Duration 4", "/Duration 20"),
        opts("true")
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    let end = nums(&eval_at(
        "",
        &format!("[ {names} ] {} creditsduration ==", opts("true")),
        0.0,
    ))[0];
    // The tail's last stretch, where it moves less than a pixel a frame, up to the rest.
    let frames: Vec<Vec<u8>> = (0..8)
        .map(|k| pixels(&mut m, end - 0.8 + f64::from(k) / 10.0))
        .collect();
    let held = frames.windows(2).filter(|w| w[0] == w[1]).count();
    assert_eq!(held, 0, "the roll held still for {held} frames in its tail");
}

#[test]
fn a_credits_procedure_runs_at_its_slot_with_no_dictionary_left_open() {
    let mut m = load(
        "/scene { countdictstack ==
           [ [/proc { countdictstack == 1 1 1 setrgbcolor -5 -5 10 10 rectfill } 20] ] << /Fade 0 >> 70 creditsat } def",
    );
    m.take_output();
    let b = ink_at(&mut m, 0.0).unwrap();
    let out = nums(&m.take_output());
    assert_eq!(out[0], out[1], "dictionaries open: {out:?}");
    assert_eq!(
        (b.0 + b.2, b.1 + b.3),
        (200, 120),
        "the origin is the slot's centre, over the column's: {b:?}"
    );
}

#[test]
fn an_unknown_credits_entry_is_an_error() {
    let mut m = load("/scene { [ [/bogus (x)] ] << >> 0 creditsat } def");
    assert!(m.render_at(0.0).is_err());
}

// --- diagrams -------------------------------------------------------------------

#[test]
fn connectors_run_from_edge_to_edge_and_draw_on() {
    let boxes = "/a [ 10 40 40 30 ] def /b [ 150 40 40 30 ] def /c [ 150 90 40 20 ] def";
    for route in ["straight", "elbow", "arc", "arc-right", "hv", "vh"] {
        let out = eval_at(
            boxes,
            &format!("a c /{route} connect 0 pathpoint pop == == 1 pathpoint pop == == newpath"),
            0.0,
        );
        let n = nums(&out);
        // Starts just outside A and ends just outside C (the gap is 6).
        assert!(
            (0.0..=60.0).contains(&n[1]) && (30.0..=80.0).contains(&n[0]),
            "{route} start {n:?}"
        );
        assert!(
            (100.0..=196.0).contains(&n[3]) && (70.0..=116.0).contains(&n[2]),
            "{route} end {n:?}"
        );
    }
    let mut m = load(&format!(
        "{boxes} /scene {{ a b << /Draw 0 2 ramp /Role [1 1 1] /Label (x) /Pulse 2 3.9 ramp >> edge }} def"
    ));
    assert!(ink_at(&mut m, 0.0).is_none(), "undrawn");
    let (x0, _, x1, _, _) = ink_at(&mut m, 1.0).unwrap();
    assert!(
        x0 >= 50 && x1 < 115,
        "half drawn, between the boxes: {x0}..{x1}"
    );
    let (_, _, x1, _, _) = ink_at(&mut m, 3.0).unwrap();
    assert!((140..=150).contains(&x1), "drawn to B's edge: {x1}");
    let mut m = load(&format!(
        "{boxes} /scene {{ 1 1 1 setrgbcolor (A) a /surface node (g) a b boxunion -4 inset /muted group a circled 0 1 trimpath stroke a /n bracepath stroke a struck stroke /body 8 face (n) b /n 1 note }} def"
    ));
    assert!(ink_at(&mut m, 0.0).is_some());
}

#[test]
fn node_shapes_fill_their_boxes() {
    for shape in ["box", "pill", "circle", "diamond", "cylinder"] {
        let mut m = load(&format!(
            "/scene {{ (n) [ 40 30 120 60 ] << /Shape /{shape} /Fill [1 1 1] /Stroke [1 1 1] /Ink [1 1 1] >> node }} def"
        ));
        let (x0, y0, x1, y1, n) = ink_at(&mut m, 0.0).unwrap();
        assert!(
            x0 >= 37 && x1 <= 163 && y0 >= 27 && y1 <= 93,
            "{shape}: {x0} {y0} {x1} {y1}"
        );
        assert!(
            x1 - x0 >= 118 && y1 - y0 >= 58,
            "{shape} does not reach its box's edges"
        );
        assert!(n > 120 * 60 / 2 - 200, "{shape} is hollow: {n}");
    }
}

// --- code -----------------------------------------------------------------------

#[test]
fn code_types_on_focuses_and_diffs() {
    let setup = "/src (fn main() {\\n    let x = 1;\\n}\\n) /rust codetokens def
                 /rows (a = 1\\nb = 2\\nc = 3\\n) (a = 1\\nc = 3\\nd = 4\\n) /python diffrows def
                 /p [ 10 10 180 100 ] def";
    assert_eq!(
        nums(&eval_at(
            setup,
            "src length == rows length == [ rows { 0 get } forall ] ==",
            0.0
        ))[..2],
        [3.0, 4.0]
    );
    assert!(eval_at(setup, "[ rows { 0 get } forall ] ==", 0.0).contains("/same /del /same /add"));
    let mut m = load(&format!(
        "{setup} /scene {{ src p << /Plate false /Reveal 0 2 ramp >> codeblock }} def"
    ));
    let counts: Vec<usize> = [0.0, 1.0, 2.0]
        .iter()
        .map(|t| ink_at(&mut m, *t).map_or(0, |b| b.4))
        .collect();
    assert!(
        counts[0] == 0 && counts[1] > 0 && counts[2] > counts[1],
        "{counts:?}"
    );
    // The box of a line lies inside the panel, below the line before it.
    let n = nums(&eval_at(
        setup,
        "src p << >> 0 codelinebox == src p << >> 1 codelinebox ==",
        0.0,
    ));
    assert!(
        n[1] > n[5] && n[4] > n[0],
        "line 1 is lower and indented: {n:?}"
    );
    assert!(
        n[0] >= 10.0 && n[5] >= 10.0 && n[1] + n[3] <= 110.0,
        "{n:?}"
    );
    // Focus dims the other lines.
    let mut a = load(&format!(
        "{setup} /scene {{ src p << /Plate false >> codeblock }} def"
    ));
    let mut b = load(&format!(
        "{setup} /scene {{ src p << /Plate false /Focus [1] >> codeblock }} def"
    ));
    assert!(pixels(&mut a, 0.0) != pixels(&mut b, 0.0));
    // The diff's two ends differ and it moves between them.
    let mut m = load(&format!(
        "{setup} /scene {{ rows p << >> 0 4 ramp codediff }} def"
    ));
    let (start, mid, end) = (
        pixels(&mut m, 0.0),
        pixels(&mut m, 2.0),
        pixels(&mut m, 3.99),
    );
    assert!(start != mid && mid != end && start != end);
    let e = Movie::load(
        &format!("{HEAD} (x) /cobol codetokens pop /scene {{ }} def"),
        &Options::default(),
    )
    .map(|_| ())
    .expect_err("unknown language");
    assert!(e.to_string().contains("/rust"), "{e}");
}

// --- charts ---------------------------------------------------------------------

#[test]
fn charts_map_data_and_grow() {
    let frame = "[ 20 20 160 80 ] [ 0 10 ] [ 0 50 ] plotframe";
    assert_eq!(
        nums(&eval_at(
            "",
            &format!("{frame} 5 25 plotpoint == == plotbox =="),
            0.0
        )),
        [60.0, 100.0, 20.0, 20.0, 160.0, 80.0]
    );
    let mut m = load(&format!(
        "/scene {{ {frame} [ 10 50 25 ] << /Grow 0 2 ramp /Values false /Role [1 1 1] /Stagger 0 /Gap 0 >> bars }} def"
    ));
    assert!(ink_at(&mut m, 0.0).is_none());
    let (x0, y0, x1, y1, n) = ink_at(&mut m, 3.0).unwrap();
    assert_eq!(
        (x0, x1, y0, y1),
        (20, 180, 20, 100),
        "the tallest bar reaches the top of the frame"
    );
    // Areas are in proportion to the values: (10 + 50 + 25) / 50 of one column.
    let column = 160.0 / 3.0 * 80.0;
    assert!((n as f64 - column * 85.0 / 50.0).abs() < 200.0, "{n}");
    let part = ink_at(&mut m, 0.5).unwrap().4;
    assert!(part > 0 && part < n);
    // A race: the bar that becomes largest moves to the top row.
    let mut m = load(&format!(
        "/A [ 10 40 ] def /B [ 45 20 ] def
         /scene {{ {frame} /u 0 4 ramp def [ u A B between 0 get 0 ] << /Values false /Role [1 1 1] /Gap 0
                  /Ranks u A ranks B ranks between >> hbars }} def"
    ));
    let (top0, top1) = (
        ink_at(&mut m, 0.0).unwrap().1,
        ink_at(&mut m, 3.99).unwrap().1,
    );
    assert!(
        top0 > top1 + 30,
        "from the second row to the first: {top0} -> {top1}"
    );
    let mut m = load(&format!(
        "/scene {{ {frame} 1 1 1 setrgbcolor << /Draw 0 1 ramp >> plotaxes
           [ [0 0] [5 40] [10 20] ] plotline 0 0 2 ramp trimpath stroke [ [5 40] ] 1 2 ramp 3 dots
           [ 1 2 3 ] [ 20 20 60 60 ] << /Grow 0 2 ramp /Center (x) >> donut
           [ 1 5 3 ] seriesline plotarea fill }} def"
    ));
    assert!(ink_at(&mut m, 1.5).is_some());
}

#[test]
fn dashboard_words_colour_by_threshold_fold_bands_and_format() {
    let th = "[ /good 70 /warn 90 /bad ] thresholdrole ==";
    assert_eq!(
        eval_at("", &format!("10 {th} 70 {th} 95 {th}"), 0.0),
        "/good\n/warn\n/bad"
    );
    assert_eq!(
        eval_at(
            "",
            "12431 1 fmtshort == 999.96 1 fmtshort == 3.2e6 0 fmtshort ==",
            0.0
        ),
        "(12.4k)\n(1.0k)\n(3M)"
    );
    // A horizon chart folds a ramp into bands: one row, every band touched,
    // nothing outside its box; /Draw reveals it from the left.
    let ramp = "/rp [ 0 1 50 { 2 mul } for ] def";
    let mut m = load(&format!(
        "{ramp} /scene {{ rp [ 20 40 160 20 ] << /Extent 100 /Bands 4 /Role [1 1 1] /Ground [0 0 0] /Draw 0 4 ramp >> horizon }} def"
    ));
    assert!(ink_at(&mut m, 0.0).is_none());
    let (x0, y0, x1, y1, _) = ink_at(&mut m, 4.0).unwrap();
    assert!(
        x0 >= 20 && x1 <= 180 && y0 >= 60 && y1 <= 80,
        "{x0} {y0} {x1} {y1}"
    );
    let half = ink_at(&mut m, 2.0).unwrap();
    assert!(half.2 <= 101, "half drawn reaches the middle: {half:?}");
    // The highest band is the brightest: the right end is white, the left end dark.
    let f = m.render_at(4.0).unwrap();
    let at = |x: u32, y: u32| f.premultiplied[((y * f.width + x) * 4) as usize];
    assert!(
        at(178, 70) > 240 && at(178, 70) > at(60, 79),
        "{} {}",
        at(178, 70),
        at(60, 79)
    );
    // Negative values hang from the top in the other role.
    let mut m = load(
        "/scene { [ 0 -50 -100 ] [ 20 40 160 20 ] << /Extent 100 /Bands 1 /NegRole [1 0 0] /Ground [0 0 0] >> horizon } def",
    );
    let f = m.render_at(0.0).unwrap();
    let px = |x: u32, y: u32| {
        let o = ((y * f.width + x) * 4) as usize;
        [f.premultiplied[o], f.premultiplied[o + 1]]
    };
    assert_eq!(px(175, 61), [255, 0], "red at the top right");
    assert_eq!(px(30, 78), [0, 0], "nothing near zero");
    // Every panel word draws inside its box, on any theme.
    for theme in ["midnight", "paper"] {
        let mut m = load(&format!(
            "/{theme} usetheme /s [ 3 1 4 1 5 9 2 6 ] def
             /scene {{ 0 0 0 setrgbcolor 0 0 200 120 rectfill
               80 [ 10 10 90 50 ] << /Title (CPU) /Unit (%) /Thresholds [/good 70 /warn 90 /bad] /Delta -2 /Spark s >> stattile
               62 [ 110 10 80 50 ] << /Unit (%) /Label (disk) >> dial
               s [ 10 70 50 40 ] << >> sparkline
               [ s s ] [ 70 70 50 40 ] << /Roles [ /surface /accent /bad ] >> heatmap
               170 90 6 /good statuslight
               [ 130 70 30 40 ] << /Title (x) /Status /bad >> panel pop }} def"
        ));
        assert!(ink_at(&mut m, 0.0).is_some());
    }
}

#[test]
fn ticks_fade_while_a_range_moves() {
    // Half way between the steps 20 and 50 the in-between ticks are half shown.
    let out = eval_at("", "0 31.62 5 mul 5 fadeticks ==", 0.0);
    let v = nums(&out);
    let pairs: Vec<(f64, f64)> = v.chunks(2).map(|p| (p[0], p[1])).collect();
    assert!(
        pairs.contains(&(0.0, 1.0)) && pairs.contains(&(100.0, 1.0)),
        "{out}"
    );
    assert!(
        pairs.iter().any(|p| p.0 == 40.0 && p.1 > 0.3 && p.1 < 0.7),
        "{out}"
    );
    // plotaxes takes them with /Fade.
    let mut m = load(
        "/scene { 1 1 1 setrgbcolor [ 40 20 140 80 ] [ 0 1 ] [ 0 0 4 ramp 100 mul 60 add ] plotframe
                  << /Fade true /YTicks 5 >> plotaxes } def",
    );
    for t in [0.0, 1.3, 2.7, 4.0] {
        assert!(ink_at(&mut m, t).is_some());
    }
    assert_eq!(
        nums(&eval_at(
            "",
            "[ 0 10 ] [ 0 0 100 50 ] [ 0 10 ] seriespoints ==",
            0.0
        )),
        [0.0, 0.0, 100.0, 50.0]
    );
}

// --- ink ------------------------------------------------------------------------

#[test]
fn marks_taper_reveal_and_repeat() {
    let line = "newpath 20 60 moveto 180 60 lineto";
    let mut m = load(&format!(
        "/scene {{ 1 1 1 setrgbcolor {line} [ 0 20 0 ] brushstroke }} def"
    ));
    let (x0, y0, x1, y1, n) = ink_at(&mut m, 0.0).unwrap();
    assert!(
        x0 >= 20 && x1 <= 180 && (18..=21).contains(&(y1 - y0)),
        "{x0} {y0} {x1} {y1}"
    );
    assert!(
        (n as f64 - 160.0 * 10.0).abs() < 250.0,
        "a diamond of area 1600: {n}"
    );
    let mut m = load(&format!(
        "/scene {{ {line} 0 4 ramp 30 {{ 1 1 1 setrgbcolor 0 0 200 120 rectfill }} revealalong }} def"
    ));
    assert!(ink_at(&mut m, 0.0).is_none());
    let (x0, y0, x1, y1, _) = ink_at(&mut m, 2.0).unwrap();
    assert!(
        x0 <= 20 && (95..=118).contains(&x1) && y0 == 45 && y1 == 75,
        "{x0} {y0} {x1} {y1}"
    );
    assert_eq!(
        eval_at("", "100 60 translate 4 { == } around", 0.0),
        "0\n1\n2\n3"
    );
    let mut m = load("/scene { [ 0 0 200 120 ] [1 0 0] [0 0 1] 0 boxgradient } def");
    let f = m.render_at(0.0).unwrap();
    let px = |x: u32| f.premultiplied[((60 * f.width + x) * 4) as usize..][..3].to_vec();
    assert!(px(5)[0] > 200 && px(195)[2] > 200 && px(100)[0] > 90 && px(100)[2] > 90);
    let n = nums(&eval_at("", "7 5 jitter == == 7 5 jitter == ==", 0.0));
    assert!(
        n[0] == n[2] && n[1] == n[3] && n.iter().all(|v| v.abs() <= 5.0),
        "{n:?}"
    );
}

// --- composition ------------------------------------------------------------------

const EVERYTHING: &str = "
  /src (let x = 1;\\n) /rust codetokens def
  /g canvasbox 4 inset 2 2 4 cells def
  /scene {
    ground
    g 0 get { ground 0 /pop { (node) canvasbox 6 inset /accent node } enters } within
    {
      0.5 { /rise { /title 10 face /fg ink 104 100 moveto (kinetic) /glyphs 0.05 /mask-up kinetic } entering } then
      0.5 { g 0 get g 3 get << /Route /elbow /Draw 0 1 ramp /Pulse 1 2 pulsetrain >> edge } then
    } script
    g 2 get [0 3] [0 10] plotframe [ 3 9 6 ] << /Grow 0.5 2 ramp /Size 5 >> bars
    src g 3 get << /Reveal 0 2 ramp >> codeblock
    2 0.1 { /accent2 ink 100 now 20 mul add 60 3 disc } echoed
    gsave 100 60 translate 6 { pop /accent3 ink newpath 10 0 moveto 20 0 lineto [ 0 3 0 ] brushstroke } around grestore
    g 1 get 1 3 ramp 0.4 mul spotlight
  } def";

#[test]
fn the_libraries_compose_and_a_frame_still_depends_on_its_time_alone() {
    let mut a = load(EVERYTHING);
    let want: Vec<Vec<u8>> = [0.0, 0.7, 1.3, 2.6, 3.9]
        .iter()
        .map(|t| pixels(&mut a, *t))
        .collect();
    // The same frames in another order, from another interpreter.
    let mut b = load(EVERYTHING);
    for (k, t) in [3.9, 1.3, 0.0, 2.6, 0.7].iter().enumerate() {
        let i = [4, 2, 0, 3, 1][k];
        assert_eq!(
            pixels(&mut b, *t),
            want[i],
            "frame at t={t} depends on what came before it"
        );
    }
    assert!(want.windows(2).all(|w| w[0] != w[1]), "it should be moving");
}

#[test]
fn a_scene_is_a_function_of_its_viewport_as_well_as_its_time() {
    // The same scene full-frame and in a half-size box: every library
    // measures from width, height and vmin, so the small one is the large
    // one scaled (compared by where the ink falls, to a pixel or two).
    let scene = "/demo { canvasbox 10 vmin inset 2 4 vmin cols
        { 1 1 1 setrgbcolor 3 vmin roundpath fill } forall } def";
    let mut full = load(&format!("{scene} /scene {{ demo }} def"));
    let mut half = load(&format!(
        "{scene} /scene {{ [ 0 0 100 60 ] {{ demo }} within }} def"
    ));
    let (a, b) = (
        ink_at(&mut full, 0.0).unwrap(),
        ink_at(&mut half, 0.0).unwrap(),
    );
    // Full: inset 12 on a 200x120 canvas. Half: inset 6 on 100x60, at the bottom left (pixel rows 60..120).
    assert_eq!((a.0, a.2), (12, 188));
    assert_eq!((b.0, b.2, b.1, b.3), (6, 94, 66, 114));
}

// --- added after the first productions' friction reports ----------------------------

#[test]
fn equations_are_runs_that_can_be_shown_whole_or_term_by_term() {
    let eq = "/eq [ (A = ) /pi [/it (r)] [/sup (2)] ( + ) [/frac (1) (2)] ] def";
    let font = "/Times-Roman 30 face 1 1 1 setrgbcolor 20 50 moveto";
    let mut whole = load(&format!("{eq} /scene {{ {font} eq showruns }} def"));
    let mut terms = load(&format!(
        "{eq} /scene {{ /Times-Roman 30 face 1 1 1 setrgbcolor 0 1 eq length 1 sub {{ 20 50 moveto eq exch showrun }} for }} def"
    ));
    assert_eq!(
        pixels(&mut whole, 0.0),
        pixels(&mut terms, 0.0),
        "the terms land where the line puts them"
    );
    let n = nums(&eval_at(
        eq,
        &format!("{font} eq runwidth == eq 2 runbox == currentpoint == =="),
        0.0,
    ));
    assert!(n[0] > 100.0 && n[0] < 180.0, "{n:?}");
    assert!(
        n[1] > 60.0 && n[3] > 5.0 && n[3] < 20.0,
        "the box of the italic r: {n:?}"
    );
    assert_eq!(n[5..], [50.0, 20.0], "runbox keeps the current point");
    // A sign shows ink where a non-ASCII character would show none.
    let mut m = load(
        "/scene { /Helvetica 30 face 1 1 1 setrgbcolor 20 50 moveto /neq sym /arrow sym } def",
    );
    assert!(ink_at(&mut m, 0.0).unwrap().4 > 100);
}

#[test]
fn a_path_straightens_without_changing_length() {
    let out = eval_at(
        "",
        "newpath 100 60 40 0 180 arc pathlength == 1 0 straightenpath pathlength == pathbbox == == == == newpath",
        0.0,
    );
    let n = nums(&out);
    assert!((n[0] - n[1]).abs() < 0.01, "{n:?}");
    // Anchored at its start (140, 60) heading up: a vertical line. The
    // box prints as top, right, bottom, left.
    assert!(
        (n[3] - 140.0).abs() < 0.5 && (n[5] - 140.0).abs() < 3.0,
        "{n:?}"
    );
    assert!(
        (n[4] - 60.0).abs() < 0.5 && (n[2] - 60.0 - n[0]).abs() < 0.5,
        "{n:?}"
    );
}

#[test]
fn tempo_counts_and_kicks() {
    assert_eq!(
        nums(&eval_at("90 tempo", "3 counts == countnow == kick ==", 2.0)),
        [2.0, 3.0, 1.0]
    );
    let k = nums(&eval_at("60 tempo", "kick ==", 1.5))[0];
    assert!(k > 0.0 && k < 0.2, "{k}");
    assert_eq!(
        eval_at(
            "",
            "7 3 zeropad print ( ) print 65 fmtclock print ( ) print 3725 fmtclock print",
            0.0
        ),
        "007 01:05 1:02:05"
    );
    assert_eq!(
        nums(&eval_at(
            "",
            "[ 0 10 40 ] 1.5 valueat == [ [0 0] [2 4] ] 0.5 valueat ==",
            0.0
        )),
        [25.0, 1.0, 2.0]
    );
}

#[test]
fn time_of_day_from_a_start_and_the_movie_clock_or_a_signal() {
    // clockparts splits and wraps; fmtday writes the 24 hour clock.
    assert_eq!(
        nums(&eval_at("", "45296.25 clockparts == == == ==", 0.0)),
        [0.25, 56.0, 34.0, 12.0]
    );
    assert_eq!(
        nums(&eval_at("", "-1 clockparts == == == ==", 0.0)),
        [0.0, 59.0, 59.0, 23.0]
    );
    assert_eq!(eval_at("", "3725 fmtday ==", 0.0), "(01:02:05)");
    // parseclock takes text or seconds, and says what it accepts.
    assert_eq!(
        nums(&eval_at(
            "",
            "(7:05) parseclock == 100 parseclock == (0:00:01.5) parseclock ==",
            0.0
        )),
        [25500.0, 100.0, 1.5]
    );
    let mut m = load("/scene { (nonsense) parseclock } def");
    let e = m
        .render_at(0.0)
        .map(|_| ())
        .expect_err("a bad time is an error")
        .to_string();
    assert!(e.contains("HH:MM:SS"), "{e}");
    // daytime: the start plus the movie time, across midnight.
    assert_eq!(
        eval_at("", "(23:59:50) null daytime fmtday ==", 15.0),
        "(00:00:05)"
    );
    assert_eq!(nums(&eval_at("", "36000 null daytime ==", 2.5)), [36002.5]);
    // With a signal name the host's value is the time and the start is ignored.
    let mut m = load("/scene { (01:00:00) /clock daytime fmtday == } def");
    m.set_signal("clock", 83_000.0);
    m.take_output();
    m.render_at(3.0).unwrap();
    assert_eq!(m.take_output().trim(), "(23:03:20)");
    // A signal named by text, as --param gives it, works too.
    let mut m = load("/scene { 0 (clock) daytime == } def");
    m.set_signal("clock", 61.5);
    m.take_output();
    m.render_at(0.0).unwrap();
    assert_eq!(m.take_output().trim(), "61.5");
}

#[test]
fn more_box_words_and_text_placed_for_kinetic() {
    let n = nums(&eval_at(
        "",
        "[ 0 0 100 50 ] 10 5 insetxy == [ 0 0 100 50 ] 0.5 0 0.5 0.2 subbox == aspect ==",
        0.0,
    ));
    assert_eq!(n[..8], [10.0, 5.0, 80.0, 40.0, 50.0, 0.0, 50.0, 10.0]);
    assert!((n[8] - 200.0 / 120.0).abs() < 1e-9);
    let font = "/Helvetica-Bold 20 face 1 1 1 setrgbcolor";
    let mut a = load(&format!(
        "/scene {{ {font} (HI) [ 20 20 160 80 ] /c showin }} def"
    ));
    let mut b = load(&format!(
        "/scene {{ {font} (HI) [ 20 20 160 80 ] /c placetext (HI) show }} def"
    ));
    assert_eq!(pixels(&mut a, 0.0), pixels(&mut b, 0.0));
}

#[test]
fn edges_take_sides_and_waypoints_and_charts_take_marks() {
    let boxes = "/a [ 10 80 40 30 ] def /b [ 150 10 40 30 ] def";
    let n = nums(&eval_at(
        boxes,
        "a b [/s /n] connect 0 pathpoint pop == == 1 pathpoint pop == == newpath",
        0.0,
    ));
    for (got, want) in n.iter().zip([74.0, 30.0, 46.0, 170.0]) {
        assert!(
            (got - want).abs() < 1e-6,
            "out of A's bottom, into B's top, 6 clear of each: {n:?}"
        );
    }
    let n = nums(&eval_at(
        boxes,
        "a b [ 100 110 ] connect 0.5 pathpoint pop == == newpath",
        0.0,
    ));
    assert!(n[0] > 90.0, "the route climbs to its waypoint: {n:?}");
    let frame = "[ 30 30 150 70 ] [ 0 10 ] [ 0 100 ] plotframe";
    let mut m = load(&format!(
        "/scene {{ {frame} << /Y 50 /Label (half) /Draw 0 1 ramp >> plotrule
           5 50 << /Label (here) /Draw 1 2 ramp >> plotmark
           {{ 1 1 1 setrgbcolor 8 setlinewidth [ [0 0] [20 400] ] plotline stroke }} inplot }} def"
    ));
    let (x0, y0, x1, y1, _) = ink_at(&mut m, 0.0).unwrap();
    assert!(
        x0 >= 30 && x1 <= 180 && y0 >= 20 && y1 <= 90,
        "clipped to the frame: {x0} {y0} {x1} {y1}"
    );
    assert!(ink_at(&mut m, 3.0).unwrap().4 > ink_at(&mut m, 0.0).unwrap().4);
    // Bar labels say the data, not a count on its way there, unless asked.
    let mut m = load(&format!(
        "/scene {{ {frame} [ 80 ] << /Grow 0.5 /Stagger 0 >> bars }} def"
    ));
    let mut n = load(&format!(
        "/scene {{ {frame} [ 80 ] << /Grow 0.5 /Stagger 0 /Count true >> bars }} def"
    ));
    assert!(pixels(&mut m, 0.0) != pixels(&mut n, 0.0));
}

#[test]
fn a_band_of_a_stroke_is_revealed_for_persistent_painting() {
    let mut m = load(
        "/scene { newpath 20 60 moveto 180 60 lineto 0.25 0.5 30 { 1 1 1 setrgbcolor 0 0 200 120 rectfill } revealband } def",
    );
    assert_eq!(
        ink_at(&mut m, 0.0).map(|b| (b.0, b.1, b.2, b.3)),
        Some((60, 45, 100, 75))
    );
}

// --- pkfigure ------------------------------------------------------------

#[test]
fn a_dance_step_hops_higher_when_louder_and_grins_on_a_big_kick() {
    let pose = |loud: f64, since: f64| {
        eval_at(
            "",
            &format!(
                "<< /Beat 4 /Since {since} /Length 0.5 /Loud {loud} >> dancepose \
                 dup /Hop get == /Eyes get =="
            ),
            0.0,
        )
    };
    let quiet = pose(0.2, 0.19);
    let loud = pose(1.0, 0.19);
    assert!(nums(&loud)[0] > nums(&quiet)[0] * 1.5, "{quiet} / {loud}");
    assert!(loud.contains("grin"), "{loud}");
    assert!(quiet.contains("open"), "{quiet}");
    // On the beat itself and once landed, the feet are on the ground.
    assert_eq!(nums(&pose(1.0, 0.0))[0], 0.0);
    assert_eq!(nums(&pose(1.0, 0.49))[0], 0.0);
}

#[test]
fn a_spring_starts_at_one_and_dies_away_and_poses_blend() {
    let v = nums(&eval_at(
        "",
        "0 1.7 4.5 spring == -1 1.7 4.5 spring == 3 1.7 4.5 spring abs 0.01 lt ==",
        0.0,
    ));
    assert_eq!(&v[..2], &[1.0, 0.0]);
    let out = eval_at(
        "",
        "restpose << /Beat 0 /Since 0.1 /Length 0.5 /Loud 0.2 >> dancepose 0.5 blendpose \
         dup /ArmR get == /Eyes get ==",
        0.0,
    );
    // Halfway: arms between hanging (14) and the first move's (up), the eyes the dance's.
    assert!(nums(&out)[0] > 14.0 && nums(&out)[0] < 149.0, "{out}");
    assert!(out.contains("open"), "{out}");
}

#[test]
fn squashed_keeps_the_feet_on_the_ground_and_leaves_no_dictionary_open() {
    // A point on the ground stays put under any squash; the proc sees the caller's dictionaries.
    let out = eval_at(
        "/mine 7 def",
        "0 0.7 0.2 { 0 0 transform exch round cvi == round cvi == mine == } squashed \
         0 0 transform exch round cvi == round cvi ==",
        0.0,
    );
    let v = nums(&out);
    assert_eq!(v[2], 7.0, "{out}");
    assert_eq!((v[0], v[1]), (v[3], v[4]), "{out}");
}

// --- emoji -----------------------------------------------------------------

#[test]
fn emoji_text_measures_the_emoji_as_one_letter_wide() {
    let font = "/Helvetica findfont 20 scalefont setfont";
    let out = eval_at(
        "",
        &format!(
            "{font} (a:heart:b) emojitextwidth (a) stringwidth pop emojiwidth add (b) stringwidth pop add sub abs ="
        ),
        0.0,
    );
    assert!(out.parse::<f64>().unwrap() < 1e-6, "{out}");
    // A clock time and an unknown name are plain text.
    let out = eval_at(
        "",
        &format!(
            "{font} (9:41 :nope: x:) emojitextwidth (9:41 :nope: x:) stringwidth pop sub abs ="
        ),
        0.0,
    );
    assert!(out.parse::<f64>().unwrap() < 1e-6, "{out}");
    // An emoji is 1.2 times the font's size.
    let out = eval_at("", &format!("{font} emojiwidth ="), 0.0);
    assert!((out.parse::<f64>().unwrap() - 24.0).abs() < 1e-6, "{out}");
}

#[test]
fn emojilines_wrap_with_emoji_counted_at_their_width() {
    // Plain text wraps as wraplines does; a token counts as the emoji, not as its seven letters.
    let out = eval_at(
        "",
        "/Helvetica findfont 20 scalefont setfont
         (one two three four five) 120 emojilines length =
         (one two three four five) 120 wraplines length =
         (:laugh::laugh::laugh::laugh:) 100 emojilines length =
         (:laugh::laugh::laugh::laugh:) 100 wraplines length =",
        0.0,
    );
    let n: Vec<&str> = out.lines().collect();
    assert_eq!(n[0], n[1], "{out}");
    // 4 emoji of 24 units fit a 100-unit width as one unbroken word
    assert_eq!(n[2], "1", "{out}");
    assert_eq!(n[3], "1", "{out}");
}

const EMOJI_NAMES: [&str; 12] = [
    "smile", "laugh", "wink", "sad", "cool", "heart", "thumbsup", "party", "fire", "check",
    "rocket", "eyes",
];

#[test]
fn every_emoji_is_drawn_inside_its_square() {
    // emojinames lists what is drawn, in this order.
    let listed = eval_at("", "emojinames { == } forall", 0.0);
    assert_eq!(
        listed.split_whitespace().collect::<Vec<_>>(),
        EMOJI_NAMES
            .iter()
            .map(|n| format!("/{n}"))
            .collect::<Vec<_>>()
    );
    for name in EMOJI_NAMES {
        let mut m = load(&format!("/scene {{ /{name} 100 60 40 emojiat }} def"));
        let (x0, y0, x1, y1, n) =
            ink_at(&mut m, 0.0).unwrap_or_else(|| panic!("/{name} drew nothing"));
        // 40 across, centred on 100,60 (pixel rows count from the top of a 120 high canvas)
        assert!(
            x0 >= 79 && x1 <= 121 && y0 >= 39 && y1 <= 81,
            "/{name} left its square: {x0},{y0}..{x1},{y1}"
        );
        assert!(n > 150, "/{name} is nearly empty ({n} pixels)");
    }
}

#[test]
fn an_inline_emoji_sits_on_the_baseline_and_moves_the_point_on() {
    // 40 point font, baseline at y = 40: the emoji spans from just under it to the cap line
    let mut m = load(
        "/scene { /Helvetica findfont 40 scalefont setfont 20 40 moveto /heart emoji
                 currentpoint 40 sub exch 20 sub emojiwidth sub abs exch abs add = }
         def",
    );
    let out = {
        m.take_output();
        m.render_at(0.0).unwrap();
        m.take_output()
    };
    assert!(out.trim().parse::<f64>().unwrap() < 1e-6, "{out}");
    let (x0, y0, x1, y1, _) = ink_at(&mut m, 0.0).expect("it drew");
    // pixel rows from the top of the 120 high canvas: y = 40 is row 80
    assert!(x0 >= 20 && x1 <= 20 + 48, "x {x0}..{x1}");
    assert!(y1 <= 80 + 10 && y0 >= 80 - 40 - 6, "y {y0}..{y1}");
}

#[test]
fn an_unknown_emoji_is_a_range_error_not_a_blank() {
    let mut m = load("/scene { /nosuch 10 10 20 emojiat } def");
    let e = m.render_at(0.0).expect_err("an error").to_string();
    assert!(e.contains("rangecheck") || e.contains("emojiat"), "{e}");
}

// --- pkpixel -------------------------------------------------------------

#[test]
fn a_pixel_font_gives_the_columns_of_a_character_and_a_line() {
    // 'I' in the 7-row font is a top bar, a stem and a bottom bar, three columns wide;
    // bit r of a column is the dot in row r, counting from the top.
    let out = eval_at("", "7 pixelfont 73 pixelcols { == } forall", 0.0);
    assert_eq!(nums(&out), vec![65.0, 127.0, 65.0], "{out}");
    // A line is its characters side by side with one blank column between, none after.
    let out = eval_at(
        "",
        "7 pixelfont (II) pixeltext dup length == { == } forall",
        0.0,
    );
    assert_eq!(
        nums(&out),
        vec![7.0, 65.0, 127.0, 65.0, 0.0, 65.0, 127.0, 65.0],
        "{out}"
    );
    let out = eval_at("", "7 pixelfont () pixeltext length ==", 0.0);
    assert_eq!(nums(&out), vec![0.0], "{out}");
}

#[test]
fn a_pixel_font_falls_back_and_the_nine_row_font_has_descenders() {
    let widths = |font: &str, code: u32| {
        let out = eval_at(
            "",
            &format!("{font} pixelfont {code} pixelcols {{ == }} forall"),
            0.0,
        );
        nums(&out)
    };
    // The 5-row font has no lower case: 'a' is 'A', and an unknown character is '?'.
    assert_eq!(widths("5", 97), widths("5", 65));
    assert_eq!(widths("7", 1), widths("7", 63));
    assert_ne!(widths("7", 97), widths("7", 65));
    // The 9-row 'g' hangs below the baseline: a dot in row 7 or 8 (a bit of 128 or 256).
    let low = |v: Vec<f64>| v.iter().any(|m| (*m as u32) & 0x180 != 0);
    assert!(!low(widths("7", 103)));
    assert!(low(widths("9", 103)));
}

#[test]
fn a_pixel_picture_is_its_columns_and_a_bad_font_size_is_an_error() {
    let out = eval_at("", "[ (#.) (.#) (##) ] pixelpicture { == } forall", 0.0);
    assert_eq!(nums(&out), vec![5.0, 6.0], "{out}");
    let mut m = load("/scene { 6 pixelfont pop } def");
    let e = m.render_at(0.0).expect_err("an error").to_string();
    assert!(e.contains("rangecheck") || e.contains("pixelfont"), "{e}");
}
