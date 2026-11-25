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
