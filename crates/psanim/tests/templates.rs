//! The explainer templates in `skills/psanim/templates/`: `poem.ps` and
//! `process.ps`, and the test contents beside them (`poem-*.ps`,
//! `process-*.ps`, each of which places its template as a component with
//! other content in `/Params`). Every one must load, draw something in
//! every frame once it has started, and end on a still picture; content
//! past a template's limits must fail with a message that names the field.

use std::path::{Path, PathBuf};

use psanim::{Movie, Options, Scale, raster};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills/psanim/templates")
        .canonicalize()
        .unwrap()
}

fn options() -> Options {
    let dir = dir();
    Options {
        scale: Scale::Width(160),
        read_roots: Some(vec![dir.clone()]),
        base_dir: Some(dir),
        ..Options::default()
    }
}

fn load(source: &str) -> Result<Movie, String> {
    Movie::load(source, &options()).map_err(|e| e.to_string())
}

/// The fraction of a frame that is not its most common colour.
fn cover(m: &mut Movie, k: u64) -> Result<f64, String> {
    let meta = m.meta().clone();
    let matte = meta
        .background
        .map_or([0, 0, 0], |c| c.map(|v| (v * 255.0).round() as u8));
    let f = m.render_frame(k).map_err(|e| e.to_string())?;
    let mut flat = Vec::new();
    raster::flatten_onto(f.premultiplied, matte, &mut flat);
    Ok(raster::stats(&flat, meta.pixel_width as usize, meta.pixel_height as usize).coverage)
}

fn pixels(m: &mut Movie, k: u64) -> Result<Vec<u8>, String> {
    Ok(m.render_frame(k)
        .map_err(|e| e.to_string())?
        .premultiplied
        .to_vec())
}

/// What `psanim check` would object to, short of the scan for pops (which
/// lives in the command-line tool; `psanim check --all` was run on every
/// file by hand).
fn problems(name: &str, source: &str) -> Vec<String> {
    let mut m = match load(source) {
        Ok(m) => m,
        Err(e) => return vec![format!("{name}: {e}")],
    };
    let meta = m.meta().clone();
    let mut out = Vec::new();
    if !meta.shadowed.is_empty() {
        out.push(format!(
            "{name}: redefines psanim words {:?}",
            meta.shadowed
        ));
    }
    if meta.duration < 10.0 {
        out.push(format!("{name}: only {} s long", meta.duration));
    }
    // Errors (a stack left with values is one) and blank frames: once the
    // film has shown something, every sampled frame must show something.
    // Every frame of the first three seconds (where a short heading alone
    // can leave the frame nearly empty), then a spread of the rest.
    let opening = (3.0 * meta.fps) as u64;
    let mut picks: Vec<u64> = (0..opening.min(meta.frames)).collect();
    picks.extend(
        psanim::sheet::spread(meta.frames, 60)
            .into_iter()
            .filter(|&k| k >= opening),
    );
    let mut started = false;
    for k in picks {
        match cover(&mut m, k) {
            Err(e) => out.push(format!("{name}: frame {k}: {e}")),
            Ok(c) if c >= 0.002 => started = true,
            Ok(_) if started => out.push(format!("{name}: frame {k} is blank")),
            Ok(_) => {}
        }
    }
    // It ends on the finished picture: the last second does not change.
    let last = meta.frames - 1;
    let before = last - meta.fps.round() as u64;
    match (pixels(&mut m, before), pixels(&mut m, last)) {
        (Ok(a), Ok(b)) if a != b => out.push(format!(
            "{name}: the last second still changes (frames {before} and {last} differ)"
        )),
        (Err(e), _) | (_, Err(e)) => out.push(format!("{name}: {e}")),
        _ => {}
    }
    out
}

fn files(prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|x| x == "ps")
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s == prefix || s.starts_with(&format!("{prefix}-")))
        })
        .collect();
    v.sort();
    v
}

fn check_all(prefix: &str) {
    let paths = files(prefix);
    assert!(
        paths.len() >= 5,
        "{prefix}: the template and at least four test contents, found {paths:?}"
    );
    let mut failures = Vec::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        failures.extend(problems(&name, &std::fs::read_to_string(p).unwrap()));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_poem_template_and_its_test_contents_play_clean() {
    check_all("poem");
}

#[test]
fn the_process_template_and_its_test_contents_play_clean() {
    check_all("process");
}

fn duration(file: &str) -> f64 {
    let source = std::fs::read_to_string(dir().join(file)).unwrap();
    load(&source).unwrap().meta().duration
}

#[test]
fn more_content_makes_a_longer_film() {
    assert!(duration("poem-long.ps") > duration("poem-buson.ps") + 20.0);
    assert!(duration("process-parcel.ps") > duration("process-tea.ps") + 20.0);
}

/// Load `template` with the given /Params and return its error message.
fn refused(template: &str, params: &str) -> String {
    let source = format!(
        "/tpl ({template}) component\n/film /tpl << /Params << {params} >> >> instance\n\
         << /Size [640 360] /Duration /film componentduration >> animation\n\
         /scene {{ /film << >> place }} def\n"
    );
    match load(&source) {
        Ok(_) => panic!("{template} accepted {params}"),
        Err(e) => e,
    }
}

#[test]
fn content_past_a_limit_fails_with_a_message_naming_the_field() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "poem.ps",
            "/poem [ (a) (b) (c) (d) (e) (f) (g) ]",
            "/poem has 7 lines; this template takes 2 to 6",
        ),
        (
            "poem.ps",
            "/poem [ (just one line) ]",
            "/poem has 1 lines; this template takes 2 to 6",
        ),
        (
            "poem.ps",
            "/notes [ ]",
            "/notes has 0 notes; this template takes 1 to 5",
        ),
        (
            "poem.ps",
            "/notes [ [ 0 (heron) (image) (not in the line) ] ]",
            "note 0: the words (heron) are not in line 0 of /poem",
        ),
        (
            "poem.ps",
            "/notes [ [ 9 (snail) (image) (no such line) ] ]",
            "note 0: line must be a number from 0 to 2",
        ),
        (
            "poem.ps",
            "/poem [ (a line so long that no legible size of type could ever set it inside the column of the poem) (two) ] \
             /notes [ [ 1 (two) (x) (y) ] ]",
            "a line of /poem is too long to set at a legible size",
        ),
        (
            "poem.ps",
            "/reading (one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen \
             seventeen eighteen nineteen twenty, and then all of it again: one two three four five six seven eight nine \
             ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty, and then all of it \
             again, one more time, slowly: one two three four five six seven eight nine ten eleven twelve thirteen.)",
            "/reading is too long for the card",
        ),
        (
            "process.ps",
            "/steps [ [(a) (x)] [(b) (x)] ]",
            "/steps has 2 steps; this template takes 3 to 8",
        ),
        (
            "process.ps",
            "/steps [ [(a) (x)] [(b) (x)] [(c) (x)] [(d) (x)] [(e) (x)] [(f) (x)] [(g) (x)] [(h) (x)] [(i) (x)] ]",
            "/steps has 9 steps; this template takes 3 to 8",
        ),
        (
            "process.ps",
            "/key 7",
            "/key must be a step number from 0 to 4",
        ),
        (
            "process.ps",
            "/pathback [ 1 1 (again) (the same step) ]",
            "/pathback: from and to must be different steps",
        ),
        (
            "process.ps",
            "/steps [ [(a) (x)] [(b) (x)] [(a step name that is far too long) (x)] ]",
            "a step name is too long",
        ),
        (
            "process.ps",
            "/summary (a closing line that goes on and on, far past the width of the frame and then further still, until no legible size will hold it on one line)",
            "/summary is too long for one line",
        ),
        (
            "process.ps",
            "/why (a reason that goes on and on, far past the width of the column beside the steps and onto a second line, \
             and then a third line, which this template does not allow)",
            "/why is too long for two lines beside the steps",
        ),
        (
            "process.ps",
            "/pathback [ 2 0 (back) (a line for the path back that goes on and on, far past the width of the frame and then further still, until no legible size will hold it) ]",
            "/pathback: what happens is too long for one line",
        ),
    ];
    for (template, params, message) in cases {
        let e = refused(template, params);
        assert!(
            e.contains(&format!("content: {message}")),
            "{template} with {params}: expected `{message}`, got:\n{e}"
        );
    }
}

/// Load `template` placed as a component with the given /Params.
fn with_params(template: &str, params: &str) -> Result<Movie, String> {
    load(&format!(
        "/tpl ({template}) component\n/film /tpl << /Params << {params} >> >> instance\n\
         << /Size [640 360] /Duration /film componentduration >> animation\n\
         /scene {{ /film << >> place }} def\n"
    ))
}

fn template_source(template: &str) -> String {
    std::fs::read_to_string(dir().join(template)).unwrap()
}

/// The template run on its own, with `extra` put just before the line
/// that ends its CONTENT block: what an author who edits the copy gets.
fn standalone(template: &str, extra: &str) -> Result<Movie, String> {
    let source = template_source(template);
    let marker = "% ===================== nothing below here needs to change";
    assert!(
        source.contains(marker),
        "{template}: no end of CONTENT block"
    );
    load(&source.replacen(marker, &format!("{extra}\n{marker}"), 1))
}

fn expect_refused(result: Result<Movie, String>, what: &str, message: &str) {
    match result {
        Ok(_) => panic!("{what}: accepted"),
        Err(e) => assert!(
            e.contains(&format!("content: {message}")),
            "{what}: expected `content: {message}`, got:\n{e}"
        ),
    }
}

const THEMES: [&str; 6] = [
    "/paper",
    "/midnight",
    "/dusk",
    "/blueprint",
    "/poster",
    "/terminal",
];

#[test]
fn a_film_longer_than_90_seconds_is_refused() {
    let many = "a ".repeat(75);
    let notes: String = (0..5)
        .map(|_| format!("[ 0 (little snail) (image) ({many}) ] "))
        .collect();
    expect_refused(
        with_params("poem.ps", &format!("/notes [ {notes}]")),
        "five notes of 75 words",
        "the film would run",
    );
    let lines: String = (0..8)
        .map(|i| format!("[ (s{i}) ({}) ] ", "a ".repeat(30)))
        .collect();
    expect_refused(
        with_params(
            "process.ps",
            &format!(
                "/steps [ {lines}] /why ({many}) /summary ({})",
                "a ".repeat(30)
            ),
        ),
        "eight steps of 30 words",
        "the film would run",
    );
}

#[test]
fn an_unbroken_word_wider_than_the_card_is_refused() {
    let w = |n| "W".repeat(n);
    expect_refused(
        with_params("poem.ps", &format!("/reading ({})", w(200))),
        "a 200-W reading",
        "/reading has a word too long",
    );
    expect_refused(
        with_params(
            "poem.ps",
            &format!("/notes [ [ 0 (little snail) (image) ({}) ] ]", w(160)),
        ),
        "a 160-W note",
        "note 0: what it does has a word too long",
    );
}

/// The names a template reads from /Params, from its `/x /x x param def` lines.
fn content_fields(template: &str) -> Vec<String> {
    template_source(template)
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let name = l.strip_prefix('/')?.split_whitespace().next()?;
            (l.ends_with(" param def") && l.starts_with(&format!("/{name} /{name} {name} param")))
                .then(|| name.to_string())
        })
        .collect()
}

#[test]
fn no_content_field_is_a_postscript_operator_or_a_psanim_word() {
    for template in ["poem.ps", "process.ps"] {
        let fields = content_fields(template);
        assert!(fields.len() >= 5, "{template}: found only {fields:?}");
        for f in &fields {
            assert!(
                !psanim::api::API.iter().any(|e| e.name == f.as_str()),
                "{template}: content field /{f} is a psanim word"
            );
            let probe = format!(
                "systemdict /{f} known {{ /{f}-is-an-operator undefined }} if\n\
                 << /Size [64 36] /Duration 1 >> animation /scene {{ }} def\n"
            );
            if let Err(e) = load(&probe) {
                panic!("{template}: content field /{f} is a PostScript operator: {e}");
            }
        }
    }
}

#[test]
fn the_library_words_still_work_after_the_content_block() {
    // A content field that shadowed `loop` broke every library helper
    // that loops, such as the width of a line of emoji text.
    for template in ["poem.ps", "process.ps"] {
        if let Err(e) = standalone(template, "") {
            panic!("{template}: {e}");
        }
        let source = template_source(template) + "\n(hello) emojitextwidth pop\n";
        if let Err(e) = load(&source) {
            panic!("{template} with a library word appended: {e}");
        }
    }
}

#[test]
fn empty_text_is_refused_by_name() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "poem.ps",
            "/poem [ () (x) ] /notes [ [ 1 (x) (image) (why) ] ]",
            "line 0 of /poem must be text",
        ),
        (
            "poem.ps",
            "/notes [ [ 0 () (image) (why) ] ]",
            "note 0: the words must be text",
        ),
        (
            "poem.ps",
            "/notes [ [ 0 (snail) (image) () ] ]",
            "note 0: what it does must be text",
        ),
        ("poem.ps", "/reading ( )", "/reading must be text"),
        (
            "process.ps",
            "/steps [ [(a) ()] [(b) (x)] [(c) (x)] ] /key 1 /pathback [ ]",
            "step 0: what happens must be text",
        ),
        (
            "process.ps",
            "/steps [ [() (x)] [(b) (x)] [(c) (x)] ] /key 1 /pathback [ ]",
            "step 0: the name must be text",
        ),
        ("process.ps", "/why ()", "/why must be text"),
        ("process.ps", "/summary ()", "/summary must be text"),
        (
            "process.ps",
            "/pathback [ 2 0 () (x) ]",
            "/pathback: the label must be text",
        ),
    ];
    for (template, params, message) in cases {
        expect_refused(
            with_params(template, params),
            &format!("{template} {params}"),
            message,
        );
    }
}

#[test]
fn reading_time_counts_words_not_spaces() {
    let spaced = |n: usize| format!("A{}B", " ".repeat(n));
    // The card wraps a long run of spaces away; a line in the foot keeps
    // them, so fewer there.
    let pairs = [
        ("poem.ps", "/reading", 198),
        ("process.ps", "/summary", 40),
        ("process.ps", "/why", 40),
    ];
    for (template, field, n) in pairs {
        let d = |text: &str| {
            with_params(template, &format!("{field} ({text})"))
                .unwrap_or_else(|e| panic!("{template} {field}: {e}"))
                .meta()
                .duration
        };
        let (a, b) = (d(&spaced(n)), d("A B"));
        assert!(
            (a - b).abs() < 0.05,
            "{template} {field}: 2 words separated by {n} spaces give {a} s, \"A B\" gives {b} s"
        );
    }
}

#[test]
fn an_overlong_path_back_label_is_blamed_on_the_label() {
    expect_refused(
        with_params(
            "process.ps",
            "/pathback [ 2 0 (Wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww) (Back again.) ]",
        ),
        "a 38-character unbroken label",
        "/pathback: the label is too long",
    );
    // No message ever offers a limit below one character.
    for template in ["poem.ps", "process.ps"] {
        for params in [
            "/heading (WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW)",
            "/summary (WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW)",
        ] {
            if let Err(e) = with_params(template, params) {
                assert!(!e.contains("about -"), "{template} {params}: {e}");
            }
        }
    }
}

#[test]
fn the_default_content_plays_under_every_theme() {
    for template in ["poem.ps", "process.ps"] {
        for theme in THEMES {
            if let Err(e) = with_params(template, &format!("/look {theme}")) {
                panic!("{template} under {theme}: {e}");
            }
        }
    }
}

#[test]
fn every_film_holds_still_for_its_last_two_and_a_half_seconds() {
    let mut failures = Vec::new();
    for p in files("poem").into_iter().chain(files("process")) {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        let mut m = load(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let meta = m.meta().clone();
        let last = meta.frames - 1;
        let first = last - (2.5 * meta.fps).round() as u64;
        let end = pixels(&mut m, last).unwrap();
        for k in [first, (first + last) / 2, last - 1] {
            if pixels(&mut m, k).unwrap() != end {
                failures.push(format!(
                    "{name}: frame {k} differs from the last ({last}); the hold is shorter than 2.5 s"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_template_edited_in_place_refuses_bad_content() {
    expect_refused(
        standalone("poem.ps", "/notes [ ] def"),
        "poem.ps with no notes",
        "/notes has 0 notes; this template takes 1 to 5",
    );
    expect_refused(
        standalone("process.ps", "/key 99 def"),
        "process.ps with /key 99",
        "/key must be a step number from 0 to",
    );
}
