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
    let mut started = false;
    for k in psanim::sheet::spread(meta.frames, 60) {
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
            "/loop [ 1 1 (again) (the same step) ]",
            "/loop: from and to must be different steps",
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
    ];
    for (template, params, message) in cases {
        let e = refused(template, params);
        assert!(
            e.contains(&format!("content: {message}")),
            "{template} with {params}: expected `{message}`, got:\n{e}"
        );
    }
}
