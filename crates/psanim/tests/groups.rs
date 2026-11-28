//! PSCat's transparency groups (`begingroup` / `endgroup`) beside psanim's
//! own layers (`faded`, `blurred`, `glowing`, the motions, the pixel
//! effects), in pure, stateful and persistent movies.
//!
//! psanim draws a layer by swapping the interpreter's canvas. While a
//! group is open, PSCat paints on the group's own layer and a swap
//! exchanges the page *below* it, so a psanim layer opened inside a group
//! would lose its effect without a word. The rules these tests hold:
//!
//! - a group inside a psanim layer or motion works, and matches the same
//!   drawing without the group;
//! - `begingroup` and `endgroup` are PSCat's operators, untouched
//!   (`//begingroup` works);
//! - `faded` and `blended` (`/normal`, `/multiply`) inside a group run as
//!   groups of their own (`.groupfaded` in the prelude), and match the
//!   same fade outside it, with the layer's graphics-state rules: the
//!   caller's alpha and blend apply to the marks, and what the body
//!   changes, gsave levels included, outlasts it;
//! - every other layer or pixel effect inside a group is refused with
//!   `groupnesting`, which names the word;
//! - a body that leaves a group open, or closes one it did not open, is
//!   `groupnesting`; a frame that leaves one open is `unclosedgroup`;
//! - after an error or an interrupt with a group open, the next frame is
//!   the one a fresh movie draws;
//! - a parallel render of a movie with groups is the single-thread render.
//!
//! **Pending PSCatLabs/pscat#1494.** Whether a group is open is asked of
//! the interpreter in one place (`crates/psanim/src/groups.rs`), and PSCat
//! 0.8.0 cannot answer yet; until it can, psanim sees no group, and
//! behaves inside one as it did before groups existed. The tests that need
//! the answer are `#[ignore]`d with that reason; `.groupfaded`'s own rules
//! are tested now by calling it directly.
//!
//! Tolerances. psanim composites a layer at 8 bits a channel
//! (`raster::composite`); PSCat composites a group through its 16-bit
//! shadow and rounds once. The same fade done the two ways can therefore
//! differ by one level per channel where the fade is partial (measured:
//! at most 1 on this content, see `group_opacity_matches_faded`). Where
//! both sides use the same mechanism the frames must be identical.

use psanim::{Error, Movie, Options};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// One level per channel: the 8-bit layer composite against PSCat's
/// 16-bit group composite (see the module comment).
const TOL: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Pure,
    Stateful,
    Persistent,
}

const MODES: [Mode; 3] = [Mode::Pure, Mode::Stateful, Mode::Persistent];

/// Two overlapping antialiased discs, red under blue: where they overlap a
/// fade of the whole must show blue alone, at the fade, over the ground.
const SHAPES: &str = "1 0 0 setrgbcolor 26 24 15 0 360 arc fill \
                      0 0.4 1 setrgbcolor 40 24 15 0 360 arc fill";

/// A ground that is not uniform, painted by the scene itself so that a
/// persistent movie's frames are whole frames too.
const GROUND: &str = "0 8 56 { dup 16 mod 0 eq { 0.9 0.8 0.2 } { 0.1 0.3 0.25 } ifelse \
                      setrgbcolor 0 8 48 rectfill } for";

fn source(mode: Mode, setup: &str, scene: &str) -> String {
    let persist = if mode == Mode::Persistent {
        "/Persist true"
    } else {
        ""
    };
    let step = if mode == Mode::Stateful {
        "/ticks 0 def /step { /ticks ticks 1 add def } def"
    } else {
        ""
    };
    format!(
        "<< /Size [64 48] /Duration 2 /FPS 10 /Background [0 0 0] {persist} >> animation\n\
         {setup}\n{step}\n/scene {{ {GROUND} {scene} }} def"
    )
}

fn load(src: &str) -> Movie {
    Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}\n{src}"))
}

fn frame_of(m: &mut Movie, k: u64) -> Result<Vec<u8>, Box<Error>> {
    m.render_frame(k)
        .map(|f| f.premultiplied.to_vec())
        .map_err(Box::new)
}

fn frame(mode: Mode, setup: &str, scene: &str, k: u64) -> Vec<u8> {
    let src = source(mode, setup, scene);
    frame_of(&mut load(&src), k).unwrap_or_else(|e| panic!("{mode:?}: {e}\n{scene}"))
}

fn error(mode: Mode, setup: &str, scene: &str, k: u64) -> Error {
    let src = source(mode, setup, scene);
    match frame_of(&mut load(&src), k) {
        Ok(_) => panic!("{mode:?}: expected an error from\n{scene}"),
        Err(e) => *e,
    }
}

fn max_diff(a: &[u8], b: &[u8]) -> u8 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

fn pixel(f: &[u8], x: usize, y: usize) -> [u8; 4] {
    let o = (y * 64 + x) * 4;
    f[o..o + 4].try_into().unwrap()
}

/// Assert two scenes draw the same frame `k` within `tol`, and return the
/// largest difference seen.
fn same(mode: Mode, setup: &str, a: &str, b: &str, k: u64, tol: u8) -> u8 {
    let d = max_diff(&frame(mode, setup, a, k), &frame(mode, setup, b, k));
    assert!(
        d <= tol,
        "{mode:?}, frame {k}: differ by {d} (allowed {tol})\n  {a}\n  {b}"
    );
    d
}

fn grouped(dict: &str, body: &str) -> String {
    format!("{dict} begingroup {body} endgroup")
}

// --- a group inside a psanim layer ---------------------------------------------

#[test]
fn group_opacity_matches_faded() {
    let group = grouped("<< /Alpha 0.5 >>", SHAPES);
    let faded = format!("0.5 {{ {SHAPES} }} faded");
    for mode in MODES {
        for k in [0, 7] {
            let d = same(mode, "", &group, &faded, k, TOL);
            eprintln!("{mode:?} frame {k}: group opacity against faded: {d}");
        }
    }
    // The point of either: the overlap shows the blue disc alone at half
    // strength, where per-mark alpha lets the red show through.
    let g = frame(Mode::Pure, "", &group, 0);
    let marks = frame(Mode::Pure, "", &format!("0.5 setalpha {SHAPES}"), 0);
    let blue_alone = frame(
        Mode::Pure,
        "",
        &grouped(
            "<< /Alpha 0.5 >>",
            "0 0.4 1 setrgbcolor 40 24 15 0 360 arc fill",
        ),
        0,
    );
    // In the overlap, on a dark stripe.
    let (x, y) = (28, 24);
    assert_eq!(pixel(&g, x, y), pixel(&blue_alone, x, y));
    assert!(pixel(&marks, x, y)[0] > pixel(&g, x, y)[0] + 30);
}

#[test]
fn a_group_inside_a_layer_matches_the_layer_without_it() {
    let plain = grouped("<< >>", SHAPES);
    let half = grouped("<< /Alpha 0.5 >>", SHAPES);
    for mode in MODES {
        // Same mechanism on both sides: an opaque group adds nothing.
        same(
            mode,
            "",
            &format!("0.5 {{ {plain} }} faded"),
            &format!("0.5 {{ {SHAPES} }} faded"),
            3,
            0,
        );
        same(
            mode,
            "",
            &format!("4 {{ {plain} }} blurred"),
            &format!("4 {{ {SHAPES} }} blurred"),
            3,
            0,
        );
        same(
            mode,
            "",
            &format!("6 1 {{ {plain} }} glowing"),
            &format!("6 1 {{ {SHAPES} }} glowing"),
            3,
            0,
        );
        // The group's own opacity inside an opaque layer: the fade, done
        // by PSCat onto the layer rather than by psanim onto the page.
        same(
            mode,
            "",
            &format!("1 {{ {half} }} faded"),
            &format!("0.5 {{ {SHAPES} }} faded"),
            3,
            TOL,
        );
        same(
            mode,
            "",
            &format!("4 {{ {half} }} blurred"),
            &format!("4 {{ 0.5 {{ {SHAPES} }} faded }} blurred"),
            3,
            TOL,
        );
    }
}

// --- a psanim layer inside a group -------------------------------------------------

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn faded_and_blended_inside_a_group_run_as_groups() {
    for mode in MODES {
        // Before the fix this drew the discs at full strength: the layer
        // was swapped in below the group and never painted on.
        same(
            mode,
            "",
            &grouped("<< >>", &format!("0.5 {{ {SHAPES} }} faded")),
            &format!("0.5 {{ {SHAPES} }} faded"),
            3,
            TOL,
        );
        same(
            mode,
            "",
            &grouped("<< >>", &format!("0.5 {{ {SHAPES} }} faded")),
            &grouped("<< /Alpha 0.5 >>", SHAPES),
            3,
            TOL,
        );
        // A fade inside a faded group: the two multiply.
        same(
            mode,
            "",
            &grouped("<< /Alpha 0.5 >>", &format!("0.5 {{ {SHAPES} }} faded")),
            &format!("0.25 {{ {SHAPES} }} faded"),
            3,
            TOL,
        );
        // A blend inside a group blends with what the group holds (groups
        // are isolated), as one inside a psanim layer blends with the
        // layer: so the reference is the same blend inside an opaque layer.
        let under = "0.6 0.6 0.9 setrgbcolor 8 6 48 36 rectfill";
        for blend in ["normal", "multiply"] {
            let body = format!("{under} 0.7 /{blend} {{ {SHAPES} }} blended");
            same(
                mode,
                "",
                &grouped("<< >>", &body),
                &format!("1 {{ {body} }} faded"),
                3,
                TOL,
            );
        }
        // crossfade is built on faded.
        same(
            mode,
            "",
            &grouped(
                "<< >>",
                &format!(
                    "{{ {SHAPES} }} {{ 0 0 0 setrgbcolor 20 10 20 20 rectfill }} 0.5 crossfade"
                ),
            ),
            &format!("{{ {SHAPES} }} {{ 0 0 0 setrgbcolor 20 10 20 20 rectfill }} 0.5 crossfade"),
            3,
            TOL,
        );
    }
}

/// What a scene inside `<< >> begingroup ... endgroup` must be refused for,
/// and the word the error names.
const REFUSED: &[(&str, &str)] = &[
    ("4 { SHAPES } blurred", "`blurred`"),
    ("6 1 { SHAPES } glowing", "`glowing`"),
    ("1 /add { SHAPES } blended", "`blended`"),
    ("1 /screen { SHAPES } blended", "`blended`"),
    ("SHAPES 4 blurlayer", "`blurlayer`"),
    ("SHAPES 6 1 glowlayer", "`glowlayer`"),
    ("SHAPES 0.5 vignette", "`vignette`"),
    ("SHAPES 0.3 filmgrain", "`filmgrain`"),
    ("beginlayer SHAPES 1 endlayer", "`beginlayer`"),
];

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn other_layers_and_effects_inside_a_group_are_refused_and_the_next_frame_is_clean() {
    for (body, word) in REFUSED {
        let body = body.replace("SHAPES", SHAPES);
        // Wrong only at frame 5; the movie goes on.
        let scene = format!(
            "frame 5 eq {{ {} }} {{ 0.5 {{ {SHAPES} }} faded }} ifelse",
            grouped("<< >>", &body)
        );
        for mode in MODES {
            let src = source(mode, "", &scene);
            let mut m = load(&src);
            let e = frame_of(&mut m, 5).unwrap_err();
            assert_eq!(e.kind, "groupnesting", "{mode:?} {body}: {e}");
            assert!(e.message.contains(word), "{mode:?}: {}", e.message);
            assert!(e.message.contains("begingroup"), "{}", e.message);
            assert!(e.hint.as_deref().unwrap_or("").contains("endgroup"));
            // A frame before, after the failure, is the fresh movie's.
            let fresh = frame_of(&mut load(&src), 2).unwrap();
            assert_eq!(frame_of(&mut m, 2).unwrap(), fresh, "{mode:?} {body}");
            // A persistent movie cannot pass the failing tick; the others can.
            if mode != Mode::Persistent {
                let fresh = frame_of(&mut load(&src), 6).unwrap();
                assert_eq!(frame_of(&mut m, 6).unwrap(), fresh, "{mode:?} {body}");
            }
        }
    }
    // The same, written the right way round, runs.
    for (body, _) in REFUSED {
        let body = body.replace("SHAPES", &grouped("<< /Alpha 0.5 >>", SHAPES));
        frame(Mode::Pure, "", &body, 3);
    }
}

// --- a group across a motion -----------------------------------------------------

const MOTION: &str = "/pkmotion use";

#[test]
fn a_group_inside_a_motion_matches_the_motion_without_it() {
    let plain = grouped("<< >>", SHAPES);
    let half = grouped("<< /Alpha 0.5 >>", SHAPES);
    for mode in MODES {
        // Frame 0: absent; 7: arriving (a layer, placed); 19: at rest.
        for k in [0, 7, 19] {
            same(
                mode,
                MOTION,
                &format!("0.5 [/rise 0.8] {{ {plain} }} enters"),
                &format!("0.5 [/rise 0.8] {{ {SHAPES} }} enters"),
                k,
                0,
            );
            same(
                mode,
                MOTION,
                &format!("0.5 [/rise 0.8] {{ {half} }} enters"),
                &format!("0.5 [/rise 0.8] {{ 0.5 {{ {SHAPES} }} faded }} enters"),
                k,
                TOL,
            );
            same(
                mode,
                MOTION,
                &format!("0.6 /pulse {{ {half} }} emphasized"),
                &format!("0.6 /pulse {{ 0.5 {{ {SHAPES} }} faded }} emphasized"),
                k,
                TOL,
            );
        }
    }
}

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_motion_inside_a_group_is_refused_while_it_moves() {
    for mode in MODES {
        let enters = grouped(
            "<< /Alpha 0.5 >>",
            &format!("0.5 [/rise 0.8] {{ {SHAPES} }} enters"),
        );
        // Absent and at rest a motion draws directly, which is right. (A
        // persistent movie reaches frame 19 only through the moving
        // frames, which fail.)
        let rest: &[u64] = if mode == Mode::Persistent {
            &[0]
        } else {
            &[0, 19]
        };
        for &k in rest {
            let without = format!(
                "0.5 [/rise 0.8] {{ {} }} enters",
                grouped("<< /Alpha 0.5 >>", SHAPES)
            );
            same(mode, MOTION, &enters, &without, k, 0);
        }
        // Moving, it needs a layer, which inside the group would be lost.
        let e = error(mode, MOTION, &enters, 7);
        assert_eq!(e.kind, "groupnesting", "{e}");
        assert!(e.message.contains("motion"), "{}", e.message);
        let e = error(
            mode,
            MOTION,
            &grouped("<< >>", &format!("0.6 /pulse {{ {SHAPES} }} emphasized")),
            7,
        );
        assert_eq!(e.kind, "groupnesting", "{e}");
    }
}

// --- unbalanced groups ---------------------------------------------------------------

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_body_that_leaves_a_group_open_or_closes_one_it_did_not_open_is_an_error() {
    let open = format!("<< /Alpha 0.5 >> begingroup {SHAPES}");
    for mode in MODES {
        for scene in [
            format!("0.5 {{ {open} }} faded"),
            format!("4 {{ {open} }} blurred"),
            format!("6 1 {{ {open} }} glowing"),
            format!("beginlayer {open} 1 endlayer"),
            grouped("<< >>", &format!("0.5 {{ {open} }} faded")),
        ] {
            let e = error(mode, "", &scene, 3);
            assert_eq!(e.kind, "groupnesting", "{mode:?} {scene}: {e}");
            assert!(e.message.contains("not closed inside it"), "{}", e.message);
        }
        let e = error(
            mode,
            MOTION,
            &format!("0.5 [/rise 0.8] {{ {open} }} enters"),
            7,
        );
        assert_eq!(e.kind, "groupnesting", "{e}");
        // At rest the motion opens no layer: the group is the frame's.
        // (A persistent movie fails at the first moving frame on its way.)
        if mode != Mode::Persistent {
            let e = error(
                mode,
                MOTION,
                &format!("0.5 [/rise 0.8] {{ {open} }} enters"),
                19,
            );
            assert_eq!(e.kind, "unclosedgroup", "{e}");
        }
        // Inside a group, faded is a group, and its body's endgroup would
        // close that one.
        let e = error(
            mode,
            "",
            &grouped(
                "<< /Alpha 0.5 >>",
                &format!("0.5 {{ {SHAPES} endgroup }} faded"),
            ),
            3,
        );
        assert_eq!(e.kind, "groupnesting", "{e}");
        assert!(e.message.contains("did not open"), "{}", e.message);
        // An endgroup with no group open does nothing, in PSCat and here,
        // inside a layer or not.
        same(
            mode,
            "",
            &format!("0.5 {{ {SHAPES} endgroup }} faded"),
            &format!("0.5 {{ {SHAPES} }} faded"),
            3,
            0,
        );
        same(mode, "", &format!("endgroup {SHAPES}"), SHAPES, 3, 0);
    }
}

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_group_left_open_at_the_end_of_the_scene_is_an_error_and_the_next_frame_is_clean() {
    let scene = format!(
        "frame 5 eq {{ << /Alpha 0.5 >> begingroup {SHAPES} }} {{ 0.5 {{ {SHAPES} }} faded }} ifelse"
    );
    for mode in MODES {
        let src = source(mode, "", &scene);
        let mut m = load(&src);
        let e = frame_of(&mut m, 5).unwrap_err();
        assert_eq!(e.kind, "unclosedgroup", "{mode:?}: {e}");
        assert!(e.hint.as_deref().unwrap().contains("endgroup"));
        assert_eq!(
            frame_of(&mut m, 2).unwrap(),
            frame_of(&mut load(&src), 2).unwrap()
        );
        if mode != Mode::Persistent {
            assert_eq!(
                frame_of(&mut m, 6).unwrap(),
                frame_of(&mut load(&src), 6).unwrap()
            );
        }
    }
}

#[test]
fn a_frame_that_fails_with_a_group_open_leaves_the_next_frame_clean() {
    for inner in [
        grouped("<< /Alpha 0.5 >>", &format!("{SHAPES} nosuchword")),
        format!(
            "0.5 {{ {} }} faded",
            grouped("<< >>", &format!("{SHAPES} nosuchword"))
        ),
        grouped("<< >>", &format!("0.5 {{ {SHAPES} nosuchword }} faded")),
    ] {
        let scene = format!(
            "frame 5 eq {{ {inner} }} {{ {} }} ifelse",
            grouped("<< /Alpha 0.5 >>", SHAPES)
        );
        for mode in MODES {
            let src = source(mode, "", &scene);
            let mut m = load(&src);
            let e = frame_of(&mut m, 5).unwrap_err();
            assert_eq!(e.kind, "undefined", "{mode:?}: {e}");
            assert_eq!(
                frame_of(&mut m, 2).unwrap(),
                frame_of(&mut load(&src), 2).unwrap()
            );
            if mode != Mode::Persistent {
                assert_eq!(
                    frame_of(&mut m, 6).unwrap(),
                    frame_of(&mut load(&src), 6).unwrap()
                );
            }
        }
    }
}

#[test]
fn an_interrupted_frame_with_a_group_open_leaves_the_next_frame_clean() {
    for inner in [
        grouped("<< /Alpha 0.5 >>", &format!("{SHAPES} {{ }} loop")),
        format!(
            "0.5 {{ {} }} faded",
            grouped("<< >>", &format!("{SHAPES} {{ }} loop"))
        ),
    ] {
        let scene = format!(
            "frame 5 eq {{ {inner} }} {{ {} }} ifelse",
            grouped("<< /Alpha 0.5 >>", SHAPES)
        );
        for mode in MODES {
            let src = source(mode, "", &scene);
            let stop = Arc::new(AtomicBool::new(false));
            let opts = Options {
                interrupt: Some(stop.clone()),
                ..Options::default()
            };
            let mut m = Movie::load(&src, &opts).unwrap();
            let s = stop.clone();
            let t = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(40));
                s.store(true, Ordering::Relaxed);
            });
            let e = frame_of(&mut m, 5).unwrap_err();
            t.join().unwrap();
            assert_eq!(e.kind, "interrupt", "{mode:?}: {e}");
            stop.store(false, Ordering::Relaxed);
            assert_eq!(
                frame_of(&mut m, 2).unwrap(),
                frame_of(&mut load(&src), 2).unwrap()
            );
            if mode != Mode::Persistent {
                assert_eq!(
                    frame_of(&mut m, 6).unwrap(),
                    frame_of(&mut load(&src), 6).unwrap()
                );
            }
        }
    }
}

// --- many interpreters, components ------------------------------------------------------

#[test]
fn a_parallel_render_with_groups_is_the_single_thread_render() {
    let src = source(
        Mode::Pure,
        MOTION,
        &format!(
            "<< /Alpha now 2 div >> begingroup {SHAPES} 1 now sub {{ 0 0 0 setrgbcolor 20 10 24 20 rectfill }} faded endgroup \
             4 {{ {} }} blurred 0.4 [/pop 0.8] {{ {} }} enters",
            grouped("<< /Alpha 0.6 >>", "1 1 1 setrgbcolor 4 4 12 12 rectfill"),
            grouped(
                "<< /Alpha 0.7 /Blend /Multiply >>",
                "0 1 0 setrgbcolor 44 30 16 14 rectfill"
            )
        ),
    );
    let frames: Vec<u64> = (0..20).collect();
    let mut runs = Vec::new();
    for threads in [1, 4] {
        let mut out = vec![Vec::new(); frames.len()];
        psanim::render::render_frames(&src, &Options::default(), &frames, threads, |f| {
            out[f.index as usize] = f.premultiplied;
            Ok(())
        })
        .unwrap_or_else(|e| panic!("{e}"));
        runs.push(out);
    }
    assert!(runs[0].iter().all(|f| !f.is_empty()));
    assert_eq!(runs[0], runs[1]);
}

#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn groups_inside_a_component() {
    let comp = |scene: &str| {
        format!("/part << /Size [40 30] /Duration 2 /Scene {{ {scene} }} >> component")
    };
    let place = "/part << /Box [10 8 40 30] >> place";
    for mode in MODES {
        let a = frame(
            mode,
            &comp(&grouped("<< >>", &format!("0.5 {{ {SHAPES} }} faded"))),
            place,
            3,
        );
        let b = frame(mode, &comp(&format!("0.5 {{ {SHAPES} }} faded")), place, 3);
        assert!(max_diff(&a, &b) <= TOL, "{mode:?}");
        let e = error(
            mode,
            &comp(&grouped("<< >>", &format!("4 {{ {SHAPES} }} blurred"))),
            place,
            3,
        );
        assert_eq!(e.kind, "groupnesting", "{e}");
    }
}

/// A group that `erasepage` ends is closed, as far as the interpreter is
/// concerned, so the frame is fine. (The earlier version of this branch,
/// which counted `begingroup` / `endgroup` in the prelude, still counted it
/// open. Today this passes because psanim sees no group at all; with
/// pscat#1494 it passes because the interpreter's count is right.)
#[test]
fn a_group_ended_by_erasepage_is_not_counted_open() {
    frame(
        Mode::Pure,
        "",
        &format!("<< /Alpha 0.5 >> begingroup {SHAPES} erasepage {SHAPES}"),
        3,
    );
}

/// `.groupfaded`, the prelude procedure a `faded` or `blended` inside a
/// group runs as, follows a layer's rules: the caller's alpha and blend
/// mode apply to the marks inside, and what the body changes in the
/// graphics state outlasts it, gsave levels it leaves open included. It is
/// called directly here, inside a group, since psanim cannot yet see the
/// group to choose it (pscat#1494).
#[test]
fn faded_as_a_group_keeps_the_layers_graphics_state_rules() {
    // (before, alpha, blend, body, after)
    let cases: &[(&str, &str, &str, String, &str)] = &[
        ("0.5 setalpha", "0.5", "", SHAPES.to_string(), ""),
        ("/Multiply setblendmode", "0.5", "", SHAPES.to_string(), ""),
        (
            "",
            "0.5",
            "",
            format!("0 1 0 setrgbcolor 6 4 translate {SHAPES}"),
            "0 0 20 20 rectfill",
        ),
        (
            "",
            "0.5",
            "",
            "0.4 setalpha 10 10 moveto".to_string(),
            "30 0 rlineto 0 20 rlineto closepath fill",
        ),
        ("", "0.5", "", "0 0 30 48 rectclip".to_string(), SHAPES),
        (
            "",
            "0.7",
            "multiply",
            format!("5 0 translate {SHAPES}"),
            "0 0 20 20 rectfill",
        ),
        // Finding 3 of the review: a gsave the body leaves open is still
        // there after it, holding the state it saved.
        (
            "1 0 0 setrgbcolor",
            "0.5",
            "",
            format!("gsave 0 0 1 setrgbcolor {SHAPES}"),
            "grestore 0 0 20 20 rectfill",
        ),
        (
            "1 0 0 setrgbcolor",
            "0.5",
            "",
            "gsave 0 1 0 setrgbcolor 3 3 translate gsave 0 0 1 setrgbcolor 40 0 10 10 rectfill"
                .to_string(),
            "grestore 0 0 20 20 rectfill grestore 20 0 20 20 rectfill",
        ),
    ];
    for mode in MODES {
        for (before, alpha, blend, body, after) in cases {
            let dict = if blend.is_empty() {
                format!("<< /Alpha {alpha} >>")
            } else {
                format!("<< /Alpha {alpha} /Blend /Multiply >>")
            };
            let layer = if blend.is_empty() {
                format!("{alpha} {{ {body} }} faded")
            } else {
                format!("{alpha} /{blend} {{ {body} }} blended")
            };
            same(
                mode,
                "",
                &grouped(
                    "<< >>",
                    &format!("{before} {{ {body} }} {dict} .groupfaded {after}"),
                ),
                &format!("1 {{ {before} {layer} {after} }} faded"),
                3,
                TOL,
            );
        }
    }
}

/// The other way round cannot be had: a group's start is a floor
/// `grestore` stops at, so a body that runs `grestore` past where it began
/// is refused rather than drawn with the wrong state.
#[test]
fn a_grouped_fade_whose_body_grestores_past_its_start_is_refused() {
    for mode in MODES {
        let e = error(
            mode,
            "",
            &grouped(
                "<< >>",
                &format!("gsave {{ grestore {SHAPES} }} << /Alpha 0.5 >> .groupfaded grestore"),
            ),
            3,
        );
        assert_eq!(e.kind, "groupnesting", "{mode:?}: {e}");
        assert!(e.message.contains("grestore"), "{}", e.message);
    }
}

/// Finding 2 of the review: `begingroup` and `endgroup` are PSCat's own
/// operators, so `//begingroup` (the operator itself, bound at scan time)
/// runs a group, as it does in any PSCat program.
#[test]
fn begingroup_and_endgroup_are_pscats_operators() {
    assert!(!psanim::PRELUDE.contains("/begingroup"));
    assert!(!psanim::PRELUDE.contains("/endgroup"));
    assert!(psanim::api::lookup("begingroup").is_none());
    for mode in MODES {
        same(
            mode,
            "",
            &format!("<< /Alpha 0.5 >> //begingroup {SHAPES} //endgroup"),
            &grouped("<< /Alpha 0.5 >>", SHAPES),
            3,
            0,
        );
        same(
            mode,
            "",
            &format!("<< /Alpha 0.5 >> //begingroup {SHAPES} //endgroup"),
            &format!("0.5 {{ {SHAPES} }} faded"),
            3,
            TOL,
        );
    }
}

/// A pattern whose `PaintProc` says `endgroup` with no group of its own
/// (a no-op at the tile's level, in PSCat).
const PATTERN: &str = "/pat << /PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 8 8] \
    /XStep 8 /YStep 8 /PaintProc { pop endgroup 1 0 0 setrgbcolor 0 0 8 8 rectfill } >> \
    matrix makepattern def";

/// Finding 1 of the review: a pattern's `endgroup` closes nothing at the
/// page's level, so the page's group is still open and a `faded` after it
/// must still run as a group (the fade kept). With the prelude's counting
/// it was lost silently.
#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_patterns_endgroup_does_not_hide_the_pages_group() {
    for mode in MODES {
        same(
            mode,
            PATTERN,
            &grouped(
                "<< >>",
                &format!("pat setpattern 0 0 64 48 rectfill 0.5 {{ {SHAPES} }} faded"),
            ),
            &format!("pat setpattern 0 0 64 48 rectfill 0.5 {{ {SHAPES} }} faded"),
            3,
            TOL,
        );
    }
}

/// Finding 4 of the review: the body of a grouped `faded` that closes the
/// fade's group and opens another in its place leaves as many groups open
/// as it found, but not the same one. Telling them apart needs the group's
/// identity, not the count.
#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_body_that_swaps_the_fades_group_for_another_is_refused() {
    for mode in MODES {
        let e = error(
            mode,
            "",
            &grouped(
                "<< >>",
                &format!("0.5 {{ endgroup << /Alpha 0.1 >> begingroup {SHAPES} }} faded"),
            ),
            3,
        );
        assert_eq!(e.kind, "groupnesting", "{mode:?}: {e}");
    }
}

/// Finding 5 of the review: a hint an operator left for an error that a
/// `stopped` caught does not describe a later, different error.
#[test]
fn a_caught_errors_hint_does_not_describe_a_later_one() {
    for mode in MODES {
        let e = error(
            mode,
            "",
            &format!("{{ 1 /nosuchmode {{ {SHAPES} }} blended }} stopped pop nosuchword"),
            3,
        );
        assert_eq!(e.kind, "undefined", "{e}");
        let hint = e.hint.unwrap_or_default();
        assert!(!hint.contains("blend mode"), "{hint}");
        assert!(hint.contains("nosuchword"), "{hint}");
        // The hint still arrives when its error is the one that escapes.
        let e = error(
            mode,
            "",
            &format!("1 /nosuchmode {{ {SHAPES} }} blended"),
            3,
        );
        assert!(e.hint.unwrap_or_default().contains("blend mode"));
    }
}

/// Finding 5, as the review found it: the `groupnesting` a `stopped`
/// caught must not become the kind of the `undefined` that escapes.
#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_caught_groupnesting_does_not_describe_a_later_error() {
    let e = error(
        Mode::Pure,
        "",
        "<< >> begingroup { 2 { } blurred } stopped pop endgroup nosuchword",
        3,
    );
    assert_eq!(e.kind, "undefined", "{e}");
    assert!(!e.hint.unwrap_or_default().contains("blurred"));
}

/// A group opened through PSCat's operator by any route is a group: a
/// `faded` inside it runs as one. (The prelude's counting could not see
/// this one; the interpreter's will.)
#[test]
#[ignore = "needs PSCat to tell a host which transparency groups are open (PSCatLabs/pscat#1494); until then psanim cannot see a group"]
fn a_group_opened_through_systemdict_is_seen() {
    let direct = format!(
        "<< >> systemdict /begingroup get exec 0.5 {{ {SHAPES} }} faded systemdict /endgroup get exec"
    );
    same(
        Mode::Pure,
        "",
        &direct,
        &format!("0.5 {{ {SHAPES} }} faded"),
        3,
        TOL,
    );
}
