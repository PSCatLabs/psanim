//! pkink's stroke scheduler: a plan of strokes painted over time onto a
//! persistent canvas. What it promises: the last frame is the board
//! painted in one go, at any output rate; no stroke vanishes or fades for
//! a frame on its way from the overlay to the kept canvas; seeking gives
//! the same frames as playing; errors name the stroke and the field.

use psanim::{Movie, Options, Param};

/// Five translucent strokes on a light ground, every kind, in a plan that
/// overlaps: stroke 0 starts first and ends last, stroke 2 ends exactly on
/// a step's time (1.8 s, step 54 at 30), the soak's passes land between
/// other strokes' ends. Strokes 5 and 6 end off a step (0.61 s, kept on
/// step 19 at 0.633 s; 0.835 s, kept on step 26 at 0.867 s), so an output
/// frame at --fps 24 or 60 (0.625, 0.617) and at 25 (0.84) falls after the
/// end and before the keeping: a stroke the overlay dropped at its end
/// would vanish there. Dark media on a light ground: ink only darkens.
/// The paths are straight: a trimmed stroke of a curve or a corner is not
/// exactly the start of the whole one (`trimpath` flattens a curve it
/// cuts, and a cut near a corner ends square where the whole one joins),
/// which would test `trimpath`, not the plan.
const PLAN: &str = "%!PS
<< /Size [160 90] /Duration 3 /FPS 30 /Persist true /Background [0.96 0.94 0.90] >> animation
/pkink use
/oneshot /oneshot 0 param cvi 1 eq def
/tip /tip 0 param cvi 1 eq def
/plan [
  << /At 0.2 /For 2.0 /Path { 10 20 moveto 150 50 lineto }
     /Paint { 0 exch trimpath 0.2 0.3 0.6 setrgbcolor 0.45 setalpha 7 setlinewidth stroke } >>
  << /Kind /sweep /At 0.5 /For 0.9 /Across [6 154] /Ease /sine-inout
     /Paint { pop 0.7 0.3 0.2 setrgbcolor 0.3 setalpha 0 40 160 22 rectfill } >>
  << /At 1.0 /For 0.8 /Path { 20 10 moveto 140 80 lineto } /Seed 7
     /Paint { 0 exch trimpath 0.1 setgray rand 1000 mod 1e4 div 0.4 add setalpha 5 setlinewidth stroke } >>
  << /Kind /soak /At 1.1 /For 1.3 /Passes 5
     /Paint { 0.1 0.45 0.3 setrgbcolor 0.15 setalpha 80 45 3 -1 roll 30 mul 6 add 0 360 arc fill } >>
  << /At 1.5 /For 0.7 /Path { 30 85 moveto 130 5 lineto } /Reveal 9 /Ease { dup mul }
     /Paint { pop 0.25 setgray 0.4 setalpha 9 setlinewidth stroke } >>
  << /At 0.3 /Until 0.61 /Path { 20 60 moveto 70 60 lineto }
     /Paint { 0 exch trimpath 0.5 0.1 0.1 setrgbcolor 0.5 setalpha 4 setlinewidth stroke } >>
  << /At 0.6 /Until 0.835 /Path { 90 70 moveto 150 70 lineto }
     /Paint { 0 exch trimpath 0.5 0.1 0.1 setrgbcolor 0.5 setalpha 4 setlinewidth stroke } >>
] paintplan def
/scene { oneshot { frame 0 eq { plan paintall } if } { plan paintkept } ifelse } def
/overlay {
  oneshot not { plan paintlive } if
  tip { plan painttip { pop pop 0 0 1 setrgbcolor 3 0 360 arc fill } if } if
} def
";

fn movie(fps: Option<f64>, oneshot: bool) -> Movie {
    let mut params = std::collections::BTreeMap::new();
    if oneshot {
        params.insert("oneshot".to_string(), Param::Number(1.0));
    }
    Movie::load(
        PLAN,
        &Options {
            fps,
            params,
            ..Options::default()
        },
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn last(m: &mut Movie) -> Vec<u8> {
    let n = m.meta().frames;
    m.render_frame(n - 1)
        .unwrap_or_else(|e| panic!("{e}"))
        .premultiplied
        .to_vec()
}

/// Per pixel, the darkest channel's distance from white: how much ink.
fn darkness(px: &[u8]) -> Vec<i32> {
    px.chunks(4)
        .map(|p| 255 - i32::from(*p[..3].iter().min().unwrap_or(&255)))
        .collect()
}

#[test]
fn the_last_frame_is_the_board_painted_in_one_go_at_any_rate() {
    let board = last(&mut movie(None, true));
    for fps in [None, Some(24.0), Some(25.0), Some(60.0)] {
        let got = last(&mut movie(fps, false));
        let differ = got
            .chunks(4)
            .zip(board.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(differ, 0, "{differ} pixels differ at --fps {fps:?}");
    }
}

#[test]
fn no_stroke_fades_or_vanishes_between_frames_at_any_rate() {
    for fps in [None, Some(24.0), Some(25.0), Some(60.0)] {
        let mut m = movie(fps, false);
        let mut prev: Option<Vec<i32>> = None;
        for k in 0..m.meta().frames {
            let d = darkness(m.render_frame(k).unwrap().premultiplied);
            if let Some(p) = &prev {
                let lost = d.iter().zip(p).filter(|(a, b)| *a < *b).count();
                assert_eq!(
                    lost, 0,
                    "frame {k} at --fps {fps:?} lost ink on {lost} pixels"
                );
            }
            prev = Some(d);
        }
    }
}

#[test]
fn seeking_gives_the_frames_playing_gives() {
    let mut played = movie(Some(25.0), false);
    let frames: Vec<Vec<u8>> = (0..played.meta().frames)
        .map(|k| played.render_frame(k).unwrap().premultiplied.to_vec())
        .collect();
    let mut seek = movie(Some(25.0), false);
    for k in [60, 3, 44, 27, 74, 0, 45, 46, 41] {
        assert_eq!(
            seek.render_frame(k).unwrap().premultiplied,
            &frames[k as usize][..],
            "frame {k}"
        );
    }
    // And at a time between frames, as a live host asks.
    let mut a = movie(None, false);
    let mut b = movie(None, false);
    a.render_at(2.5).unwrap();
    let x = a.render_at(1.234).unwrap().premultiplied.to_vec();
    assert_eq!(b.render_at(1.234).unwrap().premultiplied, &x[..]);
}

#[test]
fn a_stroke_ending_on_a_step_is_kept_on_that_step_once() {
    // Stroke 2 ends at 1.8 s: step 54. Count the steps on which paintkept
    // lays it by running the plan's own selection in the scene.
    let src = PLAN.replace(
        "/scene {",
        "/scene { plan /UT get false .paintdue 2 copy lt { 1 sub 1 exch { plan /US get exch get 2 eq { (laid ) print stepnow == } if } for } { pop pop } ifelse ",
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.render_frame(89).unwrap();
    assert_eq!(m.take_output(), "laid 54\n");
}

#[test]
fn the_tip_follows_the_stroke_being_drawn() {
    let src = format!(
        "{PLAN}\n/overlay {{ plan painttip {{ 4 array astore == }} {{ (none) = }} ifelse }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.take_output();
    m.render_at(0.1).unwrap();
    assert_eq!(m.take_output(), "none\n");
    // At 0.95 the sweep (started 0.5) is the latest begun: its front, half
    // way up the canvas.
    m.render_at(0.95).unwrap();
    let out = m.take_output();
    assert!(out.contains(" 45.0 0 1]"), "{out}");
    // At 1.4 stroke 2 (a line from 20,10 to 140,80) is 0.5 done.
    m.render_at(1.4).unwrap();
    let out = m.take_output();
    let v: Vec<f64> = out
        .trim_matches(|c| c == '[' || c == ']' || c == '\n')
        .split(' ')
        .map(|x| x.parse().unwrap())
        .collect();
    assert!(
        (v[0] - 80.0).abs() < 1e-6 && (v[1] - 45.0).abs() < 1e-6 && v[3] == 2.0,
        "{out}"
    );
}

#[test]
fn procedures_run_with_no_dictionary_of_the_librarys_open() {
    let src = PLAN.replace(
        "/Paint { pop 0.25 setgray",
        "/Paint { countdictstack (depth ) print == pop 0.25 setgray",
    ) + "/overlay { countdictstack (overlay ) print == plan paintlive } def";
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.take_output();
    m.render_at(1.9).unwrap();
    let out = m.take_output();
    let mut lines = out.lines();
    let outer = lines.next().unwrap().trim_start_matches("overlay ");
    assert_eq!(
        lines.next(),
        Some(format!("depth {outer}").as_str()),
        "{out}"
    );
}

fn frame_err(m: &mut Movie, k: u64) -> psanim::Error {
    match m.render_frame(k) {
        Ok(_) => panic!("expected an error at frame {k}"),
        Err(e) => e,
    }
}

fn error_of(strokes: &str) -> String {
    let src = format!(
        "<< /Size [64 36] /Duration 2 /FPS 10 /Persist true >> animation /pkink use
         /plan [ {strokes} ] paintplan def /scene {{ plan paintkept }} def"
    );
    match Movie::load(&src, &Options::default()) {
        Ok(_) => panic!("expected an error for {strokes}"),
        Err(e) => format!("{e} {}", e.hint.clone().unwrap_or_default()),
    }
}

#[test]
fn errors_name_the_stroke_and_the_field() {
    let ok = "<< /At 0 /For 1 /Path { 0 0 moveto 9 9 lineto } /Paint { pop } >>";
    let cases = [
        (
            "<< /At 0 /For 0 /Path { } /Paint { pop } >>",
            "stroke 1: /For",
        ),
        (
            "<< /For 1 /Path { } /Paint { pop } >>",
            "stroke 1: /At is missing",
        ),
        (
            "<< /At 0 /For 1 /Paint { pop } >>",
            "stroke 1: /Path is missing",
        ),
        (
            "<< /At 0 /For 1 /Path { } /Paint 3 >>",
            "stroke 1: /Paint is 3",
        ),
        (
            "<< /Kind /wash /At 0 /For 1 /Paint { pop } >>",
            "stroke 1: /Kind is /wash",
        ),
        (
            "<< /Kind /sweep /At 0 /For 1 /Paint { pop } >>",
            "stroke 1: /Across",
        ),
        (
            "<< /Kind /soak /At 0 /For 1 /Passes 0 /Paint { pop } >>",
            "stroke 1: /Passes",
        ),
        (
            "<< /Kind /soak /At 0 /For 1 /Passes 3 /Ease /linear /Paint { pop } >>",
            "stroke 1: /Ease has no meaning",
        ),
        (
            "<< /At 0 /For 1 /Path { } /Paint { pop } /Revael 4 >>",
            "did you mean /Reveal?",
        ),
        (
            "<< /At 0 /For 1 /Path { } /Paint { pop } /Ease /wobbly >>",
            "stroke 1: /Ease",
        ),
        ("42", "stroke 1 is a number"),
        (
            "<< /At 0 /For 1 /Until 2 /Path { } /Paint { pop } >>",
            "stroke 1: /Until and /For are both given",
        ),
        (
            "<< /At 3 /Until 2 /Path { } /Paint { pop } >>",
            "stroke 1: /Until is 2",
        ),
    ];
    for (bad, want) in cases {
        let e = error_of(&format!("{ok} {bad}"));
        assert!(e.contains(want), "{bad}: {e}");
    }
    // A key of the author's own (small letter first) is kept.
    let src = "<< /Size [64 36] /Duration 2 /FPS 10 >> animation /pkink use
         /plan [ << /At 0 /For 1 /Path { } /Paint { pop } /colour /white >> ] paintplan def
         /scene { plan /Strokes get 0 get /colour get == } def";
    let mut m = Movie::load(src, &Options::default()).unwrap();
    m.render_frame(0).unwrap();
    assert_eq!(m.take_output(), "/white\n");
}

#[test]
fn a_procedure_that_leaves_the_stack_wrong_is_named() {
    let src = "<< /Size [64 36] /Duration 2 /FPS 10 /Persist true >> animation /pkink use
         /plan [ << /At 0 /For 0.5 /Path { 0 0 moveto 9 9 lineto } /Paint { pop } >>
                 << /At 0 /For 0.5 /Path { 0 0 moveto 9 9 lineto } /Paint { } >> ] paintplan def
         /scene { plan paintkept } def";
    let mut m = Movie::load(src, &Options::default()).unwrap();
    let e = frame_err(&mut m, 9);
    let text = format!("{e} {}", e.hint.clone().unwrap_or_default());
    assert!(
        text.contains("stroke 1: /Paint left 1 more operand"),
        "{text}"
    );
}

#[test]
fn paintkept_in_the_overlay_and_paintlive_in_the_scene_are_errors() {
    let head = "<< /Size [64 36] /Duration 2 /FPS 10 /Persist true >> animation /pkink use
         /plan [ << /At 0 /For 0.5 /Path { 0 0 moveto 9 9 lineto } /Paint { pop } >> ] paintplan def";
    let mut m = Movie::load(
        &format!("{head} /scene {{ }} def /overlay {{ plan paintkept }} def"),
        &Options::default(),
    )
    .unwrap();
    let e = frame_err(&mut m, 3);
    assert!(e.hint.unwrap_or_default().contains("call it in `scene`"));
    let mut m = Movie::load(
        &format!("{head} /scene {{ plan paintlive }} def"),
        &Options::default(),
    )
    .unwrap();
    let e = frame_err(&mut m, 3);
    assert!(e.hint.unwrap_or_default().contains("call it in `overlay`"));
}

#[test]
fn in_a_pure_movie_the_same_calls_draw_the_board_so_far() {
    // Nothing is kept, so paintkept draws every stroke finished by now.
    let src = PLAN.replace("/Persist true", "");
    let src = src.replace(
        "/overlay {",
        "/scene { plan paintkept plan paintlive } def /unused {",
    );
    let mut pure = Movie::load(&src, &Options::default()).unwrap();
    let mut kept = movie(None, false);
    for t in [0.0, 0.9, 1.8, 2.4, 2.99] {
        let a = pure.render_at(t).unwrap().premultiplied.to_vec();
        let b = kept.render_at(t).unwrap().premultiplied.to_vec();
        let differ = a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count();
        assert_eq!(differ, 0, "at {t}");
    }
}

/// Real chalk, grown with `/From 0 /To u`: on a dark board ink lightens.
/// The first stroke ends off a step (0.61 s), between output frames of
/// --fps 24 and 60 and the step that keeps it.
const CHALK: &str = "%!PS
<< /Size [160 90] /Duration 1.4 /FPS 30 /Persist true /Background [0.17 0.25 0.21] >> animation
(lib/artkit.ps) run (lib/paintkit.ps) run (lib/drykit.ps) run (lib/drychalk.ps) run
/pkink use
/oneshot /oneshot 0 param cvi 1 eq def
<< /Surface /greenboard >> dksurface
/chalk { /ch-u exch def /white dcchalkcolor << /Width 3 /Hand 0.6 /From 0 /To ch-u >> dcchalk } def
/plan [
  << /At 0.1 /Until 0.61 /Path { 10 20 moveto 150 40 lineto } /Paint /chalk load >>
  << /At 0.4 /Until 0.9 /Path { 20 80 moveto 60 10 100 80 140 15 curveto } /Paint /chalk load /Ease /sine-inout >>
  << /At 0.95 /For 0.35 /Path { 80 45 30 0 300 arc } /Paint /chalk load >>
] paintplan def
/scene { oneshot { frame 0 eq { plan paintall } if } { plan paintkept } ifelse } def
/overlay { oneshot not { plan paintlive } if } def
";

fn chalk(fps: Option<f64>, oneshot: bool) -> Movie {
    let mut params = std::collections::BTreeMap::new();
    if oneshot {
        params.insert("oneshot".to_string(), Param::Number(1.0));
    }
    Movie::load(
        CHALK,
        &Options {
            fps,
            params,
            ..Options::default()
        },
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn grown_chalk_is_exact_and_never_vanishes_at_any_rate() {
    let board = last(&mut chalk(None, true));
    // Light chalk on a dark board: brightness only grows.
    let light = |px: &[u8]| -> Vec<i32> {
        px.chunks(4)
            .map(|p| p[..3].iter().map(|c| i32::from(*c)).sum())
            .collect()
    };
    for fps in [None, Some(24.0), Some(60.0)] {
        let mut m = chalk(fps, false);
        let n = m.meta().frames;
        let mut ground: Vec<i32> = Vec::new();
        let mut before = 0i64;
        for k in 0..n {
            let px = m.render_frame(k).unwrap().premultiplied.to_vec();
            let l = light(&px);
            if k == 0 {
                ground = l.clone(); // nothing is drawn until 0.1 s
            }
            // The chalk on the board: what each pixel has above the ground.
            let ink: i64 = l
                .iter()
                .zip(&ground)
                .map(|(a, g)| i64::from((a - g).max(0)))
                .sum();
            // A grown stroke moves only the last widths behind its tip, so
            // the whole board's chalk may wobble by texture noise; a stroke
            // that vanished for a frame would take thousands.
            assert!(
                ink * 1000 >= before * 995,
                "frame {k} at --fps {fps:?}: the chalk went from {before} to {ink}"
            );
            before = ink;
            if k == n - 1 {
                let differ = px
                    .chunks(4)
                    .zip(board.chunks(4))
                    .filter(|(a, b)| a != b)
                    .count();
                assert_eq!(
                    differ, 0,
                    "{differ} pixels differ from the board at --fps {fps:?}"
                );
            }
        }
    }
}

/// One plan, drawn as a pure scene: three translucent strokes over 2 s.
const PIECE_SETUP: &str = "/pkink use
  /line { 0 exch trimpath 0.2 0.3 0.6 setrgbcolor 0.5 setalpha 4 setlinewidth stroke } def
  /plan [
    << /At 0.1 /For 0.8 /Path { 4 6 moveto 60 10 lineto } /Paint /line load >>
    << /At 0.6 /Until 1.45 /Path { 6 30 moveto 58 4 lineto } /Paint /line load >>
    << /Kind /sweep /At 1.2 /For 0.7 /Across [2 62] /Paint { pop 0.7 0.3 0.2 setrgbcolor 0.4 setalpha 0 14 64 8 rectfill } >>
  ] paintplan def";
const PIECE_SCENE: &str = "plan paintkept plan paintlive";

fn piece_alone() -> Movie {
    let src = format!(
        "<< /Size [64 36] /Duration 2 /FPS 30 /Background [1 1 1] >> animation
         {PIECE_SETUP} /scene {{ {PIECE_SCENE} }} def"
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

fn piece_placed(opts: &str) -> Movie {
    let src = format!(
        "<< /Size [64 36] /Duration 12 /FPS 30 /Background [1 1 1] >> animation
         /art << /Size [64 36] /Duration 2 /Setup {{ {PIECE_SETUP} }} /Scene {{ {PIECE_SCENE} }} >> component
         /scene {{ /art {opts} place }} def"
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_plan_in_a_component_runs_on_the_components_own_clock() {
    let mut alone = piece_alone();
    // (placement, start, speed, whole laps before): local time t is shown
    // at movie time start + laps * 2 / speed + t / speed.
    let cases = [
        ("<< /At 3 >>", 3.0, 1.0, 0.0),              // delayed
        ("<< /At 1 /Speed 2 >>", 1.0, 2.0, 0.0),     // sped up
        ("<< /At 3 /After /loop >>", 3.0, 1.0, 2.0), // its third time round
    ];
    for (opts, start, speed, laps) in cases {
        let at = |t: f64| start + (laps * 2.0 + t) / speed;
        let mut placed = piece_placed(opts);
        for t in [0.3, 0.7, 1.0, 1.3, 1.6, 1.9] {
            let want = alone.render_at(t).unwrap().premultiplied.to_vec();
            let got = placed.render_at(at(t)).unwrap().premultiplied.to_vec();
            let differ = want
                .chunks(4)
                .zip(got.chunks(4))
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(differ, 0, "{opts} at local {t}: {differ} pixels differ");
        }
    }
}

#[test]
fn a_component_counts_its_own_steps_and_tip() {
    // stepnow and painttip inside a component read its own clock.
    let src = format!(
        "<< /Size [64 36] /Duration 12 /FPS 30 >> animation
         /art << /Size [64 36] /Duration 2 /Setup {{ {PIECE_SETUP} }}
            /Scene {{ (step ) print stepnow =only ( ) print plan painttip {{ 4 array astore == }} {{ (none) = }} ifelse }} >> component
         /scene {{ /art << /At 3 >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.take_output();
    m.render_at(3.5).unwrap();
    let out = m.take_output();
    // local 0.5 s: step 15, stroke 0 half way (0.4 of 0.8 s) along 4,6 -> 60,10
    assert!(out.starts_with("step 15 [32.0"), "{out}");
    assert!(out.trim_end().ends_with(" 0]"), "{out}");
}

#[test]
fn a_plan_in_a_component_drawn_onto_a_kept_canvas_is_an_error() {
    let src = format!(
        "<< /Size [64 36] /Duration 4 /FPS 30 /Persist true >> animation
         /art << /Size [64 36] /Duration 2 /Setup {{ {PIECE_SETUP} }} /Scene {{ {PIECE_SCENE} }} >> component
         /scene {{ /art << >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    let e = frame_err(&mut m, 10);
    assert!(
        e.hint
            .unwrap_or_default()
            .contains("a component has no kept canvas"),
        "wrong error"
    );
    // In the overlay it is fine.
    let src = src.replace("/scene {", "/scene { } def /overlay {");
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.render_at(1.0).unwrap();
}

/// Evening Harbour's last frame against its board painted in one go
/// (`--param oneshot=1`), at 30 fps and at `--fps` 24 and 60. A whole
/// render each: about a minute, so it is run on request:
/// `cargo test --release -p psanim --test paintplan -- --ignored`.
#[test]
#[ignore]
fn evening_harbour_ends_on_the_board_painted_in_one_go() {
    let src = include_str!("../../../gallery/evening-harbour/evening-harbour.ps");
    let load = |fps: Option<f64>, oneshot: bool| {
        let mut params = std::collections::BTreeMap::new();
        if oneshot {
            params.insert("oneshot".to_string(), Param::Number(1.0));
        }
        Movie::load(
            src,
            &Options {
                fps,
                params,
                ..Options::default()
            },
        )
        .unwrap_or_else(|e| panic!("{e}"))
    };
    let board = last(&mut load(None, true));
    for fps in [None, Some(24.0), Some(60.0)] {
        let mut m = load(fps, false);
        // In order, as a render does.
        for k in 0..m.meta().frames - 1 {
            m.render_frame(k).unwrap();
        }
        let got = last(&mut m);
        let differ = got
            .chunks(4)
            .zip(board.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(differ, 0, "{differ} pixels differ at --fps {fps:?}");
    }
}
