//! `psanim check`'s judgements, run through the real command: a small pop
//! in a busy frame is flagged and steady fast motion is not; a scene that
//! writes into a string made in setup is reported and a clean one is not.

use std::path::PathBuf;
use std::process::Command;

/// Run `psanim check --json` on `source`; return the result.
fn run(name: &str, source: &str, extra: &[&str]) -> serde_json::Value {
    let dir = std::env::temp_dir().join(format!("psanim-check-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join(format!("{name}.ps"));
    std::fs::write(&path, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_psanim"))
        .arg("check")
        .arg(&path)
        .args(extra)
        .arg("--json")
        .output()
        .expect("psanim runs");
    let text = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {text}\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert_eq!(json["ok"], true, "{json}");
    json
}

fn strings(json: &serde_json::Value, key: &str) -> Vec<String> {
    json[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect()
}

/// Run `psanim check --json` on `source`; return the warnings.
fn check(name: &str, source: &str, extra: &[&str]) -> Vec<String> {
    strings(&run(name, source, extra), "warnings")
}

/// Forty bars drifting steadily, so every tile of the frame changes a
/// little between any two frames.
const BUSY: &str = "
<< /Size [640 360] /Duration 2 /FPS 30 /Background [0.05 0.05 0.1] >> animation
/bars {
  0 1 39 { /i exch def
    i 40 div 0.6 1 sethsbcolor
    i 15 mul now 150 mul add   i 8 mul 10 add   22 14 rectfill
  } for
} def
";

#[test]
fn a_small_pop_in_a_busy_frame_is_flagged() {
    let source = format!(
        "{BUSY}/scene {{ bars  now 1 ge {{ 1 1 1 setrgbcolor 300 172 12 12 rectfill }} if }} def"
    );
    let warnings = check("pop", &source, &[]);
    let abrupt: Vec<_> = warnings
        .iter()
        .filter(|w| w.contains("abrupt change"))
        .collect();
    assert_eq!(abrupt.len(), 1, "{warnings:?}");
    // The square appears between frames 29 and 30, near (300, 300).
    assert!(abrupt[0].contains("after frame 29"), "{}", abrupt[0]);
    assert!(abrupt[0].contains("x 288..320"), "{}", abrupt[0]);
}

#[test]
fn a_pop_is_found_where_no_sampled_frame_is() {
    // Ten seconds, 300 frames: sixteen sampled frames land nowhere near
    // frame 209, so only the scan of consecutive frames can see it.
    let source = format!(
        "{}/scene {{ bars  now 6.99 ge {{ 1 1 1 setrgbcolor 300 172 12 12 rectfill }} if }} def",
        BUSY.replace("/Duration 2", "/Duration 10")
    );
    let json = run("pop_late", &source, &[]);
    let warnings = strings(&json, "warnings");
    assert!(
        warnings.iter().any(|w| w.contains("after frame 209")),
        "{warnings:?}"
    );
    // ... and it is reported once: no sample saw it.
    assert_eq!(
        warnings.iter().filter(|w| w.contains("abrupt")).count(),
        1,
        "{warnings:?}"
    );
}

#[test]
fn a_wipe_is_motion_not_a_pop() {
    // An edge crossing the frame at 160 units a frame changes each tile
    // for one pair only, tile after tile.
    let source = "
<< /Size [640 360] /Duration 1 /FPS 30 /Background [0.05 0.05 0.1] >> animation
/scene { 1 1 1 setrgbcolor 0 0 now 4800 mul 360 rectfill } def";
    let warnings = check("wipe", source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("abrupt")),
        "{warnings:?}"
    );
}

#[test]
fn a_readout_that_changes_in_steps_is_a_note() {
    // A marker that hops 20 units every ten frames, always in the same
    // place: a counter, not an accident.
    let source = format!(
        "{BUSY}/scene {{ bars 1 1 1 setrgbcolor frame 10 idiv 20 mul 300 add 172 12 12 rectfill }} def"
    );
    let json = run("readout", &source, &["--all"]);
    let warnings = strings(&json, "warnings");
    assert!(
        !warnings.iter().any(|w| w.contains("abrupt")),
        "{warnings:?}"
    );
    let notes = strings(&json, "notes");
    assert!(
        notes.iter().any(|n| n.contains("changes in steps")),
        "{notes:?}"
    );
}

#[test]
fn steady_fast_motion_is_not_flagged() {
    // Twenty units a frame, a third of the way across a tile at preview
    // size: fast, and the same in every pair.
    let source = "
<< /Size [640 360] /Duration 1 /FPS 30 /Background [0.05 0.05 0.1] >> animation
/scene {
  0 1 11 { /i exch def
    i 12 div 0.6 1 sethsbcolor
    i 20 mul now 500 mul add  i 28 mul 10 add  30 20 rectfill
  } for
  1 1 1 setrgbcolor  now 600 mul 20 add 300 20 20 rectfill
} def";
    let warnings = check("steady", source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("abrupt change")),
        "{warnings:?}"
    );
}

#[test]
fn a_fade_is_not_flagged() {
    let source = "
<< /Size [640 360] /Duration 2 /FPS 30 /Background [0 0 0] >> animation
/scene { 1 1 1 setrgbcolor 0.5 { 200 100 120 90 rectfill } faded } def";
    let warnings = check("fade", source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("abrupt")),
        "{warnings:?}"
    );
}

const LEAK: &str = "
<< /Size [200 100] /Duration 2 /FPS 10 >> animation
/buf 4 string def
buf 0 65 put
/scene {
  1 1 1 setrgbcolor 20 20 100 40 rectfill
  buf 0 buf 0 get 1 add put
  buf 0 get 255 div 0 0 setrgbcolor 130 20 40 40 rectfill
} def";

#[test]
fn a_write_into_a_setup_string_is_reported() {
    let warnings = check("leak", LEAK, &[]);
    let leak: Vec<_> = warnings.iter().filter(|w| w.contains("string")).collect();
    assert_eq!(leak.len(), 1, "{warnings:?}");
    assert!(leak[0].contains("`buf`"), "{}", leak[0]);
    assert!(leak[0].contains("dup length string copy"), "{}", leak[0]);
}

#[test]
fn a_string_inside_a_dictionary_is_found_too() {
    let source = LEAK
        .replace(
            "/buf 4 string def\nbuf 0 65 put",
            "/state << /buf 4 string >> def\nstate /buf get 0 65 put",
        )
        .replace(
            "buf 0 buf 0 get 1 add put",
            "state /buf get 0 state /buf get 0 get 1 add put",
        )
        .replace("buf 0 get 255", "state /buf get 0 get 255");
    let warnings = check("leakdict", &source, &[]);
    assert!(
        warnings.iter().any(|w| w.contains("`state/buf`")),
        "{warnings:?}"
    );
}

#[test]
fn a_scene_that_copies_the_string_first_is_clean() {
    let source = LEAK.replace(
        "/scene {\n",
        "/scene {\n  /buf buf dup length string copy def\n",
    );
    let warnings = check("noleak", &source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("string")),
        "{warnings:?}"
    );
}

#[test]
fn a_string_only_read_is_clean() {
    let source = "
<< /Size [200 100] /Duration 1 /FPS 10 >> animation
/label (hello) def
/scene { 1 1 1 setrgbcolor /Helvetica findfont 20 scalefont setfont 10 40 moveto label show } def";
    let warnings = check("readonly", source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("string")),
        "{warnings:?}"
    );
}

#[test]
fn a_stateful_movie_may_keep_state_in_a_string() {
    // Strings are exempt from rollback, which a stateful movie knows about
    // (modes.md says not to keep state in them); `check` does not second-guess.
    let source = "
<< /Size [200 100] /Duration 1 /FPS 10 >> animation
/buf 4 string def
/step { buf 0 buf 0 get 1 add put } def
/scene { 1 1 1 setrgbcolor 20 20 100 40 rectfill } def";
    let warnings = check("stateful", source, &[]);
    assert!(
        !warnings.iter().any(|w| w.contains("string")),
        "{warnings:?}"
    );
}
