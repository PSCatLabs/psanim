//! The clock entries of the catalog: the time comes from the host (a start
//! and the movie's own time, or a signal), never from the wall clock.

use psanim::{Movie, Options, Param};

const CLOCKS: &[&str] = &[
    "clock-analog",
    "clock-digital",
    "clock-ledsign",
    "clock-morph",
];

fn load(name: &str, start: Option<Param>) -> Movie {
    let src = psanim::catalog::source(name).unwrap();
    let mut opts = Options::default();
    // The sign scrolls with the movie's time, which these tests move.
    opts.params.insert("Scroll".into(), Param::Number(0.0));
    if let Some(p) = start {
        opts.params.insert("Start".into(), p);
    }
    Movie::load(src, &opts).unwrap_or_else(|e| panic!("/{name}: {e}"))
}

fn at(m: &mut Movie, t: f64) -> Vec<u8> {
    m.render_at(t).unwrap().premultiplied.to_vec()
}

#[test]
fn a_clock_is_a_pure_function_of_start_and_movie_time() {
    for name in CLOCKS {
        let mut m = load(name, Some(Param::parse("12:34:56")));
        let t0 = at(&mut m, 0.0);
        // The same moment twice is the same picture (nothing reads a clock).
        assert_eq!(t0, at(&mut m, 0.0), "/{name} is not repeatable");
        // Time passes: ten seconds on, the picture is different.
        assert_ne!(t0, at(&mut m, 10.0), "/{name} does not show time passing");
        // A start ten seconds later at movie time 0 is the clock ten seconds on.
        let mut later = load(name, Some(Param::parse("12:35:06")));
        assert_eq!(
            at(&mut later, 0.0),
            at(&mut m, 10.0),
            "/{name}: start + time"
        );
        // The start as seconds is the start as text.
        let mut secs = load(name, Some(Param::Number(45_296.0)));
        assert_eq!(at(&mut secs, 0.0), t0, "/{name}: seconds and text agree");
    }
}

#[test]
fn a_signal_drives_a_clock_the_way_a_start_does() {
    for name in CLOCKS {
        let src = psanim::catalog::source(name).unwrap();
        let mut opts = Options::default();
        opts.params.insert("Scroll".into(), Param::Number(0.0));
        opts.params.insert("Signal".into(), Param::parse("clock"));
        let mut live = Movie::load(src, &opts).unwrap();
        live.set_signal("clock", 45_296.0);
        let mut started = load(name, Some(Param::parse("12:34:56")));
        // The movie time does not matter in signal mode: the page sets the time.
        assert_eq!(at(&mut live, 7.0), at(&mut started, 0.0), "/{name}");
        live.set_signal("clock", 45_300.5);
        let mut later = load(name, Some(Param::Number(45_300.5)));
        assert_eq!(at(&mut live, 7.0), at(&mut later, 0.0), "/{name}");
    }
}

#[test]
fn the_morphing_clock_melts_into_the_next_digit_and_is_whole_on_the_second() {
    let mut m = load("clock-morph", Some(Param::parse("00:00:07")));
    let whole = at(&mut m, 0.0);
    // Inside the melt, before the second ends, the last digit is between 7 and 8.
    let melting = at(&mut m, 0.8);
    assert_ne!(melting, whole);
    let mut eight = load("clock-morph", Some(Param::parse("00:00:08")));
    let next = at(&mut eight, 0.0);
    assert_ne!(melting, next);
    // On the stroke of the second the new digit is whole (the polygon drawn
    // at u = 1 is the outline, which differs from the pen stroke only at the
    // edge pixels), so it is closer to the 8 than the melting frame was.
    let diff = |a: &[u8], b: &[u8]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| x.abs_diff(*y) as u64)
            .sum::<u64>()
    };
    let almost = at(&mut m, 0.999);
    assert!(diff(&almost, &next) < diff(&melting, &next) / 4);
}

#[test]
fn every_theme_draws_every_clock() {
    for name in CLOCKS {
        let src = psanim::catalog::source(name).unwrap();
        let mut seen = Vec::new();
        for theme in ["midnight", "amber", "paper", "phosphor"] {
            let mut opts = Options::default();
            opts.params.insert("Theme".into(), Param::parse(theme));
            let mut m = Movie::load(src, &opts).unwrap_or_else(|e| panic!("/{name} {theme}: {e}"));
            let px = at(&mut m, 1.0);
            assert!(
                px.chunks_exact(4).any(|p| p[3] > 0),
                "/{name} {theme} is empty"
            );
            assert!(
                !seen.contains(&px),
                "/{name}: {theme} looks like another theme"
            );
            seen.push(px);
        }
    }
}
