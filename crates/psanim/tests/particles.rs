//! Particles: every particle a function of its birth and the time.

use psanim::{Movie, Options};

const HEAD: &str = "<< /Size [400 300] /Duration 6 /FPS 10 /Background [0 0 0] >> animation\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

/// What the scene prints (with `=` and `==`) at time `t`.
fn printed(m: &mut Movie, t: f64) -> String {
    m.take_output();
    m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

/// A scene that prints every live particle's state, one per line.
fn dump(spec: &str) -> String {
    format!(
        "/spec {spec} def
         /scene {{ spec {{ [particlepos particlevel particleage particleu particlekey
                           particleemitter particleburst] == }} particles }} def"
    )
}

fn count(spec: &str, t: f64) -> usize {
    let mut m = load(&dump(spec));
    let out = printed(&mut m, t);
    out.lines().filter(|l| !l.is_empty()).count()
}

const FOUNTAIN: &str = "<< /At [200 20] /Rate 40 /Life [1 2] /Speed [80 140] /Spread 40
    /Gravity 90 /Drag 0.7 /Gust [60 10] /Sway [8 0] >>";

#[test]
fn a_frame_is_the_same_whatever_was_rendered_before() {
    let mut a = load(&dump(FOUNTAIN));
    let mut b = load(&dump(FOUNTAIN));
    let want = printed(&mut a, 3.3);
    assert!(want.lines().count() > 20, "{want}");
    for t in [5.9, 0.4, 2.0, 3.3] {
        printed(&mut b, t);
    }
    assert_eq!(printed(&mut b, 3.3), want);
    // And the pixels too.
    let px = a.render_at(4.1).unwrap().premultiplied.to_vec();
    b.render_at(0.2).unwrap();
    assert_eq!(b.render_at(4.1).unwrap().premultiplied, &px[..]);
}

#[test]
fn a_burst_emits_its_count_and_they_die_at_the_end_of_their_lives() {
    let spec = "<< /Bursts [1 2] /Count 37 /Life 0.8 >>";
    assert_eq!(count(spec, 0.9), 0);
    assert_eq!(count(spec, 1.0), 37);
    assert_eq!(count(spec, 1.7), 37);
    assert_eq!(count(spec, 1.9), 0);
    assert_eq!(count(spec, 2.5), 37);
    // Bursts are numbered, continuous emission is -1, emitters counted.
    let mut m = load(&dump(&format!("[ << /Rate 5 /Life 3 >> {spec} ]")));
    let out = printed(&mut m, 2.3);
    assert!(out.lines().any(|l| l.ends_with(" 1 1]")), "{out}");
    assert!(out.lines().any(|l| l.ends_with(" 0 -1]")), "{out}");
}

#[test]
fn continuous_emission_holds_steady_and_stops() {
    let spec = "<< /Rate 50 /Life 1 /From 1 /To 3 >>";
    assert_eq!(count(spec, 0.5), 0);
    for t in [2.0, 2.5, 2.99] {
        let n = count(spec, t);
        assert!((40..=60).contains(&n), "{n} alive at {t}");
    }
    assert_eq!(count(spec, 4.1), 0);
    // Only the live ones are visited: a long film costs no more late on.
    let spec = "<< /Rate 50 /Life 1 /From -100000 >>";
    let n = count(spec, 5.0);
    assert!((40..=60).contains(&n), "{n}");
}

#[test]
fn a_particle_flies_by_the_closed_form() {
    // One particle straight up at 100 units/s under gravity 50, no drag:
    // y = 100 t - 25 t^2 above where it started.
    let mut m = load(&dump(
        "<< /At [10 20] /Bursts [0] /Count 1 /Life 5 /Speed 100 /Spread 0 /Gravity 50 >>",
    ));
    let out = printed(&mut m, 2.0);
    let v: Vec<f64> = out
        .trim_matches(|c| c == '[' || c == ']')
        .split_whitespace()
        .map(|x| x.parse().unwrap())
        .collect();
    assert!((v[0] - 10.0).abs() < 1e-3, "{out}");
    assert!((v[1] - (20.0 + 200.0 - 100.0)).abs() < 1e-3, "{out}");
    assert!(v[2].abs() < 1e-3 && v[3].abs() < 1e-3, "{out}");
    assert!(
        (v[4] - 2.0).abs() < 1e-9 && (v[5] - 0.4).abs() < 1e-9,
        "{out}"
    );
}

#[test]
fn the_seed_changes_the_particles() {
    let spec = "<< /Bursts [0] /Count 5 /Life 3 /Speed [10 200] >>";
    let src = dump(spec);
    let a = printed(&mut load(&src), 1.0);
    let b = printed(
        &mut Movie::load(
            &format!("<< /Size [400 300] /Duration 6 /FPS 10 /Seed 9 >> animation\n{src}"),
            &Options::default(),
        )
        .unwrap(),
        1.0,
    );
    assert_ne!(a, b);
    // So does an emitter's own /Seed.
    let c = printed(
        &mut load(&dump(
            "<< /Bursts [0] /Count 5 /Life 3 /Speed [10 200] /Seed 4 >>",
        )),
        1.0,
    );
    assert_ne!(a, c);
}

#[test]
fn a_period_makes_emission_repeat() {
    let spec = "<< /Rate 30 /Life [1 2] /Period 2 /Speed 50 /Gust [20 0] /Sway [5 5] >>";
    let mut m = load(&dump(spec));
    let a = printed(&mut m, 1.3);
    let b = printed(&mut m, 3.3);
    let c = printed(&mut m, 5.3);
    // The same particles, born a period apart: positions agree (keys too).
    let strip = |s: &str| {
        s.lines()
            .map(|l| {
                l.split_whitespace()
                    .take(7)
                    .map(|x| format!("{:.2}", x.trim_matches('[').parse::<f64>().unwrap()))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(strip(&b), strip(&c));
    assert_eq!(strip(&a), strip(&b));
}

#[test]
fn the_procedure_sees_the_callers_stack_and_no_dictionary() {
    let mut m = load(
        "/scene { 7 << /Bursts [0] /Count 3 >> { dup = countdictstack = } particles pop } def",
    );
    let out = printed(&mut m, 0.5);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 6, "{out}");
    assert_eq!(lines[0], "7");
    // The same depth as the scene itself has.
    let mut plain = load("/scene { countdictstack = } def");
    assert_eq!(lines[1], printed(&mut plain, 0.5));
}

#[test]
fn an_emitters_own_draw_and_the_trail() {
    let mut m = load(
        "/scene {
           [ << /Bursts [0] /Count 2 /Draw { (a) = } >> << /Bursts [0] /Count 1 >> ]
           { (b) = } particles
           << /Bursts [0] /Count 1 /Speed 100 /Spread 0 /Angle 0 >>
           { 0.5 5 particletrail pathbbox 4 array astore == } particles
         } def",
    );
    let out = printed(&mut m, 1.0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(&lines[..3], &["a", "a", "b"]);
    // A track half a second long at 100 units/s, behind the particle.
    assert_eq!(lines[3], "[-50.0 0.0 0.0 0.0]");
}

#[test]
fn reading_a_particle_outside_particles_fails_with_a_hint() {
    let err = match Movie::load(
        &format!("{HEAD}/scene {{ particleage pop }} def"),
        &Options::default(),
    ) {
        Ok(mut m) => format!("{}", m.render_at(0.0).unwrap_err()),
        Err(e) => format!("{e}"),
    };
    assert!(
        err.contains("inside the procedure given to `particles`"),
        "{err}"
    );
}

#[test]
fn sincelast_reads_an_array_of_times() {
    let mut m = load("/scene { [1 2.5 4] sincelast = } def");
    assert_eq!(printed(&mut m, 0.5), "1000000.0");
    assert_eq!(printed(&mut m, 3.0), "0.5");
    let mut m = load("/scene { 2 { [1 2.5 4] sincelast = } delayed } def");
    assert_eq!(printed(&mut m, 3.5), "0.5");
}

#[test]
fn a_gust_over_an_enormous_life_is_refused_not_followed() {
    // A billion-second life with nothing emitted: nothing to tabulate.
    let mut m = psanim::Movie::load(
        "<< /Size [64 48] /Duration 2 /FPS 10 >> animation
         /scene { << /Life 1000000000 /Rate 0 /Gust [1 1] >> { } particles } def",
        &psanim::Options::default(),
    )
    .unwrap();
    m.render_at(1.0).unwrap();
    // Emitting since time 0: only the second of life so far is followed.
    let mut m = psanim::Movie::load(
        "<< /Size [64 48] /Duration 2 /FPS 10 >> animation
         /scene { << /Life 3000 /Rate 5 /Gust [1 1] >> { } particles } def",
        &psanim::Options::default(),
    )
    .unwrap();
    m.render_at(1.0).unwrap();
    // A burst a long time before now, still alive: refused with a reason.
    let mut m = psanim::Movie::load(
        "<< /Size [64 48] /Duration 2 /FPS 10 >> animation
         /scene { << /Life 1000000000 /Bursts [-100000000] /Count 1 /Gust [1 1] >> { } particles } def",
        &psanim::Options::default(),
    )
    .unwrap();
    let e = m.render_at(1.0).unwrap_err().to_string();
    assert!(e.contains("too long to follow"), "{e}");
}

#[test]
fn an_enormous_time_never_hangs_a_stream() {
    // A stream that stopped at 1 s: at 1e20 s there is simply nothing.
    let mut m = psanim::Movie::load(
        "<< /Size [64 48] /Duration 2 /FPS 10 >> animation
         /scene { 1e20 { << /Rate 1 /To 1 >> { } particles } attime } def",
        &psanim::Options::default(),
    )
    .unwrap();
    m.render_at(0.5).unwrap();
    // A stream that never stops: refused, not looped over.
    let mut m = psanim::Movie::load(
        "<< /Size [64 48] /Duration 2 /FPS 10 >> animation
         /scene { 1e20 { << /Rate 1 /Life 2 >> { } particles } attime } def",
        &psanim::Options::default(),
    )
    .unwrap();
    let e = m.render_at(0.5).unwrap_err().to_string();
    assert!(e.contains("too large"), "{e}");
}
