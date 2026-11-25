//! The reference table against the interpreter: everything documented
//! exists, everything that exists is documented, and none of it collides
//! with a name one of PSCat's own libraries defines.

use std::collections::BTreeSet;

use psanim::api::{API, CATEGORIES};
use psanim::{Movie, Options};

/// Names `lib` adds to userdict or systemdict, once `before` has loaded.
fn defined_by(before: &str, lib: &str) -> BTreeSet<String> {
    let mut interp = pscat::Interp::new();
    interp.capture_stdout();
    interp
        .run_str(before)
        .unwrap_or_else(|e| panic!("{before}: {e}"));
    let base = names(&mut interp);
    interp.run_str(lib).unwrap_or_else(|e| panic!("{lib}: {e}"));
    names(&mut interp).difference(&base).cloned().collect()
}

fn names(interp: &mut pscat::Interp) -> BTreeSet<String> {
    interp
        .run_str("[ systemdict { pop } forall userdict { pop } forall ]")
        .unwrap();
    match interp.pop_value().unwrap() {
        pscat::host::Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_text().map(str::to_string))
            .collect(),
        _ => panic!(),
    }
}

#[test]
fn every_documented_name_is_defined_in_a_movie() {
    // A scene that looks every name up; an undefined one fails the load.
    let lookups: String = API
        .iter()
        .map(|e| {
            format!(
                "/{} where {{ pop }} {{ /{} missing }} ifelse\n",
                e.name, e.name
            )
        })
        .collect();
    let libs: String = psanim::LIBRARIES
        .iter()
        .map(|l| format!("/{} use\n", l.0))
        .collect();
    let src = format!("<< /Size [8 8] >> animation\n{libs}{lookups}/scene {{ }} def");
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn every_entry_is_complete_and_categories_are_known() {
    let cats: BTreeSet<&str> = CATEGORIES.iter().map(|c| c.0).collect();
    let mut seen = BTreeSet::new();
    for e in API {
        assert!(
            cats.contains(e.category),
            "{}: unknown category {}",
            e.name,
            e.category
        );
        assert!(
            e.stack.contains(&format!("  {}  ", e.name))
                || e.stack.starts_with(&format!("-  {}  ", e.name)),
            "{}: stack effect `{}` should show the name",
            e.name,
            e.stack
        );
        assert!(
            e.summary.len() > 10 && e.summary.ends_with('.'),
            "{}: summary",
            e.name
        );
        assert!(seen.insert(e.name), "{} is listed twice", e.name);
    }
    for (key, _) in CATEGORIES {
        assert!(
            API.iter().any(|e| e.category == *key),
            "category {key} is empty"
        );
    }
    assert!(psanim::api::to_markdown().contains("### `tween`"));
    assert!(!psanim::api::to_markdown().contains("### `enters`"));
    assert!(
        psanim::api::library_markdown("pkmotion")
            .unwrap()
            .contains("### `enters`")
    );
    assert!(psanim::api::library_markdown("tween").is_none());
    assert_eq!(psanim::api::to_json().as_array().unwrap().len(), API.len());
    assert_eq!(psanim::api::lookup("now").unwrap().category, "clock");
}

#[test]
fn every_public_name_the_prelude_defines_is_documented() {
    // The prelude's public names are the `/name { ... } bind def` lines
    // whose name does not start with a dot.
    let documented: BTreeSet<&str> = API.iter().map(|e| e.name).collect();
    for line in psanim::PRELUDE.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix('/')
            && !line.starts_with("/.")
            && (rest.contains("{") || rest.contains(" dict def"))
            && !line.starts_with("  ")
        {
            let name = rest.split_whitespace().next().unwrap();
            // Transition names live in their own dictionary.
            if psanim::TRANSITIONS.contains(&name) {
                continue;
            }
            assert!(
                documented.contains(name),
                "prelude defines `{name}` but the reference does not list it"
            );
        }
    }
}

#[test]
fn transitions_table_matches_the_prelude() {
    let src = format!(
        "<< /Size [8 8] >> animation
         {}
         /n 0 def transitions {{ pop pop /n n 1 add def }} forall
         n {} ne {{ /transitions-count-mismatch missing }} if
         /scene {{ }} def",
        psanim::TRANSITIONS
            .iter()
            .map(|t| format!("transitions /{t} known not {{ /{t} missing }} if\n"))
            .collect::<String>(),
        psanim::TRANSITIONS.len()
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
}

/// The public names a library's source defines at its top level: lines
/// that start `/name {`, `/name <<`, `/name [` or `/name N dict def`.
fn public_names(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix('/')?;
            if rest.starts_with('.') {
                return None;
            }
            let mut words = rest.split_whitespace();
            let name = words.next()?;
            let next = words.next()?;
            (next.starts_with('{')
                || next.starts_with("<<")
                || next.starts_with('[')
                || rest.ends_with(" def"))
            .then_some(name)
        })
        .collect()
}

#[test]
fn every_library_word_is_documented_under_its_library_and_nowhere_else() {
    let mut owner: std::collections::BTreeMap<&str, &str> = Default::default();
    for (lib, source, what) in psanim::LIBRARIES {
        assert!(what.len() > 20, "{lib}: describe it");
        let defined = public_names(source);
        assert!(defined.len() >= 4, "{lib}: found only {defined:?}");
        for name in &defined {
            let entry = psanim::api::lookup(name).unwrap_or_else(|| {
                panic!("{lib} defines `{name}` but the reference does not list it")
            });
            assert_eq!(
                entry.category, *lib,
                "`{name}` is defined by {lib} but documented under {}",
                entry.category
            );
            if let Some(other) = owner.insert(name, lib) {
                panic!("`{name}` is defined by both {other} and {lib}");
            }
        }
        // And the other way: every word documented under the library is one
        // its source defines, or a native operator documented with it.
        let src = format!(
            "<< /Size [8 8] >> animation\n/{lib} use\n{}/scene {{ }} def",
            API.iter()
                .filter(|e| e.category == *lib)
                .map(|e| format!(
                    "/{} where {{ pop }} {{ /{} missing }} ifelse\n",
                    e.name, e.name
                ))
                .collect::<String>()
        );
        Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{lib}: {e}"));
    }
}

/// Each library, loaded alone, must be able to run every word it
/// documents: a word that quietly needs another library fails here.
#[test]
fn each_library_alone_can_run_its_own_words() {
    let uses = [
        (
            "pkstage",
            "ground canvasbox 2 2 1 cells 0 get 1 roundpath fill /accent ink [1 1 4 4] 0.5 spotlight",
        ),
        ("pkmotion", "0 /pop { 1 setgray 1 1 3 3 rectfill } enters"),
        (
            "pktype",
            "/body 2 face /fg ink (a) canvasbox /c showin [ (a) /pi [/sup (2)] ] showruns",
        ),
        (
            "pkdiagram",
            "(a) [0 0 4 4] /surface node [0 0 2 2] [5 5 2 2] /elbow edge /body 1 face (n) [2 2 2 2] /n 1 note",
        ),
        (
            "pkcode",
            "(x = 1) /python codetokens canvasbox << /Title (t) >> codeblock",
        ),
        (
            "pkchart",
            "canvasbox [0 1] [0 1] plotframe << >> plotaxes [1] << >> bars 0.5 0.5 << /Label (m) >> plotmark << /Y 0.5 /Label (r) >> plotrule [1 2] canvasbox << /Center (c) >> donut",
        ),
        (
            "pkink",
            "newpath 0 0 moveto 8 8 lineto 1 2 { 1 setgray 0 0 8 8 rectfill } revealalong canvasbox /bg /accent 0 boxgradient",
        ),
    ];
    assert_eq!(uses.len(), psanim::LIBRARIES.len());
    for (lib, body) in uses {
        let src =
            format!("<< /Size [8 8] /Duration 1 >> animation /{lib} use /scene {{ {body} }} def");
        let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{lib}: {e}"));
        m.render_at(0.5)
            .map(|_| ())
            .unwrap_or_else(|e| panic!("{lib} alone: {e}"));
    }
}

#[test]
fn a_library_loads_once_and_an_unknown_one_is_explained() {
    let src = "<< /Size [8 8] >> animation
        /pkchart use /pkchart use (pkchart) use
        /theme where { pop } { /pkstage-was-not-loaded-by-pkchart missing } ifelse
        /scene { } def";
    Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    let err = Movie::load(
        "<< /Size [8 8] >> animation /pkmagic use /scene { } def",
        &Options::default(),
    )
    .map(|_| ())
    .expect_err("no such library");
    assert!(err.to_string().contains("/pkmotion"), "{err}");
    // A library word used without its library says which library.
    let mut movie = Movie::load(
        "<< /Size [8 8] >> animation /scene { 0 /pop { } enters } def",
        &Options::default(),
    )
    .unwrap();
    let err = movie
        .render_frame(0)
        .map(|_| ())
        .expect_err("enters is not loaded");
    assert!(err.to_string().contains("/pkmotion use"), "{err}");
}

/// PSCat's libraries define hundreds of short names. If one of them
/// redefined a psanim word, loading that library in a movie would silently
/// change what the word means. This loads each library on a bare
/// interpreter and checks the names it adds against the vocabulary.
#[test]
fn no_pscat_library_redefines_a_psanim_word() {
    let vocabulary: BTreeSet<&str> = API.iter().map(|e| e.name).collect();
    // Each library with what it needs loaded first.
    let libs = [
        ("artkit", ""),
        ("paintkit", "artkit"),
        ("drykit", "artkit"),
        ("drychalk", "artkit"),
        ("drycharcoal", "artkit"),
        ("drycrayon", "artkit"),
        ("graph", ""),
        ("dataviz", ""),
        ("pagekit", "artkit"),
        ("functional", ""),
        ("aspect", "functional"),
        ("fillkit", "artkit"),
        ("hatchkit", "artkit"),
        ("stipplekit", "artkit"),
        ("halftonekit", "artkit"),
        ("surfacekit", "artkit"),
        ("mathkit", "artkit"),
        ("lettering", "artkit"),
        ("headline", "artkit"),
        ("handscript", "artkit"),
        ("strokeglyphs", "handscript"),
        ("typewriter_outlines", ""),
        ("typewriter", "typewriter_outlines"),
        ("warp", "functional aspect"),
        ("printkit", "artkit"),
        ("etching", ""),
        ("qr", ""),
        ("kernel", ""),
        ("apparel", "artkit"),
        ("jersey", "artkit"),
        ("hangul", "artkit"),
        ("noiseref", ""),
    ];
    let mut clashes = Vec::new();
    for (lib, needs) in libs {
        let before: String = needs
            .split_whitespace()
            .map(|n| format!("(lib/{n}.ps) run "))
            .collect();
        let added = defined_by(&before, &format!("(lib/{lib}.ps) run"));
        assert!(
            !added.is_empty(),
            "lib/{lib}.ps defined nothing: was it found?"
        );
        for name in added {
            if vocabulary.contains(name.as_str()) {
                clashes.push(format!("{lib}: {name}"));
            }
        }
    }
    assert!(
        clashes.is_empty(),
        "name clashes with PSCat libraries: {clashes:?}"
    );
}
