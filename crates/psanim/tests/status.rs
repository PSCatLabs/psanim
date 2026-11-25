//! The status entries of the catalog: progress, test dots, pipeline and the
//! spinner's result.

use psanim::{Movie, Options};

/// A rendered frame's pixels, kept after its movie is gone.
struct Frame {
    premultiplied: Vec<u8>,
}

fn place(entry: &str, params: &str, at: f64) -> Frame {
    let src = format!(
        "<< /Size [400 300] /Duration 6 /FPS 10 /Background [0 0 0] >> animation
         /x /{entry} << /Params << {params} >> >> instance
         /scene {{ /x << /Box [0 0 400 300] /After /hold >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{entry}: {e}"));
    let f = m.render_at(at).unwrap_or_else(|e| panic!("{entry}: {e}"));
    Frame {
        premultiplied: f.premultiplied.to_vec(),
    }
}

fn lit(f: &Frame, pred: impl Fn(u8, u8, u8) -> bool) -> usize {
    f.premultiplied
        .chunks_exact(4)
        .filter(|p| pred(p[0], p[1], p[2]))
        .count()
}

#[test]
fn a_progress_bar_fills_by_value_and_goes_red_when_it_fails() {
    let at = |v: &str| {
        place(
            "progress",
            &format!("/Mode /value /Value {v} /Label false"),
            0.5,
        )
    };
    let blue = |f: &Frame| lit(f, |r, _, b| b > 150 && b > r + 60);
    let (a, b) = (blue(&at("0.25")), blue(&at("0.75")));
    assert!(b > a * 2, "{a} {b}");
    let failed = place(
        "progress",
        "/Mode /value /Value 0.75 /FailAt 0.5 /Label false",
        0.5,
    );
    assert!(lit(&failed, |r, g, _| r > 200 && g < 120) > 100);
    assert_eq!(blue(&failed), 0);
    let done = place("progress", "/Mode /value /Value 1 /Label false", 0.5);
    assert!(lit(&done, |r, g, _| g > 180 && r < 120) > 100);
    // By time: nothing yet at the start, full at the end.
    let t0 = place("progress", "/Seconds 2 /Label false", 0.0);
    let t1 = place("progress", "/Seconds 2 /Label false", 2.4);
    assert!(blue(&t1) > 10 * blue(&t0).max(1) || lit(&t1, |r, g, _| g > 180 && r < 120) > 100);
    // Indeterminate never fills the track.
    let ind = place("progress", "/Mode /indeterminate /Label false", 0.8);
    assert!(blue(&ind) > 50, "{}", blue(&ind));
}

#[test]
fn test_dots_count_greens_and_reds() {
    let dots = |v: &str| {
        place(
            "testdots",
            &format!("/Mode /value /Value {v} /Count 20 /Cols 10 /Fails [3 12] /Label false"),
            0.5,
        )
    };
    let green = |f: &Frame| lit(f, |r, g, _| g > 180 && r < 120);
    let red = |f: &Frame| lit(f, |r, g, _| r > 200 && g < 120);
    let half = dots("0.5");
    assert!(
        green(&half) > 100 && red(&half) > 30,
        "{} {}",
        green(&half),
        red(&half)
    );
    // Dot 12 has not come in at half way; once all are in there are two reds.
    let all = dots("1");
    assert!(
        red(&all) > red(&half) * 3 / 2,
        "{} {}",
        red(&all),
        red(&half)
    );
    assert!(green(&all) > green(&half));
    assert_eq!(red(&dots("0.1")), 0);
}

#[test]
fn a_pipeline_lights_its_stages_and_a_failure_stops_it() {
    let run = |p: &str, t: f64| place("pipeline", p, t);
    let green = |f: &Frame| lit(f, |r, g, _| g > 150 && r < 120);
    let red = |f: &Frame| lit(f, |r, g, _| r > 200 && g < 120);
    // By time: nothing passed at first, everything passed at the end.
    let times = "/Times [0.2 1 2 3]";
    assert_eq!(green(&run(times, 0.1)), 0);
    let end = run(times, 5.0);
    assert!(green(&end) > 500 && red(&end) == 0);
    // Midway, the first is green and the last is not yet.
    let mid = run(times, 1.6);
    assert!(
        green(&mid) > 100 && green(&mid) < green(&end),
        "{} {}",
        green(&mid),
        green(&end)
    );
    // A failure at the second stage: red appears, green stays at one stage's worth.
    let bad = run(&format!("{times} /Fail 1"), 5.0);
    assert!(red(&bad) > 100);
    assert!(green(&bad) < green(&end) * 2 / 3);
    // By value, a still.
    let still = run("/Value 2", 0.0);
    assert_eq!(still.premultiplied, run("/Value 2", 3.0).premultiplied);
    assert!(green(&still) > 100);
}

#[test]
fn a_spinner_can_come_to_a_result() {
    let green = |f: &Frame| lit(f, |r, g, b| g > 180 && r < 120 && b < 160);
    let red = |f: &Frame| lit(f, |r, g, _| r > 200 && g < 120);
    assert_eq!(green(&place("spinner", "/Resolve 1", 0.5)), 0);
    assert!(green(&place("spinner", "/Resolve 1", 2.0)) > 200);
    assert!(red(&place("spinner", "/Resolve 1 /Result /fail", 2.0)) > 200);
    // Without /Resolve it just loops, and keeps its own colour.
    assert_eq!(green(&place("spinner", "", 2.0)), 0);
}

#[test]
fn signal_driven_entries_follow_the_signal_and_stay_on_screen_past_their_own_clock() {
    // No /After: a signal-driven entry is never "finished", so it must not hide at 4 s or so.
    let src = |entry: &str, params: &str| {
        format!(
            "<< /Size [400 300] /Duration 8 /FPS 10 /Background [0 0 0] >> animation
             /x /{entry} << /Params << /Signal (level) {params} >> >> instance
             /scene {{ /x << /Box [0 0 400 300] >> place }} def"
        )
    };
    let count = |entry: &str, params: &str, t: f64, pred: fn(u8, u8, u8) -> bool| {
        // A pipeline's signal counts stages (0 to 3); the others run 0 to 1.
        let top = if entry == "pipeline" { 3 } else { 1 };
        let opts = Options {
            signals: psanim::Signals::from_csv(&format!("t,level\n0,0\n8,{top}\n")).unwrap(),
            ..Options::default()
        };
        let mut m = Movie::load(&src(entry, params), &opts).unwrap();
        let f = m.render_at(t).unwrap();
        f.premultiplied
            .chunks_exact(4)
            .filter(|p| pred(p[0], p[1], p[2]))
            .count()
    };
    let blue: fn(u8, u8, u8) -> bool = |r, _, b| b > 150 && b > r + 60;
    let green: fn(u8, u8, u8) -> bool = |r, g, b| g > 180 && r < 120 && b < 160;
    let (a, b) = (
        count("progress", "/Label false", 1.5, blue),
        count("progress", "/Label false", 6.0, blue),
    );
    assert!(a > 50 && b > a * 2, "bar {a} {b}");
    let (a, b) = (
        count("testdots", "/Label false /Fails []", 1.5, green),
        count("testdots", "/Label false /Fails []", 6.0, green),
    );
    assert!(b > a * 2 && b > 500, "dots {a} {b}");
    // And a pipeline, whose signal counts stages.
    let p = count("pipeline", "", 6.0, green);
    assert!(p > 100, "pipeline {p}");
}
