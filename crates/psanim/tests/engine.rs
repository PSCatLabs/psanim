//! End-to-end tests of the engine: PostScript in, pixels and metadata out.

use psanim::{Error, Mode, MotionBlur, Movie, Options, Param, Phase, Scale};

const HEAD: &str = "<< /Size [64 48] /Duration 2 /FPS 10 /Background [0 0 0] >> animation\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

fn load_err(source: &str) -> Error {
    match Movie::load(source, &Options::default()) {
        Ok(_) => panic!("expected an error"),
        Err(e) => e,
    }
}

/// Run `expr` (which leaves one number) inside a scene at frame `k` and
/// return the number, by drawing it: the value is written into the red
/// channel of the whole frame as value/scale.
fn eval_at(setup: &str, expr: &str, k: u64) -> f64 {
    let src = format!(
        "{HEAD}{setup}\n/scene {{ {expr} 1000 div 0.5 add dup 0 lt {{ pop 0 }} if dup 1 gt {{ pop 1 }} if
          0 0 setrgbcolor 0 0 64 48 rectfill }} def"
    );
    // 8 bits of red is too coarse; print the value instead.
    let _ = src;
    let src =
        format!("{HEAD}{setup}\n/scene {{ /psanim.probe {expr} store }} def /psanim.probe 0 def");
    // `store` into a setup-defined name would be rolled back with the frame,
    // so read it out through the canvas instead: encode in three channels.
    let _ = src;
    let src = format!(
        "{HEAD}{setup}
         /emit {{ % v -> paints v as 24-bit fixed point (v in -8388..8388, 1/1000 units)
           1000 mul round cvi 8388608 add
           dup 65536 idiv 255 div exch dup 256 idiv 256 mod 255 div exch 256 mod 255 div
           setrgbcolor 0 0 64 48 rectfill }} def
         /scene {{ {expr} emit }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    let f = m.render_frame(k).unwrap_or_else(|e| panic!("{e}"));
    let p = &f.premultiplied[..3];
    let raw = (i64::from(p[0]) << 16) | (i64::from(p[1]) << 8) | i64::from(p[2]);
    (raw - 8_388_608) as f64 / 1000.0
}

fn px(frame: &psanim::Frame, x: u32, y: u32) -> [u8; 4] {
    let o = ((y * frame.width + x) * 4) as usize;
    frame.premultiplied[o..o + 4].try_into().unwrap()
}

#[test]
fn declaration_becomes_metadata() {
    let m = Movie::load(
        "<< /Size [320 180] /Duration 2.5 /FPS 24 /Seed 9 /Title (Dawn) /Loop true
            /Background [1 0.5 0] >> animation
         /scene { } def",
        &Options::default(),
    )
    .unwrap();
    let meta = m.meta();
    assert_eq!((meta.width, meta.height), (320, 180));
    assert_eq!((meta.pixel_width, meta.pixel_height), (320, 180));
    assert_eq!(meta.frames, 60);
    assert_eq!(meta.seed, 9);
    assert_eq!(meta.title.as_deref(), Some("Dawn"));
    assert!(meta.looping);
    assert_eq!(meta.mode, Mode::Pure);
    assert_eq!(meta.background, Some([1.0, 0.5, 0.0]));
    assert_eq!(meta.to_json()["frames"], 60);
    assert_eq!(meta.time_of(12), 0.5);
    assert_eq!(meta.frame_at(0.51), 12);
    assert_eq!(meta.frame_at(99.0), 59);
}

#[test]
fn options_override_the_declaration() {
    let opts = Options {
        scale: Scale::Width(32),
        fps: Some(5.0),
        duration: Some(1.0),
        seed: Some(77),
        ..Options::default()
    };
    let mut m = Movie::load(&format!("{HEAD}/scene {{ }} def"), &opts).unwrap();
    let meta = m.meta().clone();
    assert_eq!(
        (meta.pixel_width, meta.pixel_height, meta.scale),
        (32, 24, 0.5)
    );
    assert_eq!((meta.frames, meta.seed), (5, 77));
    let f = m.render_frame(0).unwrap();
    assert_eq!((f.width, f.height), (32, 24));
    assert_eq!(f.premultiplied.len(), 32 * 24 * 4);
}

#[test]
fn a_program_without_a_declaration_gets_the_defaults() {
    let m = Movie::load(
        "/scene { } def",
        &Options {
            scale: Scale::Factor(0.1),
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(
        (
            m.meta().width,
            m.meta().height,
            m.meta().fps,
            m.meta().duration
        ),
        (1280, 720, 30.0, 5.0)
    );
}

#[test]
fn background_is_painted_and_transparent_is_transparent() {
    let mut m = Movie::load(
        "<< /Size [8 8] /Background [1 0 0] >> animation /scene { } def",
        &Options::default(),
    )
    .unwrap();
    assert_eq!(px(&m.render_frame(0).unwrap(), 3, 3), [255, 0, 0, 255]);
    let mut m = Movie::load(
        "<< /Size [8 8] /Background /transparent >> animation
         /scene { 0 1 0 setrgbcolor 0 0 4 8 rectfill } def",
        &Options::default(),
    )
    .unwrap();
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 1, 1), [0, 255, 0, 255]);
    assert_eq!(px(&f, 6, 1), [0, 0, 0, 0]);
    // PNG keeps the alpha.
    assert_eq!(&f.to_png()[1..4], b"PNG");
    assert_eq!(f.to_rgba8()[..4], [0, 255, 0, 255]);
}

#[test]
fn frames_are_time_addressable() {
    let mut m = load(
        "/n 0 def
         /scene { /n n 1 add def   % a mutation every frame: rolled back
           32 24 translate now 180 mul rotate 1 1 1 setrgbcolor -20 -3 40 6 rectfill
           rand 1000 mod 100 div 0 moveto 2 2 rlineto stroke } def",
    );
    let in_order: Vec<Vec<u8>> = (0..20)
        .map(|k| m.render_frame(k).unwrap().premultiplied.to_vec())
        .collect();
    assert_ne!(in_order[0], in_order[1]);
    for k in [19, 0, 7, 7, 3, 12] {
        assert_eq!(
            m.render_frame(k).unwrap().premultiplied,
            &in_order[k as usize][..],
            "frame {k}"
        );
    }
    // A second movie from the same source agrees: nothing depends on the
    // process or on what was rendered before.
    let mut again = load(
        "/n 0 def
         /scene { /n n 1 add def
           32 24 translate now 180 mul rotate 1 1 1 setrgbcolor -20 -3 40 6 rectfill
           rand 1000 mod 100 div 0 moveto 2 2 rlineto stroke } def",
    );
    assert_eq!(
        again.render_frame(13).unwrap().premultiplied,
        &in_order[13][..]
    );
}

#[test]
fn scene_gets_the_time_on_the_stack_and_from_now() {
    assert_eq!(eval_at("", "now", 5), 0.5);
    assert_eq!(eval_at("", "now", 5), 0.5);
    assert_eq!(eval_at("", "globaltime", 15), 1.5);
    assert_eq!(eval_at("", "frame", 7), 7.0);
    assert_eq!(eval_at("", "progress", 5), 0.25);
    assert_eq!(
        eval_at("", "duration fps add dt add width add height add", 0),
        2.0 + 10.0 + 0.1 + 64.0 + 48.0
    );
    assert_eq!(eval_at("", "span", 0), 2.0);
}

#[test]
fn a_scene_must_leave_the_stack_empty() {
    // An operand nothing used is reported, with its value: it is usually
    // the trace of a mistake that would otherwise draw the wrong thing.
    let mut m = load("/scene { 7 1 setgray 0 0 64 48 rectfill } def");
    let e = m.render_frame(3).unwrap_err();
    assert_eq!(e.kind, "stackleft");
    assert_eq!(
        e.phase,
        Phase::Frame {
            index: 3,
            time: 0.3
        }
    );
    assert!(e.message.contains("`scene` left 1 operand"), "{e}");
    assert!(e.hint.unwrap().contains("`now`"));
    // One operand short is now an error too, where the old contract
    // (the time passed as an operand) let it eat the engine's value.
    let mut m = load("/scene { 0.5 0.25 between pop } def");
    assert_eq!(m.render_frame(0).unwrap_err().kind, "stackunderflow");
    // The movie is still usable.
    let mut ok = load("/scene { 1 setgray 0 0 64 48 rectfill } def");
    assert!(ok.render_frame(0).is_ok());
    // The same for a step.
    let mut m = Movie::load(
        "<< /Size [8 8] /Duration 1 /FPS 10 >> animation /step { 42 } def /scene { } def",
        &Options::default(),
    )
    .unwrap();
    let e = m.render_frame(2).unwrap_err();
    assert_eq!(
        (e.kind.as_str(), e.stack.as_slice()),
        ("stackleft", &["42".to_string()][..])
    );
    assert!(matches!(e.phase, Phase::Step { tick: 0, .. }));
}

#[test]
fn what_a_program_prints_is_collected() {
    let mut m = load("(setup says hi) = /scene { frame 2 eq { now == (x) print } if } def");
    assert_eq!(m.take_output(), "setup says hi\n");
    m.render_frame(1).unwrap();
    assert_eq!(m.take_output(), "");
    m.render_frame(2).unwrap();
    assert_eq!(m.take_output(), "0.2\nx");
    assert_eq!(m.take_output(), "");
}

#[test]
fn printed_output_cut_at_64_kb_says_so() {
    // 3000 lines of 41 bytes: about 120 KB, twice what is kept.
    let mut m =
        load("/scene { 1 1 3000 { pop (0123456789012345678901234567890123456789) = } for } def");
    assert_eq!(
        m.take_printed(),
        psanim::Printed::default(),
        "setup printed nothing"
    );
    m.render_frame(0).unwrap();
    let p = m.take_printed();
    assert!(p.truncated);
    assert!(p.text.ends_with("[output truncated at 64 KB]\n"));
    let marker = "\n[output truncated at 64 KB]\n".len();
    assert!(
        p.text.len() <= 64 * 1024 + marker,
        "{} bytes kept",
        p.text.len()
    );
    assert!(p.text.len() > 64 * 1024 - 64, "most of the 64 KB is kept");
    // Each collection starts fresh: a short print afterwards is whole.
    m.render_frame(1).unwrap();
    let p = m.take_printed();
    assert!(p.truncated, "the second frame prints as much again");
    let mut quiet = load("/scene { frame 1 eq { (short) = } if } def");
    quiet.render_frame(0).unwrap();
    assert!(!quiet.take_printed().truncated);
    quiet.render_frame(1).unwrap();
    assert_eq!(
        quiet.take_printed(),
        psanim::Printed {
            text: "short\n".into(),
            truncated: false
        }
    );
}

#[test]
fn printed_output_under_the_limit_is_whole() {
    let mut m =
        load("/scene { 1 1 1000 { pop (0123456789012345678901234567890123456789) = } for } def");
    m.render_frame(0).unwrap();
    let p = m.take_printed();
    assert!(!p.truncated);
    assert_eq!(p.text.len(), 1000 * 41);
    assert!(!p.text.contains("truncated"));
}

#[test]
fn printed_output_in_setup_is_cut_too() {
    let mut m =
        load("1 1 3000 { pop (0123456789012345678901234567890123456789) = } for /scene { } def");
    let p = m.take_printed();
    assert!(p.truncated);
    assert!(p.text.ends_with("[output truncated at 64 KB]\n"));
    assert!(p.text.len() <= 64 * 1024 + 40);
    // `take_output` gives the same text.
    let mut m2 =
        load("1 1 3000 { pop (0123456789012345678901234567890123456789) = } for /scene { } def");
    assert_eq!(m2.take_output(), p.text);
}

/// A clock that advances 0.1 s each time it is read: whatever it times
/// looks like it took 0.1 s.
fn slow_clock() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TICKS: AtomicU64 = AtomicU64::new(0);
    TICKS.fetch_add(1, Ordering::SeqCst) as f64 * 0.1
}

/// A clock that advances a millisecond each time it is read.
fn fast_clock() -> f64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static TICKS: AtomicU64 = AtomicU64::new(0);
    TICKS.fetch_add(1, Ordering::SeqCst) as f64 * 0.001
}

fn meta_with_clock(source: &str, clock: Option<fn() -> f64>) -> psanim::Meta {
    let options = Options {
        clock,
        ..Options::default()
    };
    Movie::load(source, &options)
        .unwrap_or_else(|e| panic!("{e}"))
        .meta()
        .clone()
}

#[test]
fn a_slow_preamble_is_measured_and_warned_about() {
    let src = format!("/helper {{ 1 add }} def\n{HEAD}/scene {{ }} def");
    let slow = meta_with_clock(&src, Some(slow_clock));
    assert!(slow.preamble_seconds.unwrap() > psanim::PREAMBLE_WARN_SECONDS);
    assert!(!slow.preamble_printed);
    let w = slow.preamble_warning().expect("a warning");
    assert!(
        w.contains("before `animation`") && w.contains("runs twice"),
        "{w}"
    );
    assert!(w.contains("after `animation`"), "{w}");
    // The meta JSON carries both facts.
    assert!(slow.to_json()["preamble_ms"].as_f64().unwrap() > 50.0);
    assert_eq!(slow.to_json()["preamble_printed"], false);
}

#[test]
fn a_quick_quiet_preamble_is_not_warned_about() {
    let src = format!("/helper {{ 1 add }} def\n{HEAD}/scene {{ }} def");
    let m = meta_with_clock(&src, Some(fast_clock));
    assert!(m.preamble_seconds.unwrap() < psanim::PREAMBLE_WARN_SECONDS);
    assert_eq!(m.preamble_warning(), None);
    // Without a clock nothing is measured, and nothing is warned about.
    let m = meta_with_clock(&src, None);
    assert_eq!(m.preamble_seconds, None);
    assert_eq!(m.preamble_warning(), None);
    // A program with no `animation` has no preamble.
    let m = meta_with_clock("/scene { } def", Some(slow_clock));
    assert_eq!(m.preamble_seconds, None);
    assert_eq!(m.preamble_warning(), None);
    // Code after `animation` is not the preamble, however slow.
    let m = meta_with_clock(
        &format!("{HEAD}1 1 100000 {{ pop }} for /scene {{ }} def"),
        None,
    );
    assert_eq!(m.preamble_warning(), None);
}

#[test]
fn a_printing_preamble_is_warned_about_and_printed_once() {
    let src = format!("(preamble) =\n{HEAD}(setup) =\n/scene {{ }} def");
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    assert!(m.meta().preamble_printed);
    let w = m.meta().preamble_warning().expect("a warning");
    assert!(w.contains("prints") && w.contains("runs twice"), "{w}");
    // The probe pass printed it too; only the real pass's output is shown.
    assert_eq!(m.take_output(), "preamble\nsetup\n");
    // Printing after `animation` alone is no reason to warn.
    let src = format!("{HEAD}(setup) =\n/scene {{ }} def");
    let m = Movie::load(&src, &Options::default()).unwrap();
    assert!(!m.meta().preamble_printed);
    assert_eq!(m.meta().preamble_warning(), None);
}

#[test]
fn globally_reads_movie_time_inside_a_scope() {
    assert_eq!(
        eval_at("", "-1 0.5 2 { pop { now } globally } during", 9),
        0.9
    );
    assert_eq!(
        eval_at(
            "",
            "-1 0.5 2 { pop { 0 2 /linear tween } globally } during",
            10
        ),
        0.5
    );
    // ...and the scope it was called from is back afterwards.
    assert_eq!(
        eval_at("", "-1 0.5 2 { pop { } globally now } during", 9),
        0.4
    );
}

#[test]
fn spaced_and_path_text() {
    let font = "/Helvetica findfont 10 scalefont setfont";
    // spacedwidth = natural width + track * (n - 1)
    assert_eq!(
        eval_at(
            "",
            &format!("{font} (ABCD) 3 spacedwidth (ABCD) stringwidth pop sub"),
            0
        ),
        9.0
    );
    assert_eq!(
        eval_at(
            "",
            &format!("{font} (A) 3 spacedwidth (A) stringwidth pop sub"),
            0
        ),
        0.0
    );
    // eachspaced: track 0 is eachglyph; a positive track spreads the origins.
    let mut plain = load(&format!(
        "/scene {{ {font} 1 setgray 4 20 moveto (WAVE) {{ pop show }} eachglyph }} def"
    ));
    let mut zero = load(&format!(
        "/scene {{ {font} 1 setgray 4 20 moveto (WAVE) 0 {{ pop show }} eachspaced }} def"
    ));
    let a = plain.render_frame(0).unwrap().premultiplied.to_vec();
    assert_eq!(zero.render_frame(0).unwrap().premultiplied, &a[..]);
    let spread = "(WAVE) 4 { exch pop 3 eq { currentpoint pop } if } eachspaced";
    let natural = "(WAVE) 0 { exch pop 3 eq { currentpoint pop } if } eachspaced";
    let d = eval_at("", &format!("{font} 4 20 moveto {spread}"), 0)
        - eval_at("", &format!("{font} 4 20 moveto {natural}"), 0);
    assert!((d - 12.0).abs() < 0.01, "{d}");
    // eachonpath: along a horizontal line it is plain text, shifted by u0.
    let mut on = load(&format!(
        "/scene {{ {font} 1 setgray newpath 4 20 moveto 60 20 lineto (WAVE) 0 {{ pop show }} eachonpath }} def"
    ));
    let b = on.render_frame(0).unwrap().premultiplied.to_vec();
    let lit = |buf: &[u8]| buf.chunks(4).filter(|p| p[0] > 128).count() as f64;
    assert!(
        (lit(&a) - lit(&b)).abs() / lit(&a) < 0.05,
        "{} vs {}",
        lit(&a),
        lit(&b)
    );
    // On a vertical line the text runs upward: tall, not wide.
    let mut up = load(&format!(
        "/scene {{ {font} 1 setgray newpath 30 4 moveto 30 44 lineto (WAVE) 0 {{ pop show }} eachonpath
                  pathlength 40 ne {{ 1 }} if }} def"
    ));
    let f = up.render_frame(0).unwrap().premultiplied.to_vec();
    let cols = (0..64)
        .filter(|x| (0..48).any(|y| f[(y * 64 + x) * 4] > 128))
        .count();
    let rows = (0..48)
        .filter(|y| (0..64).any(|x| f[(y * 64 + x) * 4] > 128))
        .count();
    assert!(rows > 2 * cols, "{rows} rows, {cols} cols");
}

#[test]
fn sequence_gap_and_wiggle_array_key() {
    let seq = "-1 [ 0.5 null 1 { pop 20 now add } ] sequence";
    assert_eq!(eval_at("", seq, 2), -1.0);
    assert_eq!(eval_at("", seq, 7), 20.2);
    let w = eval_at("", "[3 7] 2 50 wiggle", 4);
    assert!((-50.0..=50.0).contains(&w));
    assert_ne!(w, eval_at("", "[3 8] 2 50 wiggle", 4));
    assert_eq!(w, eval_at("", "[3 7] 2 50 wiggle", 4));
}

/// `delayed` only shifts the clock: `turns` and `loopnoise` inside it are
/// the undelayed value from `d` seconds earlier, so a loop still closes
/// (issue 40). `progress` and `span` still see the shortened window.
#[test]
fn delayed_is_a_pure_shift_for_turns_and_loopnoise() {
    // The film is 2 s at 10 fps: frame 5 is t = 0.5, frame 3 is t = 0.3.
    assert_eq!(eval_at("", "0.2 { 2 turns } delayed", 5), 108.0);
    assert_eq!(
        eval_at("", "0.2 { 2 turns } delayed", 5),
        eval_at("", "2 turns", 3)
    );
    assert_eq!(
        eval_at("", "0.2 { 4 0.8 loopnoise } delayed", 5),
        eval_at("", "4 0.8 loopnoise", 3)
    );
    // Through the words built on it.
    assert_eq!(
        eval_at("", "3 0.1 { 2 turns } staggered", 5),
        eval_at("", "2 turns", 2)
    );
    assert_eq!(
        eval_at("", "[ 0 0 ] 0.1 { pop pop 2 turns } cascade add", 5),
        eval_at("", "2 turns", 5) + eval_at("", "2 turns", 4)
    );
    // The loop closes: a delayed turn gains whole turns over the film.
    let gain =
        eval_at("", "0.3 { 3 turns } delayed", 19) - eval_at("", "0.3 { 3 turns } delayed", 0);
    let step =
        eval_at("", "0.3 { 3 turns } delayed", 1) - eval_at("", "0.3 { 3 turns } delayed", 0);
    assert!((gain + step - 1080.0).abs() < 0.01, "{gain} {step}");
    // Inside a window the loop is the window, delayed or not.
    assert_eq!(
        eval_at("", "0 1 { 0.1 { 3 turns } delayed } during", 5),
        432.0
    );
    // progress and span keep the shortened window, as they always did.
    assert!((eval_at("", "0.2 { progress } delayed", 5) - 0.3 / 1.8).abs() < 0.001);
    assert!((eval_at("", "0.2 { span } delayed", 5) - 1.8).abs() < 0.001);
}

#[test]
fn polypath_turns_and_loopnoise() {
    assert_eq!(eval_at("", "[0 0 10 0 10 10] polypath pathlength", 0), 20.0);
    // In the current user space, like any path.
    assert_eq!(
        eval_at("", "3 3 scale [0 0 10 0] polypath pathlength", 0),
        10.0
    );
    assert_eq!(
        eval_at(
            "",
            "gsave 2 2 scale [0 0 10 0] polypath grestore 5 5 moveto pathlength",
            0
        ),
        0.0
    );
    assert_eq!(eval_at("", "2 turns", 5), 180.0);
    assert_eq!(eval_at("", "0 1 { 3 turns } during", 5), 540.0);
    // loopnoise closes: its value just before the end is close to its value at the start.
    let at = |k| eval_at("", "4 0.8 loopnoise 1000 mul", k);
    assert!((-1000.0..=1000.0).contains(&at(7)));
    assert!(
        (at(19) - at(0)).abs() < 2.0 * (at(1) - at(0)).abs() + 1.0,
        "{} {} {}",
        at(19),
        at(0),
        at(1)
    );
    assert_ne!(at(3), at(11));
}

#[test]
fn render_at_takes_any_time() {
    let mut m = load("/scene { now 2 div dup dup setrgbcolor 0 0 64 48 rectfill } def");
    let f = m.render_at(1.0).unwrap();
    assert_eq!((f.index, f.time), (10, 1.0));
    assert_eq!(px(&f, 0, 0)[0], 128);
    let f = m.render_at(0.333).unwrap();
    assert_eq!(f.index, 3);
    assert_eq!(px(&f, 0, 0)[0], 42);
}

#[test]
fn time_scopes_rebase_the_clock() {
    // during: live only inside the window, local clock from 0.
    assert_eq!(eval_at("", "-1 0.5 1.5 { pop now } during", 2), -1.0);
    assert_eq!(eval_at("", "-1 0.5 1.5 { pop now } during", 7), 0.2);
    assert_eq!(eval_at("", "-1 0.5 1.5 { pop progress } during", 10), 0.5);
    assert_eq!(eval_at("", "-1 0.5 1.5 { pop span } during", 10), 1.0);
    assert_eq!(eval_at("", "-1 0.5 1.5 { pop now } during", 15), -1.0);
    // after / before
    assert_eq!(eval_at("", "-1 0.5 { pop now } after", 3), -1.0);
    assert_eq!(eval_at("", "-1 0.5 { pop now } after", 9), 0.4);
    assert_eq!(eval_at("", "-1 0.5 { pop progress } before", 2), 0.4);
    assert_eq!(eval_at("", "-1 0.5 { pop now } before", 5), -1.0);
    // delayed, staggered, timescaled
    assert_eq!(eval_at("", "0.3 { now } delayed", 5), 0.2);
    assert_eq!(eval_at("", "3 0.1 { now } staggered", 5), 0.2);
    assert_eq!(eval_at("", "2 { now } timescaled", 5), 1.0);
    // looped, pingpong
    assert_eq!(eval_at("", "0.4 { now } looped", 9), 0.1);
    assert_eq!(eval_at("", "0.4 { progress } looped", 9), 0.25);
    assert_eq!(eval_at("", "0.4 { now } pingpong", 3), 0.3);
    assert_eq!(eval_at("", "0.4 { now } pingpong", 5), 0.3);
    assert_eq!(eval_at("", "0.4 { now } pingpong", 6), 0.2);
    // attime, timewindow, reversed
    assert_eq!(eval_at("", "1.25 { now } attime", 3), 1.25);
    assert_eq!(eval_at("", "0.5 2 { progress } timewindow", 3), 0.25);
    assert_eq!(eval_at("", "{ now } reversed", 5), 1.5);
    // Scopes nest, and the clock comes back afterwards.
    assert_eq!(eval_at("", "0.5 2 { 0.2 { now } delayed } during", 9), 0.2);
    assert_eq!(eval_at("", "0.5 2 { 0.2 { } delayed } during now", 9), 0.9);
    // A procedure can be given by name.
    assert_eq!(eval_at("/lt { now } def", "0.3 /lt delayed", 5), 0.2);
}

#[test]
fn sequence_plays_clips_in_turn() {
    let seq = "-1 [ 0.5 { pop 10 now add } 1 { pop 20 now add } 0.5 /third ] sequence";
    let setup = "/third { pop 30 progress add } def";
    assert_eq!(eval_at(setup, seq, 2), 10.2);
    assert_eq!(eval_at(setup, seq, 5), 20.0);
    assert_eq!(eval_at(setup, seq, 14), 20.9);
    assert_eq!(eval_at(setup, seq, 17), 30.4);
    // Past the end nothing plays.
    let m = Movie::load(
        "<< /Size [8 8] /Duration 4 /FPS 10 >> animation /scene { [ 1 { 1 1 1 setrgbcolor 0 0 8 8 rectfill } ] sequence } def",
        &Options::default(),
    );
    let mut m = m.unwrap();
    assert_eq!(px(&m.render_frame(5).unwrap(), 0, 0)[0], 255);
    assert_eq!(px(&m.render_frame(15).unwrap(), 0, 0)[0], 0);
}

#[test]
fn tween_ease_between_and_friends() {
    assert_eq!(eval_at("", "0.5 1.5 /linear tween", 10), 0.5);
    assert_eq!(eval_at("", "0.5 1.5 /linear tween", 2), 0.0);
    assert_eq!(eval_at("", "0.5 1.5 /linear tween", 19), 1.0);
    assert_eq!(eval_at("", "0.5 1.5 /quad-in tween", 10), 0.25);
    assert_eq!(eval_at("", "0.5 1.5 ramp", 10), 0.5);
    assert_eq!(eval_at("", "0.5 /cubic-out ease", 0), 0.875);
    assert_eq!(eval_at("", "0.25 100 200 between", 0), 125.0);
    assert_eq!(
        eval_at("", "0.5 [0 10] [4 20] between aload pop add", 0),
        17.0
    );
    assert_eq!(eval_at("", "1.5 0 100 between", 0), 150.0); // unclamped
    assert_eq!(eval_at("", "7 0 5 clamp", 0), 5.0);
    assert_eq!(eval_at("", "2.75 1 fmod", 0), 0.75);
    assert_eq!(eval_at("", "-0.25 1 fmod", 0), 0.75);
    assert_eq!(eval_at("", "0 10 5 smoothstep", 0), 0.5);
    assert_eq!(eval_at("", "5 0 10 100 200 remap", 0), 150.0);
    // The idiom: tween piped into between.
    assert_eq!(eval_at("", "0 2 /linear tween 100 300 between", 5), 150.0);
}

#[test]
fn keyframes_interpolate_and_hold() {
    let k = "[ 0.5 10  1.0 20  1.5 0 /quad-in ] keyframes";
    assert_eq!(eval_at("", k, 0), 10.0); // before the first key
    assert_eq!(eval_at("", k, 5), 10.0);
    assert_eq!(eval_at("", k, 7), 14.0);
    assert_eq!(eval_at("", k, 10), 20.0);
    assert_eq!(eval_at("", k, 12), 20.0 - 20.0 * 0.16); // quad-in at u=0.4
    assert_eq!(eval_at("", k, 19), 0.0); // after the last
    assert_eq!(
        eval_at("", "0.75 [ 0.5 [0 0] 1.0 [8 4] ] keyat aload pop add", 0),
        6.0
    );
}

#[test]
fn hash01_and_wiggle_are_stable_across_frames_and_seeds_differ() {
    let a = eval_at("", "5 hash01", 0);
    assert_eq!(a, eval_at("", "5 hash01", 13));
    assert_ne!(a, eval_at("", "6 hash01", 0));
    assert_ne!(a, eval_at("", "[5 1] hash01", 0));
    assert!((0.0..1.0).contains(&a));
    let w = eval_at("", "1 2 50 wiggle", 4);
    assert!((-50.0..=50.0).contains(&w));
    assert_ne!(w, eval_at("", "1 2 50 wiggle", 5));
    assert_ne!(w, eval_at("", "2 2 50 wiggle", 4));
}

#[test]
fn rand_is_reseeded_every_frame() {
    // The same scatter on every frame: no flicker.
    assert_eq!(
        eval_at("", "rand 1000 mod", 0),
        eval_at("", "rand 1000 mod", 17)
    );
    let mut a = Movie::load(
        &format!(
            "{HEAD}/scene {{ rand 255 mod 255 div dup dup setrgbcolor 0 0 64 48 rectfill }} def"
        ),
        &Options::default(),
    )
    .unwrap();
    let mut b = Movie::load(
        &format!(
            "{HEAD}/scene {{ rand 255 mod 255 div dup dup setrgbcolor 0 0 64 48 rectfill }} def"
        ),
        &Options {
            seed: Some(2),
            ..Options::default()
        },
    )
    .unwrap();
    assert_ne!(
        px(&a.render_frame(0).unwrap(), 0, 0),
        px(&b.render_frame(0).unwrap(), 0, 0)
    );
}

#[test]
fn path_operators() {
    let sq = "newpath 0 0 moveto 10 0 lineto 10 10 lineto 0 10 lineto closepath";
    assert_eq!(eval_at("", &format!("{sq} pathlength"), 0), 40.0);
    assert_eq!(
        eval_at("", &format!("{sq} 0.375 pathpoint pop exch pop"), 0),
        5.0
    );
    assert_eq!(
        eval_at("", &format!("{sq} 0.375 pathpoint 3 1 roll pop pop"), 0),
        90.0
    );
    // trimpath changes the path's length in proportion, under a transform too.
    assert_eq!(
        eval_at(
            "",
            &format!("3 3 scale {sq} 0.25 0.75 trimpath pathlength"),
            0
        ),
        20.0
    );
    assert_eq!(
        eval_at("", &format!("{sq} 0 1 trimpath pathlength"), 0),
        40.0
    );
    assert_eq!(
        eval_at(
            "",
            "newpath 0 0 moveto 10 0 lineto 0 5 moveto 10 5 lineto 0 0.5 trimeach pathlength",
            0
        ),
        10.0
    );
    // resample + morph: halfway between a small and a large square.
    let morph = format!(
        "{sq} 4 resamplepath
         newpath 0 0 moveto 30 0 lineto 30 30 lineto 0 30 lineto closepath 4 resamplepath
         0.5 morphpath pathlength"
    );
    assert_eq!(eval_at("", &morph, 0), 80.0);
    assert_eq!(eval_at("", &format!("{sq} 8 resamplepath length"), 0), 16.0);
}

#[test]
fn alignpath_turns_an_outline_to_its_best_start_and_sense() {
    // The same square, started at another corner and walked the other way:
    // aligned, the morph does not move a point at all.
    let a = "newpath 0 0 moveto 20 0 lineto 20 20 lineto 0 20 lineto closepath 8 resamplepath";
    let b = "newpath 20 20 moveto 20 0 lineto 0 0 lineto 0 20 lineto closepath 8 resamplepath";
    let still = format!(
        "{a} dup {b} alignpath 0.5 morphpath pathlength {a} {a} 0.5 morphpath pathlength sub"
    );
    assert_eq!(eval_at("", &still, 0), 0.0);
    // Unaligned, the halfway shape is smaller (the points cross the middle).
    let raw = format!("{a} {b} 0.5 morphpath pathlength");
    assert!(eval_at("", &raw, 0) < 79.0);
}

#[test]
fn trimpath_draws_on() {
    let mut m = load(
        "/scene { 0 24 moveto 64 24 lineto 0 progress 2 mul trimpath 1 setgray 8 setlinewidth stroke } def",
    );
    let f = m.render_frame(5).unwrap(); // progress 0.25 -> half the line
    assert_eq!(px(&f, 16, 24)[0], 255);
    assert_eq!(px(&f, 48, 24)[0], 0);
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 48, 24)[0], 255);
}

#[test]
fn layers_fade_as_a_group_and_blend() {
    // Two overlapping white squares at half opacity: as a group the overlap
    // is no brighter than the rest.
    let mut m =
        load("/scene { 0.5 { 1 setgray 0 0 40 48 rectfill 20 0 44 48 rectfill } faded } def");
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 10, 10)[0], 128);
    assert_eq!(px(&f, 30, 10)[0], 128, "the overlap");
    // Additive blend.
    let mut m = load(
        "/scene { 0.4 0.4 0.4 backdrop 1 /add { 0.5 0 0 setrgbcolor 0 0 32 48 rectfill } blended } def",
    );
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 5, 5), [230, 102, 102, 255]);
    assert_eq!(px(&f, 50, 5), [102, 102, 102, 255]);
    // Explicit begin/end with a nested layer.
    let mut m = load(
        "/scene { beginlayer 1 0 0 setrgbcolor 0 0 64 48 rectfill
                  beginlayer 0 0 1 setrgbcolor 0 0 32 48 rectfill 1 endlayer
                  0.5 endlayer } def",
    );
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 5, 5), [0, 0, 128, 255]);
    assert_eq!(px(&f, 50, 5), [128, 0, 0, 255]);
}

#[test]
fn layer_combinator_bodies_see_the_callers_stack() {
    // The body reaches operands left under the combinator's own.
    for body in [
        "20 0.5 { 0 30 48 rectfill } faded",
        "20 0.5 /normal { 0 30 48 rectfill } blended",
        "20 0.01 { 0 30 48 rectfill } blurred",
        "20 0.01 0.01 { 0 30 48 rectfill } glowing",
    ] {
        let mut m = load(&format!("/scene {{ 1 setgray {body} }} def"));
        let f = m.render_frame(0).unwrap_or_else(|e| panic!("{body}: {e}"));
        assert!(px(&f, 30, 10)[0] > 100, "{body}");
        assert_eq!(px(&f, 10, 10)[0], 0, "{body}");
    }
    let mut m = load(
        "/scene { 7 { 1 0 0 setrgbcolor 0 0 64 48 rectfill } { 0 0 1 setrgbcolor 0 0 64 48 rectfill } 0.5 crossfade
                  7 ne { wrong } if } def",
    );
    assert_eq!(px(&m.render_frame(0).unwrap(), 5, 5), [127, 0, 128, 255]);
}

#[test]
fn blur_and_glow_spread_light() {
    let mut sharp = load("/scene { 1 setgray 28 20 8 8 rectfill } def");
    let mut soft = load("/scene { 3 { 1 setgray 28 20 8 8 rectfill } blurred } def");
    let mut glow = load("/scene { 3 1 { 1 setgray 28 20 8 8 rectfill } glowing } def");
    let s = sharp.render_frame(0).unwrap().premultiplied.to_vec();
    let at = |buf: &[u8], x: usize, y: usize| buf[(y * 64 + x) * 4];
    assert_eq!((at(&s, 32, 24), at(&s, 24, 24)), (255, 0));
    let b = soft.render_frame(0).unwrap().premultiplied.to_vec();
    assert!(at(&b, 32, 24) < 255 && at(&b, 24, 24) > 0, "blurred edges");
    let g = glow.render_frame(0).unwrap().premultiplied.to_vec();
    assert_eq!(at(&g, 32, 24), 255, "glow keeps the core");
    assert!(at(&g, 24, 24) > 0, "and adds a halo");
}

#[test]
fn blur_and_glow_stay_inside_a_within_box() {
    // The box is x 16..48, rows 12..36. The blur and the glow are drawn
    // inside it and must not spread past it; the backdrop is kept.
    for effect in [
        "4 { 1 setgray 0 0 32 24 rectfill } blurred",
        "4 1.5 { 1 setgray 0 0 32 24 rectfill } glowing",
        "beginlayer 1 setgray 0 0 32 24 rectfill 4 blurlayer 1 endlayer",
        "beginlayer 1 setgray 0 0 32 24 rectfill 4 1.5 glowlayer 1 endlayer",
        "1 setgray 0 0 32 24 rectfill 4 blurlayer",
    ] {
        let mut m = load(&format!(
            "/scene {{ 0 0 0.5 setrgbcolor 0 0 64 48 rectfill [16 12 32 24] {{ {effect} }} within }} def"
        ));
        let f = m.render_frame(0).unwrap();
        for y in 0..48 {
            for x in 0..64 {
                if !((16..48).contains(&x) && (12..36).contains(&y)) {
                    assert_eq!(px(&f, x, y), [0, 0, 128, 255], "{effect}: {x},{y}");
                }
            }
        }
        assert_eq!(px(&f, 32, 24)[0], 255, "{effect}: the middle is lit");
    }
}

#[test]
fn a_glow_follows_a_rotated_clip_exactly() {
    // A square clip turned 45 degrees: a diamond about (50, 50) reaching
    // 28 pixels each way. (30, 30) lies inside the diamond's bounding box
    // but outside the diamond, within the glow's reach: it must stay dark.
    let mut m = Movie::load(
        "<< /Size [100 100] /Duration 1 /FPS 10 /Background [0 0 0] >> animation
         /scene { gsave 50 50 translate 45 rotate -20 -20 40 40 rectclip
                  6 1.5 { 0.5 setgray -50 -50 100 100 rectfill } glowing grestore } def",
        &Options::default(),
    )
    .unwrap();
    let f = m.render_frame(0).unwrap();
    assert!(px(&f, 50, 50)[0] > 128, "the glow brightens the inside");
    for (x, y) in [(30, 30), (70, 70), (30, 70), (70, 30)] {
        assert_eq!(
            px(&f, x, y),
            [0, 0, 0, 255],
            "{x},{y} is outside the diamond"
        );
    }
}

#[test]
fn a_blur_ends_inside_the_clip_it_began_in() {
    // Not the clip in force where it ends: a clip set and kept by the
    // body does not decide where the blur may spread...
    let mut m = load(
        "/scene { [16 12 32 24] { 4 { 1 setgray 0 0 32 24 rectfill } blurred } within
                  8 { 0 0 4 4 rectclip 1 setgray 0 0 4 4 rectfill } blurred } def",
    );
    let f = m.render_frame(0).unwrap();
    // ...but the clip it began under (none here) does: the corner square's
    // blur reaches past its own clip.
    assert!(
        px(&f, 6, 47 - 2)[0] > 0,
        "the blur spreads past the body's own clip"
    );
    assert_eq!(
        px(&f, 14, 24),
        [0, 0, 0, 255],
        "and the within box still holds"
    );
}

#[test]
fn an_unclosed_layer_is_reported_and_does_not_poison_the_next_frame() {
    let mut m = load("/scene { frame 1 eq { beginlayer } if 1 setgray 0 0 64 48 rectfill } def");
    let e = m.render_frame(1).unwrap_err();
    assert_eq!(e.kind, "unclosedlayer");
    assert!(e.hint.unwrap().contains("endlayer"));
    assert_eq!(px(&m.render_frame(2).unwrap(), 5, 5)[0], 255);
}

#[test]
fn backdrop_camera_vignette_grain() {
    let mut m = load("/scene { 0.2 0.4 0.6 backdrop } def");
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 0, 0), [51, 102, 153, 255]);
    assert_eq!(px(&f, 63, 47), [51, 102, 153, 255]);
    // camera: the world point looked at lands in the middle of the frame.
    let mut m = load("/scene { 500 500 2 0 camera 1 setgray 498 498 4 4 rectfill } def");
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 32, 24)[0], 255);
    assert_eq!(px(&f, 20, 24)[0], 0);
    let mut m = load("/scene { 1 1 1 backdrop 1 vignette } def");
    let f = m.render_frame(0).unwrap();
    assert_eq!(px(&f, 32, 24)[0], 255);
    assert!(px(&f, 0, 0)[0] < 30);
    let mut m = load("/scene { 0.5 0.5 0.5 backdrop 0.5 filmgrain } def");
    let a = m.render_frame(0).unwrap().premultiplied.to_vec();
    let b = m.render_frame(1).unwrap().premultiplied.to_vec();
    assert_ne!(a, b, "grain moves");
    assert_eq!(
        m.render_frame(0).unwrap().premultiplied,
        &a[..],
        "and is repeatable"
    );
    assert!(a.chunks(4).any(|p| p[0] != 128));
}

#[test]
fn text_helpers() {
    assert_eq!(eval_at("", "(abcdefgh) 0.5 typed length", 0), 4.0);
    assert_eq!(eval_at("", "(abc) 0 typed length", 0), 0.0);
    assert_eq!(eval_at("", "(abc) 9 typed length", 0), 3.0);
    // eachglyph visits every character in order, at advancing positions.
    let setup = "/acc 0 def /lastx -1 def /ok 1 def";
    let expr = "/Helvetica findfont 10 scalefont setfont 5 5 moveto (HELLO) {
        exch pop                           % i
        acc add /acc exch def
        currentpoint pop dup lastx le { /ok 0 def } if /lastx exch def
      } eachglyph acc ok 100 mul add";
    assert_eq!(eval_at(setup, expr, 0), 0.0 + 1.0 + 2.0 + 3.0 + 4.0 + 100.0);
    // `{ pop show }` reproduces show.
    let mut a = load(
        "/scene { /Helvetica findfont 12 scalefont setfont 1 setgray 4 20 moveto (Wave) show } def",
    );
    let mut b = load(
        "/scene { /Helvetica findfont 12 scalefont setfont 1 setgray 4 20 moveto (Wave) { pop show } eachglyph } def",
    );
    let fa = a.render_frame(0).unwrap().premultiplied.to_vec();
    assert!(fa.contains(&255));
    assert_eq!(b.render_frame(0).unwrap().premultiplied, &fa[..]);
    // showcenter centres on the current point.
    let mut m = load(
        "/scene { /Helvetica findfont 12 scalefont setfont 1 setgray 32 20 moveto (WWWW) showcenter } def",
    );
    let f = m.render_frame(0).unwrap().premultiplied.to_vec();
    let cols: Vec<usize> = (0..64)
        .filter(|x| (0..48).any(|y| f[(y * 64 + x) * 4] > 100))
        .collect();
    let mid = (cols[0] + cols[cols.len() - 1]) as f64 / 2.0;
    assert!((mid - 32.0).abs() < 1.5, "{mid}");
}

fn reel_movie(transition: &str) -> Movie {
    Movie::load(
        &format!(
            "<< /Size [40 20] /Duration 3 /FPS 10 >> animation
             /red  {{ 1 0 0 setrgbcolor 0 0 40 20 rectfill }} def
             /blue {{ 0 0 1 setrgbcolor 0 0 40 20 rectfill }} def
             /film [ [1.5 /red] [1 /{transition}] [1.5 {{ blue }}] ] def
             /total film reelduration def
             /scene {{ film reel }} def"
        ),
        &Options::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn reel_places_clips_and_transitions() {
    // 1.5 + 1.5 - 1 overlap = 2 seconds; the transition runs 0.5..1.5.
    let mut m = reel_movie("dissolve");
    assert_eq!(px(&m.render_frame(2).unwrap(), 5, 5), [255, 0, 0, 255]);
    assert_eq!(px(&m.render_frame(10).unwrap(), 5, 5), [127, 0, 128, 255]);
    assert_eq!(px(&m.render_frame(17).unwrap(), 5, 5), [0, 0, 255, 255]);
    // After the reel ends nothing is drawn: the background shows.
    assert_eq!(px(&m.render_frame(25).unwrap(), 5, 5), [0, 0, 0, 255]);
}

#[test]
fn every_transition_starts_on_a_and_ends_on_b() {
    for name in psanim::TRANSITIONS {
        let mut m = reel_movie(name);
        let first = m.render_frame(5).unwrap(); // u = 0
        let n = first
            .premultiplied
            .chunks(4)
            .filter(|p| p[0] > 200 && p[2] < 50)
            .count();
        assert!(
            n as f64 > 0.95 * 800.0,
            "{name} at u=0 shows {n} red pixels"
        );
        let last = m.render_frame(15).unwrap(); // just past the transition
        assert!(
            last.premultiplied.chunks(4).all(|p| p == [0, 0, 255, 255]),
            "{name} after"
        );
        // Late in the transition B dominates.
        let late = m.render_frame(14).unwrap();
        let blue = late.premultiplied.chunks(4).filter(|p| p[2] > p[0]).count();
        assert!(
            blue as f64 > 0.6 * 800.0,
            "{name} at u=0.9 shows {blue} blue pixels"
        );
    }
}

#[test]
fn clips_that_start_with_backdrop_can_be_wiped_and_pushed() {
    let mut m = Movie::load(
        "<< /Size [40 20] /Duration 3 /FPS 10 >> animation
         /a { 1 0 0 backdrop } def /b { 0 0 1 backdrop } def
         /scene { [ [1.5 /a] [1 /wipe-right] [1.5 /b] ] reel } def",
        &Options::default(),
    )
    .unwrap();
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 2, 10), [0, 0, 255, 255]);
    assert_eq!(
        px(&f, 37, 10),
        [255, 0, 0, 255],
        "the outgoing clip's backdrop is still there"
    );
    let mut m = Movie::load(
        "<< /Size [40 20] /Duration 3 /FPS 10 >> animation
         /a { 1 0 0 backdrop } def /b { 0 0 1 backdrop } def
         /scene { [ [1.5 /a] [1 /push-left] [1.5 /b] ] reel } def",
        &Options::default(),
    )
    .unwrap();
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 2, 10), [255, 0, 0, 255]);
    assert_eq!(px(&f, 37, 10), [0, 0, 255, 255]);
}

#[test]
fn a_program_can_add_a_transition_and_bad_names_fail_early() {
    // Any name put in `transitions` is one; the reel's length accounts for it.
    let src = |entry: &str| {
        format!(
            "/a {{ 1 0 0 backdrop }} def /b {{ 0 0 1 backdrop }} def
             transitions /halves {{ pop exch gsave 0 0 20 20 rectclip exec grestore gsave 20 0 20 20 rectclip exec grestore }} put
             /film [ [1.5 /a] [1 /{entry}] [1.5 /b] ] def
             << /Size [40 20] /Duration film reelduration /FPS 10 >> animation
             /scene {{ film reel }} def"
        )
    };
    let mut m = Movie::load(&src("halves"), &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.meta().duration, 2.0);
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 5, 10), [255, 0, 0, 255]);
    assert_eq!(px(&f, 30, 10), [0, 0, 255, 255]);
    // A name that is neither is refused as soon as the reel plays, on the
    // first frame, not only when that entry's turn comes.
    let mut m = Movie::load(&src("nosuchthing"), &Options::default()).unwrap();
    let e = m.render_frame(0).unwrap_err();
    assert!(
        e.hint
            .unwrap()
            .contains("/nosuchthing is neither a transition"),
        "hint names it"
    );
}

/// A reel measured by `reelduration` in the declaration, with a custom
/// transition named `name` put in `transitions` either `before` the
/// declaration or after it.
fn measured_reel(name: &str, before: bool) -> Result<Movie, Box<psanim::Error>> {
    let define = format!("transitions /{name} {{ pop exch pop exec }} put\n");
    let (early, late) = if before {
        (define, String::new())
    } else {
        (String::new(), define)
    };
    Movie::load(
        &format!(
            "/a {{ 1 0 0 backdrop }} def /b {{ 0 0 1 backdrop }} def
             {early}/film [ [1.5 /a] [1 /{name}] [1.5 /b] ] def
             << /Size [40 20] /Duration film reelduration /FPS 10 >> animation
             {late}/scene {{ film reel }} def"
        ),
        &Options::default(),
    )
    .map_err(Box::new)
}

#[test]
fn reelduration_before_a_custom_transition_is_defined_is_an_error() {
    // `shutter` is not built in and does not start with `t-`, and it is put
    // in `transitions` after `reelduration` measured the reel: it was
    // counted as a clip (4 s, not 2 s). That must be loud.
    let e = measured_reel("shutter", false).err().expect("an error");
    assert_eq!(e.kind, "reelduration");
    assert!(
        e.message.contains("/shutter"),
        "names the entry: {}",
        e.message
    );
    assert!(e.message.contains("4 s"), "{}", e.message);
    assert!(e.message.contains("2 s"), "{}", e.message);
    assert!(e.message.contains("name it t-shutter"), "{}", e.message);
    assert!(e.message.contains("before `reelduration`"), "{}", e.message);
}

#[test]
fn reelduration_with_the_right_order_or_a_t_name_still_works() {
    // Defined before the measurement, with any name.
    let m = measured_reel("shutter", true).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.meta().duration, 2.0);
    // Defined after, but named `t-...`: recognised before it exists.
    let mut m = measured_reel("t-shutter", false).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.meta().duration, 2.0);
    assert_eq!(px(&m.render_frame(2).unwrap(), 5, 5), [255, 0, 0, 255]);
    // Built-in transitions, with clips defined after the measurement.
    let m = Movie::load(
        "/film [ [1.5 /a] [1 /dissolve] [1.5 /b] ] def
         << /Size [40 20] /Duration film reelduration /FPS 10 >> animation
         /a { 1 0 0 backdrop } def /b { 0 0 1 backdrop } def
         /scene { film reel } def",
        &Options::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.meta().duration, 2.0);
}

#[test]
fn a_t_name_that_is_never_defined_fails_when_the_reel_plays() {
    let mut m = Movie::load(
        "/a { 1 0 0 backdrop } def /b { 0 0 1 backdrop } def
         /film [ [1.5 /a] [1 /t-missing] [1.5 /b] ] def
         << /Size [40 20] /Duration film reelduration /FPS 10 >> animation
         /scene { film reel } def",
        &Options::default(),
    )
    .unwrap();
    let e = m.render_frame(0).unwrap_err();
    assert!(
        e.hint
            .unwrap()
            .contains("/t-missing is neither a transition"),
        "names it"
    );
}

#[test]
fn directional_transitions_move_the_right_way() {
    // wipe-right: the reveal travels rightward, so B appears at the left first.
    let mut m = reel_movie("wipe-right");
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 2, 10), [0, 0, 255, 255]);
    assert_eq!(px(&f, 37, 10), [255, 0, 0, 255]);
    let mut m = reel_movie("wipe-left");
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 2, 10), [255, 0, 0, 255]);
    assert_eq!(px(&f, 37, 10), [0, 0, 255, 255]);
    // wipe-up: from the bottom of the frame (the last rows of the buffer).
    let mut m = reel_movie("wipe-up");
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 20, 18), [0, 0, 255, 255]);
    assert_eq!(px(&f, 20, 1), [255, 0, 0, 255]);
    let mut m = reel_movie("wipe-down");
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 20, 1), [0, 0, 255, 255]);
    assert_eq!(px(&f, 20, 18), [255, 0, 0, 255]);
    // push-left: B enters from the right.
    let mut m = reel_movie("push-left");
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 2, 10), [255, 0, 0, 255]);
    assert_eq!(px(&f, 37, 10), [0, 0, 255, 255]);
    // iris: B opens from the centre.
    let mut m = reel_movie("iris");
    let f = m.render_frame(8).unwrap();
    assert_eq!(px(&f, 20, 10), [0, 0, 255, 255]);
    assert_eq!(px(&f, 1, 1), [255, 0, 0, 255]);
}

#[test]
fn reel_clips_keep_their_own_clocks_through_a_transition() {
    // Each clip paints its local time into a channel.
    let mut m = Movie::load(
        "<< /Size [8 8] /Duration 2 /FPS 10 >> animation
         /a { now 1.5 div 0 0 setrgbcolor 0 0 4 8 rectfill } def
         /b { 0 0 now 1.5 div setrgbcolor 4 0 4 8 rectfill } def
         /scene { [ [1.5 /a] [1 /cut] [1.5 /b] ] reel } def",
        &Options::default(),
    )
    .unwrap();
    // t = 0.7: in the transition, before the cut: A at local 0.7.
    assert_eq!(
        px(&m.render_frame(7).unwrap(), 1, 1)[0],
        (0.7f64 / 1.5 * 255.0).round() as u8
    );
    // t = 1.2: after the cut: B at local 0.7 (it started at 0.5).
    assert_eq!(
        px(&m.render_frame(12).unwrap(), 6, 1)[2],
        (0.7f64 / 1.5 * 255.0).round() as u8
    );
}

#[test]
fn params_signals_and_events_reach_the_program() {
    let src = format!(
        "{HEAD}/gain /gain 2 param def /label /label (none) param def
         /scene {{ /level signal gain mul /beat sinceevent 10 lt {{ 0.5 }} {{ 0 }} ifelse /beat eventcount 4 div
                  setrgbcolor 0 0 64 48 rectfill }} def"
    );
    let mut opts = Options::default();
    opts.params.insert("gain".into(), Param::parse("0.5"));
    opts.params.insert("label".into(), Param::parse("hello"));
    opts.signals = psanim::Signals::from_csv("t,level,beat!\n0,0,\n1,1,x\n2,1,x\n").unwrap();
    let mut m = Movie::load(&src, &opts).unwrap();
    assert_eq!(m.meta().params.get("gain").map(String::as_str), Some("2"));
    assert_eq!(
        m.meta().params.get("label").map(String::as_str),
        Some("(none)")
    );
    assert_eq!(px(&m.render_frame(5).unwrap(), 0, 0), [64, 0, 0, 255]); // level .5 * gain .5, no beat yet
    assert_eq!(px(&m.render_frame(15).unwrap(), 0, 0), [128, 128, 64, 255]); // one beat
    // A live host overrides the signal.
    m.set_signal("level", 2.0);
    assert_eq!(px(&m.render_frame(5).unwrap(), 0, 0)[0], 255);
    // Defaults when the host supplies nothing.
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    assert_eq!(px(&m.render_frame(5).unwrap(), 0, 0), [0, 0, 0, 255]);
}

#[test]
fn a_signal_has_a_past_that_a_movie_can_read() {
    // 10 FPS; level rises 0 -> 10 over the first second, then holds.
    let src = format!(
        "{HEAD}/scene {{ /level 0.5 signalago == /level 1 3 signalseries == /level 0 1 3 signalbetween ==
                  signaltime == /level hassignal == /nope hassignal == /nope 1 signalago == }} def"
    );
    let opts = Options {
        signals: psanim::Signals::from_csv("t,level\n0,0\n1,10\n").unwrap(),
        ..Options::default()
    };
    let mut m = Movie::load(&src, &opts).unwrap();
    m.take_output();
    m.render_frame(10).unwrap(); // t = 1
    let out = m.take_output();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "5.0", "{out}");
    assert_eq!(lines[1], "[0.0 5.0 10.0]", "{out}");
    assert_eq!(lines[2], "[0.0 5.0 10.0]", "{out}");
    assert_eq!(lines[3], "1.0", "{out}");
    assert_eq!(&lines[4..], ["true", "false", "0.0"], "{out}");
    // A live host records readings as they come; the movie reads between them.
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.record_signal("level", 0.0, 0.0);
    m.record_signal("level", 1.0, 10.0);
    m.take_output();
    m.render_at(1.0).unwrap();
    assert!(m.take_output().starts_with("5.0\n[0.0 5.0 10.0]"));
    // signaltime is the movie's clock inside a component too.
    let src = format!(
        "{HEAD}/c << /Size [10 10] /Duration 9 /Scene {{ globaltime == signaltime == }} >> component
         /scene {{ /c << /At 0.5 >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap();
    m.take_output();
    m.render_at(2.0).unwrap();
    assert_eq!(m.take_output(), "1.5\n2.0\n");
}

const BOUNCE: &str = "<< /Size [64 48] /Duration 4 /FPS 10 /StepRate 50 >> animation
    /x 5 def /vx 40 def /bounces 0 def
    /step { /h dt def
      /x x vx h mul add def
      x 60 gt { /vx vx neg def /bounces bounces 1 add def } if
      x 4 lt  { /vx vx neg def } if
      % a little randomness, to check replay reproduces it
      /x x rand 100 mod 100 div 0.5 sub 0.2 mul add def
    } def
    /scene { 1 setgray x 2 sub 22 4 4 rectfill
             /x 999 def   % the scene cannot corrupt the simulation
           } def";

#[test]
fn stateful_movies_replay_to_the_same_frames() {
    let mut m = Movie::load(BOUNCE, &Options::default()).unwrap();
    assert_eq!(m.meta().mode, Mode::Stateful);
    assert_eq!(m.meta().tick_rate, 50.0);
    let forward: Vec<Vec<u8>> = (0..40)
        .map(|k| m.render_frame(k).unwrap().premultiplied.to_vec())
        .collect();
    assert_ne!(forward[0], forward[5]);
    // The ball moves about 4 units a frame at first.
    let col = |buf: &[u8]| (0..64).find(|x| buf[(24 * 64 + x) * 4] > 0).unwrap();
    assert!((col(&forward[5]) as i64 - col(&forward[0]) as i64 - 20).abs() <= 2);
    // Backward, random access, repeated: identical.
    for k in [39, 0, 20, 19, 21, 3, 3, 38] {
        assert_eq!(
            m.render_frame(k).unwrap().premultiplied,
            &forward[k as usize][..],
            "frame {k}"
        );
    }
    // A fresh movie jumping straight to a late frame agrees.
    let mut cold = Movie::load(BOUNCE, &Options::default()).unwrap();
    assert_eq!(
        cold.render_frame(33).unwrap().premultiplied,
        &forward[33][..]
    );
}

#[test]
fn stateful_result_does_not_depend_on_the_output_frame_rate() {
    let mut a = Movie::load(BOUNCE, &Options::default()).unwrap();
    let mut b = Movie::load(
        BOUNCE,
        &Options {
            fps: Some(25.0),
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(b.meta().tick_rate, 50.0);
    // t = 2.0 is frame 20 at 10 fps and frame 50 at 25 fps.
    let fa = a.render_frame(20).unwrap().premultiplied.to_vec();
    assert_eq!(b.render_frame(50).unwrap().premultiplied, &fa[..]);
}

const TRAIL: &str =
    "<< /Size [64 48] /Duration 6 /FPS 10 /Persist true /Background [0 0 0] >> animation
    /scene { 1 setgray frame 1 add 0 1 48 rectfill } def";

#[test]
fn persistent_movies_accumulate_and_seek() {
    let mut m = Movie::load(TRAIL, &Options::default()).unwrap();
    assert_eq!(m.meta().mode, Mode::Persistent);
    let lit = |f: &psanim::Frame| (0..64).filter(|x| px(f, *x, 10)[0] == 255).count();
    // Frame k has painted columns 1..=k+1.
    assert_eq!(lit(&m.render_frame(0).unwrap()), 1);
    assert_eq!(lit(&m.render_frame(9).unwrap()), 10);
    assert_eq!(lit(&m.render_frame(45).unwrap()), 46);
    // Seeking back restores the canvas of that moment (across a checkpoint).
    assert_eq!(lit(&m.render_frame(25).unwrap()), 26);
    assert_eq!(lit(&m.render_frame(3).unwrap()), 4);
    assert_eq!(lit(&m.render_frame(59).unwrap()), 60);
    let mut cold = Movie::load(TRAIL, &Options::default()).unwrap();
    let a = cold.render_frame(30).unwrap().premultiplied.to_vec();
    assert_eq!(m.render_frame(30).unwrap().premultiplied, &a[..]);
}

#[test]
fn a_persistent_movie_can_show_an_overlay_that_is_not_kept() {
    // The painting adds one column per frame; the overlay is a marker at
    // the brush, which must not be left behind in the painting.
    let src = "<< /Size [64 48] /Duration 3 /FPS 10 /Persist true /Background [0 0 0] >> animation
        /scene { 1 setgray frame 0 1 20 rectfill } def
        /overlay { 1 0 0 setrgbcolor frame 24 4 4 rectfill } def";
    let mut m = Movie::load(src, &Options::default()).unwrap();
    let f = m.render_frame(10).unwrap();
    assert_eq!(px(&f, 11, 21), [255, 0, 0, 255], "the overlay is shown");
    assert_eq!(px(&f, 5, 40)[0], 255, "over the painting");
    let f = m.render_frame(20).unwrap();
    assert_eq!(
        px(&f, 11, 21),
        [0, 0, 0, 255],
        "and an earlier frame's overlay is gone"
    );
    assert_eq!(px(&f, 21, 21), [255, 0, 0, 255]);
    // Seeking back still gives the same frame.
    let again = m.render_frame(10).unwrap().premultiplied.to_vec();
    let mut cold = Movie::load(src, &Options::default()).unwrap();
    assert_eq!(cold.render_frame(10).unwrap().premultiplied, &again[..]);
    assert_eq!(cold.last_frame_ticks(), 11);
}

#[test]
fn redefining_a_psanim_word_is_reported() {
    let m = load("/width 300 def /frame { } def /scene { 1 setgray 0 0 8 8 rectfill } def");
    assert_eq!(m.meta().shadowed, ["frame", "width"]);
    let m = load("/scene { 1 setgray 0 0 8 8 rectfill } def");
    assert!(m.meta().shadowed.is_empty());
    // A word the prelude defines is reported only when the program replaces it.
    let m = load("/vignette { pop } def /scene { } def");
    assert_eq!(m.meta().shadowed, ["vignette"]);
}

#[test]
fn motion_blur_averages_the_shutter_interval() {
    let src = "<< /Size [64 8] /Duration 1 /FPS 4 >> animation
               /scene { 1 setgray now 64 mul 0 2 8 rectfill } def";
    let mut sharp = Movie::load(src, &Options::default()).unwrap();
    let mut blur = Movie::load(
        src,
        &Options {
            motion_blur: Some(Some(MotionBlur {
                samples: 8,
                shutter: 1.0,
            })),
            ..Options::default()
        },
    )
    .unwrap();
    let lit = |f: &psanim::Frame| (0..64).filter(|x| px(f, *x, 4)[0] > 0).count();
    assert_eq!(lit(&sharp.render_frame(1).unwrap()), 2);
    let f = blur.render_frame(1).unwrap();
    // The bar travels 16 units during the frame: a streak, dimmer than the bar.
    assert!((14..=18).contains(&lit(&f)), "{}", lit(&f));
    assert!(px(&f, 24, 4)[0] < 80);
    // Declared in the program, and switched off by the host.
    let declared =
        "<< /Size [64 8] /Duration 1 /FPS 4 /MotionBlur 4 /Shutter 0.5 >> animation /scene { } def";
    let m = Movie::load(declared, &Options::default()).unwrap();
    assert_eq!(
        m.meta().motion_blur,
        Some(MotionBlur {
            samples: 4,
            shutter: 0.5
        })
    );
    let m = Movie::load(
        declared,
        &Options {
            motion_blur: Some(None),
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(m.meta().motion_blur, None);
    // Not available to a simulation.
    let e = match Movie::load(
        BOUNCE,
        &Options {
            motion_blur: Some(Some(MotionBlur {
                samples: 4,
                shutter: 0.5,
            })),
            ..Options::default()
        },
    ) {
        Err(e) => e,
        Ok(_) => panic!(),
    };
    assert_eq!(e.kind, "options");
}

#[test]
fn setup_errors_carry_the_line() {
    let e = load_err("<< /Size [64 48] >> animation\n/scene { } def\n\n1 (two) add\n");
    assert_eq!(e.phase, Phase::Setup);
    assert_eq!(
        (e.kind.as_str(), e.command.as_deref(), e.line),
        ("typecheck", Some("add"), Some(4))
    );
    assert_eq!(e.stack, ["1", "(two)"]);
    assert!(e.to_string().contains("[line 4]"));
    let e = load_err("<< /Size [64 48] >> animation\n/scene { (unterminated } def");
    assert_eq!(e.kind, "syntaxerror");
}

#[test]
fn frame_errors_say_which_frame_and_suggest_a_fix() {
    let mut m = load("/scene { now 0.5 ge { 0 0 1 durring } if } def");
    assert!(m.render_frame(2).is_ok());
    let e = m.render_frame(7).unwrap_err();
    assert_eq!(
        e.phase,
        Phase::Frame {
            index: 7,
            time: 0.7
        }
    );
    assert_eq!(
        (e.kind.as_str(), e.command.as_deref()),
        ("undefined", Some("durring"))
    );
    assert!(e.hint.as_deref().unwrap().contains("`during`"), "{e}");
    assert_eq!(e.line, None);
    assert_eq!(e.to_json()["frame"], 7);
    // The movie still renders other frames afterwards.
    assert!(m.render_frame(3).is_ok());

    let mut m = load("/scene { 0.5 /cubic-oot ease pop } def");
    let e = m.render_frame(0).unwrap_err();
    assert_eq!(e.kind, "rangecheck");
    assert!(e.hint.unwrap().contains("/cubic-out"));

    let mut m = load("/scene { 0 1 during } def");
    let e = m.render_frame(0).unwrap_err();
    assert!(e.hint.unwrap().contains("procedure"));
}

#[test]
fn frame_errors_show_where_in_the_program() {
    let mut m = load(
        "/petal { 0 0 moveto 10 (x) lineto stroke } def
         /flower { 4 { petal 90 rotate } repeat } def
         /scene { 32 24 translate 0.2 1.5 { gsave flower grestore } during 1 setgray } def",
    );
    let e = m.render_frame(5).unwrap_err();
    assert_eq!(e.command.as_deref(), Some("lineto"));
    let names: Vec<Option<&str>> = e.backtrace.iter().map(|t| t.procedure.as_deref()).collect();
    // petal, the repeat body, flower's caller (the during body), scene.
    assert_eq!(names.first(), Some(&Some("petal")), "{e}");
    assert_eq!(names.last(), Some(&Some("scene")), "{e}");
    assert_eq!(
        e.backtrace[0].excerpt,
        "0 0 moveto 10 (x) >>> lineto <<< stroke"
    );
    // The prelude's own plumbing is not in the way.
    assert!(
        e.backtrace.iter().all(|t| !t.excerpt.contains(".timepop")),
        "{e}"
    );
    let text = e.to_string();
    assert!(
        text.contains("in petal: 0 0 moveto 10 (x) >>> lineto <<< stroke"),
        "{text}"
    );
    assert_eq!(e.to_json()["backtrace"][0]["procedure"], "petal");
    // A scene failing at its very last operator is still located.
    let mut m = load("/scene { 1 setgray 5 5 moveto (no font) show } def");
    let e = m.render_frame(0).unwrap_err();
    assert_eq!(e.backtrace[0].procedure.as_deref(), Some("scene"), "{e}");
    assert!(e.backtrace[0].excerpt.ends_with(">>> show <<<"));
}

#[test]
fn declaration_errors_are_explained() {
    let e = load_err("<< /Size [64 48] /Durration 3 >> animation /scene { } def");
    assert_eq!((e.phase, e.kind.as_str()), (Phase::Load, "declaration"));
    assert!(e.hint.unwrap().contains("/Duration"));
    let e = load_err("<< /Size [64.5 48] >> animation /scene { } def");
    assert!(e.message.contains("/Size"));
    let e = load_err("<< /Size [64 48] >> animation /draw { } def");
    assert!(e.message.contains("`scene`") && e.hint.unwrap().contains("/scene"));
    let e = load_err("<< /Size [64 48] /Duration -1 >> animation /scene { } def");
    assert_eq!(e.kind, "options");
    let e = load_err("42 animation /scene { } def");
    assert!(e.hint.unwrap().contains("dictionary"));
    // A custom scene name.
    let mut m = Movie::load(
        "<< /Size [8 8] /Scene /draw >> animation /draw { 1 setgray 0 0 8 8 rectfill } def",
        &Options::default(),
    )
    .unwrap();
    assert_eq!(px(&m.render_frame(0).unwrap(), 0, 0)[0], 255);
}

#[test]
fn budgets_bound_setup_and_frames() {
    let mut opts = Options::default();
    opts.budget.setup = psanim::Limits::none().with_max_steps(50_000);
    opts.budget.frame = psanim::Limits::none().with_max_steps(20_000);
    let e = match Movie::load(&format!("{HEAD}{{ }} loop /scene {{ }} def"), &opts) {
        Err(e) => e,
        Ok(_) => panic!(),
    };
    assert_eq!(
        (e.phase, e.kind.as_str(), e.limit),
        (Phase::Setup, "limitexceeded", Some("steps"))
    );
    let mut m = Movie::load(
        &format!("{HEAD}/scene {{ frame 3 eq {{ {{ }} loop }} if 100 {{ 1 pop }} repeat }} def"),
        &opts,
    )
    .unwrap();
    m.render_frame(0).unwrap();
    assert!(
        (200..2000).contains(&m.last_frame_steps()),
        "{}",
        m.last_frame_steps()
    );
    let e = m.render_frame(3).unwrap_err();
    assert_eq!((e.kind.as_str(), e.limit), ("limitexceeded", Some("steps")));
    assert!(e.hint.unwrap().contains("budget"));
    assert!(m.render_frame(4).is_ok());
}

#[test]
fn programs_cannot_write_files_or_read_outside_their_roots() {
    let dir = std::env::temp_dir().join(format!("psanim-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("out.txt");
    let e = load_err(&format!(
        "{HEAD}({}) (w) file closefile /scene {{ }} def",
        target.display()
    ));
    assert_eq!(e.kind, "invalidfileaccess");
    assert!(!target.exists());
    std::fs::write(&target, "secret").unwrap();
    let read = format!(
        "{HEAD}({}) (r) file closefile /scene {{ }} def",
        target.display()
    );
    // Open by default, as for any program you run yourself...
    assert!(Movie::load(&read, &Options::default()).is_ok());
    // ...and confined when the host says so; the built-in libraries still load.
    let confined = Options {
        read_roots: Some(vec![]),
        ..Options::default()
    };
    assert!(Movie::load(&read, &confined).is_err());
    assert!(
        Movie::load(
            &format!("{HEAD}(lib/artkit.ps) run /scene {{ }} def"),
            &confined
        )
        .is_ok()
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn interrupt_stops_a_render() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let stop = Arc::new(AtomicBool::new(false));
    let mut m = Movie::load(
        &format!("{HEAD}/scene {{ {{ }} loop }} def"),
        &Options {
            interrupt: Some(stop.clone()),
            ..Options::default()
        },
    )
    .unwrap();
    let s = stop.clone();
    let t = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(40));
        s.store(true, Ordering::Relaxed);
    });
    let e = m.render_frame(0).unwrap_err();
    t.join().unwrap();
    assert_eq!(e.kind, "interrupt");
}

#[test]
fn showpage_in_a_scene_is_harmless_and_the_wall_clock_is_the_movie_clock() {
    let mut m = load("/scene { 1 setgray 0 0 64 48 rectfill showpage } def");
    for k in 0..3 {
        assert_eq!(px(&m.render_frame(k).unwrap(), 5, 5)[0], 255);
    }
    assert_eq!(eval_at("", "realtime", 5), 500.0);
    assert_eq!(eval_at("", "usertime", 5), 500.0);
}

#[test]
fn a_font_set_in_setup_does_not_carry_and_the_error_says_so() {
    let mut m = load("/Helvetica findfont 10 scalefont setfont /scene { 5 5 moveto (x) show } def");
    let e = m.render_frame(0).unwrap_err();
    assert_eq!(e.kind, "invalidfont");
    assert!(e.hint.unwrap().contains("inside the scene"));
}

#[test]
fn pscat_libraries_work_inside_a_movie() {
    // artkit's palette and star, loaded in setup, used every frame.
    let mut m = load(
        "(lib/artkit.ps) run
         /scene { 32 24 translate now 90 mul rotate 1 0.8 0.2 setrgbcolor 0 0 18 8 5 star fill } def",
    );
    let a = m.render_frame(0).unwrap().premultiplied.to_vec();
    assert!(
        a.chunks(4).any(|p| p[0] > 200 && p[2] < 100),
        "a star was drawn"
    );
    assert_ne!(m.render_frame(3).unwrap().premultiplied, &a[..]);
}

/// How many pixels of row `y` have any ink.
fn lit_in_row(frame: &psanim::Frame, y: u32) -> usize {
    (0..frame.width)
        .filter(|&x| px(frame, x, y)[0] > 40)
        .count()
}

#[test]
fn onplane_facing_the_eye_is_plain_drawing() {
    let draw = "1 setgray 8 8 40 30 rectfill 2 setlinewidth newpath 32 24 10 0 360 arc stroke";
    let mut a = load(&format!("/scene {{ {draw} }} def"));
    let mut b = load(&format!("/scene {{ {{ {draw} }} << >> onplane }} def"));
    let (fa, fb) = (a.render_frame(0).unwrap(), b.render_frame(0).unwrap());
    let differ = fa
        .premultiplied
        .iter()
        .zip(fb.premultiplied)
        .filter(|(p, q)| p.abs_diff(**q) > 24)
        .count();
    // The stroke is outlined by strokepath and filled, so its edge pixels
    // may round differently; the shapes are the same.
    assert!(differ < 40, "{differ} channels differ");
}

#[test]
fn onplane_foreshortens_what_is_far() {
    // A square on a floor tilted away: its far edge is narrower and it is
    // squashed vertically. Rows are counted from the top of the frame.
    let mut m = load(
        "/scene { 32 4 translate
           { 1 setgray -20 0 40 40 rectfill } << /Tilt 70 /Distance 60 >> onplane } def",
    );
    let f = m.render_frame(0).unwrap();
    let near = lit_in_row(&f, 43);
    let far = (0..43)
        .rev()
        .map(|y| (y, lit_in_row(&f, y)))
        .take_while(|r| r.1 > 0)
        .last()
        .unwrap();
    assert!(near >= 38, "near edge {near}");
    assert!(far.1 * 3 < near * 2, "far edge {} vs near {near}", far.1);
    assert!(far.0 > 30, "squashed: the top is at row {}", far.0);
}

#[test]
fn onplane_leaves_defs_where_they_belong_and_the_built_ins_alone() {
    // A def inside lands in the caller's dictionary; afterwards fill is the
    // built-in again, even after nested planes.
    let v = eval_at(
        "",
        "{ /q 7 def { } << >> onplane userdict /fill known { /q q 100 add def } if } << >> onplane
         userdict /fill known { 1000 } { q } ifelse",
        0,
    );
    assert_eq!(v, 107.0);
}

#[test]
fn onplane_projects_text_strokes_and_clips() {
    let mut m = load(
        "/scene { 32 2 translate 1 setgray
           { /Helvetica-Bold findfont 14 scalefont setfont
             -28 0 moveto (AB) show (CD) show 2 0 (EF) ashow
             3 setlinewidth newpath -25 30 moveto 25 30 lineto stroke
             newpath -30 20 moveto 30 20 lineto 30 25 lineto -30 25 lineto closepath clip -30 0 60 60 rectfill
           } << /Tilt 50 /Distance 120 >> onplane } def",
    );
    let f = m.render_frame(0).unwrap_or_else(|e| panic!("{e}"));
    assert!(lit_in_row(&f, 40) > 20, "the text is drawn");
}

#[test]
fn onplane_cuts_away_what_is_behind_the_eye_and_fades_the_distance() {
    // The plane tipped flat: -y runs towards and past the eye.
    let mut m = load(
        "/scene { 32 24 translate 1 setgray
           { -20 -5000 40 5010 rectfill } << /Tilt 90 /Distance 50 /Eye [0 20] >> onplane } def",
    );
    let f = m.render_frame(0).unwrap_or_else(|e| panic!("{e}"));
    assert!(lit_in_row(&f, 47) > 0);
    // Past the end of the fade, nothing is drawn.
    let mut m = load(
        "/scene { 1 setgray { 0 0 64 48 rectfill } << /Depth 500 /Fade [100 200] >> onplane } def",
    );
    let f = m.render_frame(0).unwrap();
    assert!(f.premultiplied.chunks_exact(4).all(|p| p[0] < 10));
}

#[test]
fn planepoint_and_projectpath() {
    // Twice as far from the eye: half the size, about the eye point.
    let v = eval_at(
        "",
        "40 20 << /Depth 100 /Distance 100 >> planepoint pop exch pop",
        0,
    );
    assert_eq!(v, 10.0);
    let v = eval_at(
        "",
        "newpath 0 0 moveto 40 0 lineto 40 20 lineto closepath
         << /Depth 100 /Distance 100 >> projectpath pathbbox pop exch pop exch pop",
        0,
    );
    assert_eq!(v, 20.0);
}

#[test]
fn strings_written_by_a_scene_survive_the_rollback_and_are_found() {
    // `restore` does not roll string contents back: a scene that writes into
    // a string made in setup changes it for every later frame.
    let mut m = load(
        "/buf 4 string def buf 0 65 put
         /table << /name (abc) /rows [ (xy) (zw) ] /note (read only) >> def
         /scene { buf 0 buf 0 get 1 add put
                  table /rows get 1 get 0 66 put
                  /local 4 string def local 0 7 put } def",
    );
    let before = m.string_snapshot().unwrap().expect("a pure movie");
    assert!(
        before.len() >= 5,
        "buf, name, two rows, note: {}",
        before.len()
    );
    assert!(
        m.string_snapshot()
            .unwrap()
            .unwrap()
            .changed_since(&before)
            .is_empty()
    );
    m.render_frame(0).unwrap();
    m.render_frame(1).unwrap();
    let after = m.string_snapshot().unwrap().unwrap();
    assert_eq!(after.changed_since(&before), vec!["buf", "table/rows[1]"]);
}

#[test]
fn a_literal_string_in_a_procedure_is_found_when_a_scene_writes_into_it() {
    // A string literal in a procedure body is one string shared by every call:
    // the write below changes it for every later frame, so it is found, and so
    // is one in a procedure nested in another. A literal that is only read is not.
    let mut m = load(
        "/stamp { (abcd) dup 0 now cvi 65 add put pop } def
         /outer { { (wxyz) dup 1 90 put pop } exec (read only) pop } def
         /scene { stamp outer } def",
    );
    let before = m.string_snapshot().unwrap().expect("a pure movie");
    assert!(before.len() >= 3, "{}", before.len());
    m.render_frame(1).unwrap();
    let after = m.string_snapshot().unwrap().unwrap();
    assert_eq!(
        after.changed_since(&before),
        vec!["outer{0}{0}", "stamp{0}"]
    );
}

#[test]
fn only_a_pure_movie_has_a_string_snapshot() {
    let mut m = load("/step { } def /scene { } def");
    assert!(m.string_snapshot().unwrap().is_none());
}
