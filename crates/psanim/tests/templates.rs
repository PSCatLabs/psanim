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

// ---- the poem's card, measured on rendered frames ---------------------------
//
// The second blind review (out/eval6, issue #108) marked the poem template
// down for four things a contact sheet can miss and a frame scan cannot:
// two notes legible together during a handoff, a card left empty between
// notes, a sentence shown unfinished while it is revealed, and a closing
// hold too short or too long. These tests render a poem film at 480 px
// wide and read the card's column (right of 58% of the width, where only
// the card is drawn) frame by frame, in grey.
//
// - A *rest* is a run of at least three identical frames of the column.
//   The rests before the card are those identical to the first frame.
// - The *ground* of a pixel is its value in every rest of the walk if it
//   never changes there (the card's frame, the page around the card), and
//   otherwise the card's surface. Ink is a difference from the ground.
// - A *text* is a chain of rests in which each keeps 90% of the ink of the
//   one before (a line added to a note); its last rest is the text whole.
// - A *handoff* is the frames between one text's last rest and the next
//   text's first.

/// A pixel is inked when it differs from the ground by this fraction of
/// the typical difference of text from the ground: about the opacity at
/// which a word can still be read. Checked against the frames the
/// reviewers named: it finds the round-2 template's frame 480 with both
/// notes legible, and the round-1 template's frame 390 empty.
const LEGIBLE: f64 = 0.3;
/// The most frames the card may hold no legible ink between two texts:
/// two (0.07 s). When one text has gone before the next comes, one frame
/// can fall between them; a second allows for where the frames fall. The
/// reviews marked a blank card of 12 frames (12.60 to 13.00 s in a
/// from-scratch film); they also caught the round-1 template's single
/// empty frame, at 13.0 s, in a still, which this does not fail.
const MAX_EMPTY: usize = 2;
/// The most frames a text may take from its first rest to its whole one:
/// 0.2 s. A text that comes in whole has no rest before it is whole; one
/// revealed a line or a word at a time sits still, unfinished, between
/// lines.
const MAX_PARTIAL: usize = 6;
/// Pixels of legible ink that count as text (a letter of the smallest
/// type on the card has about this many at 480 px wide).
const SOME_INK: usize = 25;

struct CardScan {
    fps: f64,
    /// The first frame of the first rest that shows a text.
    walk: usize,
    /// Per text: the first frame of its first rest, and its last rest.
    texts: Vec<(usize, (usize, usize))>,
    /// Per frame: legible ink in the column, in pixels.
    inked: Vec<usize>,
    /// Frames, in a handoff, where the outgoing text is still legible in
    /// its place and other ink is legible beside it.
    overlaps: Vec<usize>,
}

fn grey(rgb: &[u8]) -> u8 {
    ((u32::from(rgb[0]) * 77 + u32::from(rgb[1]) * 150 + u32::from(rgb[2]) * 29) >> 8) as u8
}

fn scan_card(file: &str) -> CardScan {
    let dir = dir();
    let source = std::fs::read_to_string(dir.join(file)).unwrap();
    let options = Options {
        scale: Scale::Width(480),
        read_roots: Some(vec![dir.clone()]),
        base_dir: Some(dir),
        ..Options::default()
    };
    let movie = Movie::load(&source, &options).unwrap();
    let meta = movie.meta().clone();
    let (w, h) = (meta.pixel_width as usize, meta.pixel_height as usize);
    let x0 = (w as f64 * 0.58) as usize;
    let cw = w - x0;
    let matte = meta
        .background
        .map_or([0, 0, 0], |c| c.map(|v| (v * 255.0).round() as u8));
    let indices: Vec<u64> = (0..meta.frames).collect();
    let mut cols: Vec<Vec<u8>> = Vec::with_capacity(indices.len());
    let mut flat = Vec::new();
    psanim::render::render_frames_from(movie, &source, &options, &indices, 4, |r| {
        raster::flatten_onto(&r.premultiplied, matte, &mut flat);
        let mut col = Vec::with_capacity(cw * h);
        for y in 0..h {
            for x in x0..w {
                let i = (y * w + x) * 4;
                col.push(grey(&flat[i..i + 3]));
            }
        }
        cols.push(col);
        Ok(())
    })
    .unwrap();
    let n = cols.len();
    let npix = cw * h;

    let mut rests: Vec<(usize, usize)> = Vec::new();
    let mut a = 0;
    while a < n {
        let mut b = a + 1;
        while b < n && cols[b] == cols[a] {
            b += 1;
        }
        if b - a >= 3 && cols[a] != cols[0] {
            rests.push((a, b));
        }
        a = b;
    }
    assert!(!rests.is_empty(), "{file}: nothing ever comes to the card");
    let mut counts = [0u32; 256];
    for &(a, _) in &rests {
        for &v in &cols[a] {
            counts[usize::from(v)] += 1;
        }
    }
    let surface = (0..256).max_by_key(|&v| counts[v]).unwrap() as u8;
    let ground: Vec<u8> = (0..npix)
        .map(|p| {
            let v = cols[rests[0].0][p];
            if rests.iter().all(|&(a, _)| cols[a][p] == v) {
                v
            } else {
                surface
            }
        })
        .collect();
    let diff = |k: usize, p: usize| f64::from((i16::from(cols[k][p]) - i16::from(ground[p])).abs());
    let ink = |k: usize| -> Vec<bool> { (0..npix).map(|p| diff(k, p) > 40.0).collect() };

    // Texts: chains of rests, each keeping most of the ink of the one before.
    let mut texts: Vec<(usize, (usize, usize))> = Vec::new();
    let mut prev: Option<Vec<bool>> = None;
    for &(a, b) in &rests {
        let m = ink(a);
        if m.iter().filter(|&&v| v).count() < SOME_INK {
            continue; // the card, empty
        }
        let grows = prev.as_ref().is_some_and(|q| {
            let qn = q.iter().filter(|&&v| v).count();
            let kept = q.iter().zip(&m).filter(|(x, y)| **x && **y).count();
            kept as f64 >= 0.9 * qn as f64
        });
        match texts.last_mut() {
            Some(t) if grows => t.1 = (a, b),
            _ => texts.push((a, (a, b))),
        }
        prev = Some(m);
    }
    let walk = texts.first().expect("no text on the card").0;
    let masks: Vec<Vec<bool>> = texts.iter().map(|t| ink(t.1.0)).collect();
    // The typical difference of text from the ground.
    let (sum, count) = masks
        .iter()
        .zip(&texts)
        .flat_map(|(m, t)| (0..npix).filter(|&p| m[p]).map(|p| diff(t.1.0, p)))
        .fold((0.0, 0usize), |(s, c), d| (s + d, c + 1));
    let typical = sum / count as f64;
    let inked: Vec<usize> = (0..n)
        .map(|k| {
            (0..npix)
                .filter(|&p| diff(k, p) >= LEGIBLE * typical)
                .count()
        })
        .collect();

    // Handoffs. The outgoing text has finished arriving and does not move,
    // so its presence is how inked its letters are (as a fraction of when
    // it is whole) less how inked the gaps between them are, inside the
    // boxes of its lines, both away from where the incoming text's letters
    // end up: text passing over it inks letters and gaps alike. It is
    // legible at LEGIBLE or more. The incoming text is whatever legible ink
    // lies outside the outgoing one's letters.
    let rows = h;
    let mut overlaps = Vec::new();
    for c in 0..texts.len().saturating_sub(1) {
        let (out_end, in_start) = (texts[c].1.1, texts[c + 1].0);
        let f = texts[c].1.0;
        let m = &masks[c];
        let grow = |m: &Vec<bool>| -> Vec<bool> {
            (0..npix)
                .map(|p| {
                    let (x, y) = ((p % cw) as i64, (p / cw) as i64);
                    (-1..=1).any(|dy: i64| {
                        (-1..=1).any(|dx: i64| {
                            let (xx, yy) = (x + dx, y + dy);
                            xx >= 0
                                && yy >= 0
                                && (xx as usize) < cw
                                && (yy as usize) < rows
                                && m[yy as usize * cw + xx as usize]
                        })
                    })
                })
                .collect()
        };
        let near_mask = grow(m);
        let near_next = grow(&masks[c + 1]);
        let inked_rows: Vec<bool> = (0..rows).map(|y| (0..cw).any(|x| m[y * cw + x])).collect();
        let lines: Vec<(Vec<usize>, Vec<usize>)> = runs(0, rows, |y| inked_rows[y])
            .into_iter()
            .map(|(y0, len)| {
                let band: Vec<usize> = (y0 * cw..(y0 + len) * cw).collect();
                let xs = band.iter().filter(|&&p| m[p]).map(|&p| p % cw);
                let (lo, hi) = (xs.clone().min().unwrap(), xs.max().unwrap());
                let inside: Vec<usize> = band
                    .into_iter()
                    .filter(|&p| (lo..=hi).contains(&(p % cw)))
                    .collect();
                let letters = inside
                    .iter()
                    .copied()
                    .filter(|&p| m[p] && !near_next[p])
                    .collect();
                let gaps = inside
                    .iter()
                    .copied()
                    .filter(|&p| !near_mask[p] && !near_next[p])
                    .collect();
                (letters, gaps)
            })
            .collect();
        for k in out_end..in_start {
            let (letters, gaps): (Vec<usize>, Vec<usize>) = (
                lines.iter().flat_map(|l| l.0.iter().copied()).collect(),
                lines.iter().flat_map(|l| l.1.iter().copied()).collect(),
            );
            let on = letters
                .iter()
                .map(|&p| (diff(k, p) / diff(f, p)).min(1.0))
                .sum::<f64>()
                / letters.len() as f64;
            let off = gaps
                .iter()
                .map(|&p| (diff(k, p) / typical).min(1.0))
                .sum::<f64>()
                / gaps.len().max(1) as f64;
            let present = on - off >= LEGIBLE;
            let other = (0..npix)
                .filter(|&p| !near_mask[p] && diff(k, p) >= LEGIBLE * typical)
                .count();
            if present && other >= SOME_INK {
                overlaps.push(k);
            }
        }
    }
    CardScan {
        fps: meta.fps,
        walk,
        texts,
        inked,
        overlaps,
    }
}

fn runs(from: usize, to: usize, test: impl Fn(usize) -> bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut k = from;
    while k < to {
        if test(k) {
            let a = k;
            while k < to && test(k) {
                k += 1;
            }
            out.push((a, k - a));
        } else {
            k += 1;
        }
    }
    out
}

/// The films the card tests scan, each with its number of notes: every
/// poem film, among them the brief the blind reviews judged (basho); four
/// themes, one note to five.
const CARD_FILMS: [(&str, usize); 7] = [
    ("poem-basho.ps", 3),
    ("poem.ps", 3),
    ("poem-long.ps", 5),
    ("poem-buson.ps", 1),
    ("poem-pound.ps", 2),
    ("poem-sandburg.ps", 5),
    ("poem-verlaine.ps", 2),
];

fn scan_all(check: impl Fn(&str, &CardScan, &mut Vec<String>)) {
    let mut failures = Vec::new();
    for (file, notes) in CARD_FILMS {
        let s = scan_card(file);
        assert_eq!(
            s.texts.len(),
            notes + 1,
            "{file}: found {} texts on the card, expected {notes} notes and the reading",
            s.texts.len()
        );
        check(file, &s, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_card_never_shows_two_notes_legibly_at_once() {
    scan_all(|file, s, failures| {
        if let Some(&k) = s.overlaps.first() {
            failures.push(format!(
                "{file}: {} frames with the outgoing text legible beside other ink, first {k} ({:.2} s): {:?}",
                s.overlaps.len(),
                k as f64 / s.fps,
                s.overlaps
            ));
        }
    });
}

#[test]
fn the_card_is_never_empty_for_more_than_a_blink() {
    scan_all(|file, s, failures| {
        for (a, len) in runs(s.walk, s.inked.len(), |k| s.inked[k] < SOME_INK) {
            if len > MAX_EMPTY {
                failures.push(format!(
                    "{file}: nothing legible on the card for {len} frames from frame {a} ({:.2} s)",
                    a as f64 / s.fps
                ));
            }
        }
    });
}

#[test]
fn no_sentence_is_shown_unfinished() {
    scan_all(|file, s, failures| {
        for (t, &(first, (whole, _))) in s.texts.iter().enumerate() {
            if whole - first > MAX_PARTIAL {
                failures.push(format!(
                    "{file}: text {t} sits unfinished from frame {first} ({:.2} s) to frame {whole} ({:.2} s)",
                    first as f64 / s.fps,
                    whole as f64 / s.fps
                ));
            }
        }
    });
}

/// The words of the first `/reading (...)` in a file.
fn reading_words(file: &str) -> usize {
    let source = template_source(file);
    let start = source.find("/reading (").unwrap() + "/reading (".len();
    let end = start + source[start..].find(')').unwrap();
    source[start..end].split_whitespace().count()
}

#[test]
fn the_finished_picture_holds_as_long_as_the_reading_takes_to_read() {
    // At 0.24 s a word (250 words a minute, the template's reading pace)
    // the reading takes words x 0.24 s; the finished picture should then
    // stay 1.5 to 3.5 s more. The round-2 film held 4.4 s for a 21-word
    // reading, less than it takes to read; the round-1 film about 10 s,
    // marked down as too long.
    let mut failures = Vec::new();
    for (file, _) in CARD_FILMS {
        let source = template_source(file);
        let mut m = load(&source).unwrap();
        let meta = m.meta().clone();
        let last = meta.frames - 1;
        let end = pixels(&mut m, last).unwrap();
        let mut k = last;
        while k > 0 && pixels(&mut m, k - 1).unwrap() == end {
            k -= 1;
        }
        let hold = (last - k + 1) as f64 / meta.fps;
        let read = reading_words(file) as f64 * 0.24;
        if hold < read + 1.5 || hold > read + 3.5 {
            failures.push(format!(
                "{file}: the finished picture holds {hold:.2} s; its reading takes {read:.2} s, \
                 so it should hold {:.2} to {:.2} s",
                read + 1.5,
                read + 3.5
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_poem_card_adds_no_quotation_marks() {
    // Quote marks come from the content. The round-2 film put curly marks
    // around words its brief gave in ASCII; the round-1 film straight ones.
    let source = template_source("poem.ps");
    let marker = "% ===================== nothing below here needs to change";
    let code: Vec<&str> = source[source.find(marker).unwrap()..]
        .lines()
        .map(|l| l.split('%').next().unwrap_or(""))
        .collect();
    for (i, line) in code.iter().enumerate() {
        for mark in ["\u{201c}", "\u{201d}", "\u{2018}", "(\")", "(')"] {
            assert!(
                !line.contains(mark),
                "poem.ps adds a quotation mark {mark:?} below the content block, line {}: {line}",
                i + 1
            );
        }
    }
}
