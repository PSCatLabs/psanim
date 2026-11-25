//! Components: movies defined on their own and placed inside a movie.

use psanim::{Movie, Options};

const HEAD: &str = "<< /Size [200 100] /Duration 4 /FPS 10 /Background [0 0 0] >> animation\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

fn load_err(body: &str) -> String {
    match Movie::load(&format!("{HEAD}{body}"), &Options::default()) {
        Ok(_) => panic!("expected an error"),
        Err(e) => format!("{e}"),
    }
}

/// What the scene prints (with `==`) at time `t`.
fn printed(m: &mut Movie, t: f64) -> String {
    m.take_output();
    m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

/// A component that prints what it sees: its clock, size and parameters.
const PROBE: &str = "/probe << /Size [50 40] /Duration 2
    /Setup { /Label (default) param /Label exch def /secret 42 def }
    /Scene { [ now globaltime duration width height Label movieseed ] == } >> component\n";

#[test]
fn a_component_runs_on_its_own_clock_size_and_seed() {
    let mut m = load(&format!(
        "{PROBE}/scene {{ /probe << /At 1 /Box [0 0 100 80] >> place }} def"
    ));
    assert_eq!(printed(&mut m, 0.5), "");
    assert_eq!(printed(&mut m, 1.5), "[0.5 0.5 2.0 50.0 40.0 (default) 1]");
    // After its two seconds it is gone: it does not loop unless it says so.
    assert_eq!(printed(&mut m, 3.5), "");
}

#[test]
fn speed_hold_and_loop() {
    let mut m = load(&format!(
        "{PROBE}/scene {{ /probe << /Speed 2 /After /hold >> place }} def"
    ));
    assert!(printed(&mut m, 0.5).starts_with("[1.0 1.0 "));
    assert!(printed(&mut m, 3.0).starts_with("[2.0 2.0 "));
    let mut m = load(&format!(
        "{PROBE}/scene {{ /probe << /After /loop /At 1 /Before /hold >> place }} def"
    ));
    assert!(printed(&mut m, 0.5).starts_with("[0.0 0.0 "));
    assert!(printed(&mut m, 3.5).starts_with("[0.5 0.5 "));
}

#[test]
fn instances_have_their_own_params_and_seed_and_ignore_the_hosts() {
    let mut opts = Options::default();
    opts.params
        .insert("Label".into(), psanim::Param::Text("host".into()));
    let src = format!(
        "{HEAD}{PROBE}/other /probe << /Seed 7 /Params << /Label (mine) >> >> instance
         /Label (default) param ==
         /scene {{ /probe << >> place /other << >> place }} def"
    );
    let mut m = Movie::load(&src, &opts).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.take_output().trim(), "(host)");
    let out = printed(&mut m, 0.0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "[0.0 0.0 2.0 50.0 40.0 (default) 1]");
    assert_eq!(lines[1], "[0.0 0.0 2.0 50.0 40.0 (mine) 7]");
}

#[test]
fn a_components_names_stay_its_own() {
    let mut m = load(&format!(
        "/secret 1 def {PROBE}/scene {{ secret == /probe << >> place }} def"
    ));
    let out = printed(&mut m, 0.0);
    assert!(out.starts_with("1\n"), "{out}");
    // The host never sees the component's names.
    let e = load_err(&format!("{PROBE}Label =="));
    assert!(e.contains("Label"), "{e}");
}

#[test]
fn a_component_changing_the_theme_keeps_it_to_itself() {
    let mut m = load(
        "/pkstage use /midnight usetheme
         /lit << /Setup { /pkstage use /paper usetheme } /Scene { /bg rgbof 3 array astore == } >> component
         /scene { /bg rgbof 3 array astore == /lit << >> place /bg rgbof 3 array astore == } def",
    );
    let out = printed(&mut m, 0.0);
    let l: Vec<&str> = out.lines().collect();
    assert_eq!(l[0], l[2], "{out}");
    assert_ne!(l[0], l[1], "{out}");
}

#[test]
fn randomness_follows_the_instance_seed() {
    let mut m = load(
        "/r << /Seed 3 /Scene { 0.5 hash01 == 1 0.7 0.5 wiggle == } >> component
         /r2 /r << /Seed 4 >> instance /r3 /r << /Seed 3 >> instance
         /scene { /r << >> place /r2 << >> place /r3 << >> place } def",
    );
    let out = printed(&mut m, 0.3);
    let l: Vec<&str> = out.lines().collect();
    assert_eq!(l.len(), 6);
    assert_eq!(l[0..2], l[4..6]);
    assert_ne!(l[0], l[2]);
}

#[test]
fn a_component_draws_scaled_into_its_box() {
    let mut m = load(
        "/sq << /Size [10 10] /Scene { 1 0 0 setrgbcolor 0 0 10 10 rectfill } >> component
         /scene { /sq << /Box [100 0 100 50] >> place } def",
    );
    let f = m.render_at(0.0).unwrap();
    let red = |x: u32, y: u32| f.premultiplied[((y * f.width + x) * 4) as usize];
    // Fitted, centred: a 50x50 square from x = 125 to 175, at the bottom
    // of the canvas (rows 50..100 from the top).
    assert_eq!(red(150, 75), 255);
    assert_eq!(red(120, 75), 0);
    assert_eq!(red(150, 25), 0);
}

#[test]
fn a_movie_file_is_a_component_read_beside_the_movie() {
    let dir = std::env::temp_dir().join(format!("psanim-comp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("blink.ps"),
        "<< /Size [20 20] /Duration 1 /Loop true >> animation
         /Hue 0.5 param /hue exch def
         /scene { [ hue progress ] == } def",
    )
    .unwrap();
    let opts = Options {
        base_dir: Some(dir.clone()),
        ..Options::default()
    };
    let src = format!(
        "{HEAD}/b (blink.ps) component /b2 /b << /Params << /Hue 0.9 >> >> instance
         /scene {{ /b << >> place /b2 << /At 0.5 >> place }} def"
    );
    let mut m = Movie::load(&src, &opts).unwrap_or_else(|e| panic!("{e}"));
    // Loops because the file declared /Loop true.
    assert_eq!(printed(&mut m, 2.25), "[0.5 0.25]\n[0.9 0.75]");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn mistakes_are_explained() {
    let mut m = load("/scene { /nope << >> place } def");
    let e = m.render_at(0.0).unwrap_err().to_string();
    assert!(e.contains("no component named /nope"), "{e}");
    let e = load_err("/p << /Persist true /Scene { } >> component");
    assert!(e.contains("cannot /Persist"), "{e}");
    let e = load_err("/p << /Setup { /x 1 def } >> component");
    assert!(e.contains("must define its scene"), "{e}");
}

#[test]
fn blur_inside_a_component_scales_with_its_box() {
    // A 10-unit square blurred by 2 units, drawn at 1:1 and at 4x: the
    // soft edge is four times as wide at 4x.
    let mut m = load(
        "/b << /Size [20 20] /Scene { 2 { 1 1 1 setrgbcolor 5 5 10 10 rectfill } blurred } >> component
         /scene { /b << /Box [0 0 20 20] >> place /b << /Box [100 0 80 80] >> place } def",
    );
    let f = m.render_at(0.0).unwrap();
    let alpha = |x: u32, y: u32| f.premultiplied[((y * f.width + x) * 4) as usize];
    // Width of the ramp along a row through each square's middle.
    let ramp = |y: u32, x0: u32, x1: u32| {
        (x0..x1)
            .filter(|&x| (8..248).contains(&alpha(x, y)))
            .count()
    };
    let small = ramp(100 - 10, 0, 20);
    let big = ramp(100 - 40, 100, 180);
    assert!(small >= 1, "small ramp {small}");
    assert!(big >= 3 * small, "small {small}, big {big}");
}

#[test]
fn blur_glow_and_grain_inside_a_component_stay_in_its_box() {
    let mut m = load(
        "/g << /Size [40 40] /Scene { 6 1.5 { 1 1 1 setrgbcolor 0 0 40 40 rectfill } glowing
                                      8 { 1 0 0 setrgbcolor 30 30 10 10 rectfill } blurred
                                      0.5 filmgrain 0.8 vignette } >> component
         /scene { 0 0 0.3 setrgbcolor 0 0 200 100 rectfill /g << /Box [80 30 40 40] >> place } def",
    );
    let f = m.render_at(0.0).unwrap();
    let w = f.width as usize;
    for (k, p) in f.premultiplied.chunks_exact(4).enumerate() {
        let (x, y) = (k % w, k / w);
        let inside = (80..120).contains(&x) && (30..70).contains(&y);
        if !inside {
            assert_eq!(
                &p[..3],
                &[0, 0, 77],
                "pixel {x},{y} touched outside the box"
            );
        }
    }
}

#[test]
fn a_movie_redefining_a_word_does_not_change_it_inside_a_component() {
    // The movie defines scale, between and rgbof for itself; the component
    // still means PostScript's scale, the prelude's between and pkstage's rgbof.
    let mut m = load(
        "/pkstage use
         /scale { pop pop (movie) == } def
         /between { pop pop pop 99 } def
         /rgbof { pop 1 2 3 } def
         /c << /Size [10 10] /Scene { 2 2 scale 0.5 0 10 between == /accent rgbof pop pop == } >> component
         /scene { /c << >> place } def",
    );
    let out = printed(&mut m, 0.0);
    let l: Vec<&str> = out.lines().collect();
    assert_eq!(l[0], "5.0", "{out}");
    // pkstage's midnight accent, not the movie's 1 2 3.
    assert_eq!(l[1], "1.0", "{out}");
    assert!(!out.contains("movie"), "{out}");
}
