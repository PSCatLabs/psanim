//! The `psanim` binary, run as a user would: what it prints, as JSON.

use std::path::PathBuf;
use std::process::Command;

/// Write `source` to a file of its own and run `psanim <command>` on it
/// with `--json`; return the parsed result.
fn run_json(name: &str, source: &str, command: &[&str]) -> serde_json::Value {
    run_json_as(name, source, command, "png")
}

/// [`run_json`], writing the output file with extension `ext`.
fn run_json_as(name: &str, source: &str, command: &[&str], ext: &str) -> serde_json::Value {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let ps = dir.join(format!("{name}.ps"));
    std::fs::write(&ps, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_psanim"))
        .args(command)
        .arg(&ps)
        .args(["--json", "-o"])
        .arg(dir.join(format!("{name}.{ext}")))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(text.lines().last().unwrap_or(""))
        .unwrap_or_else(|e| panic!("{e}: {text}\n{}", String::from_utf8_lossy(&out.stderr)))
}

const HEAD: &str = "<< /Size [64 48] /Duration 1 /FPS 10 >> animation\n";
const LINE: &str = "(0123456789012345678901234567890123456789) =";

#[test]
fn frame_json_says_when_output_was_cut() {
    let loud = run_json(
        "frame_loud",
        &format!("{HEAD}/scene {{ 1 1 3000 {{ pop {LINE} }} for }} def"),
        &["frame"],
    );
    assert_eq!(loud["ok"], true);
    assert_eq!(loud["output_truncated"], true);
    assert!(
        loud["printed"]
            .as_str()
            .unwrap()
            .ends_with("[output truncated at 64 KB]\n")
    );
    let quiet = run_json(
        "frame_quiet",
        &format!("{HEAD}/scene {{ (hi) = }} def"),
        &["frame"],
    );
    assert_eq!(quiet["output_truncated"], false);
    assert_eq!(quiet["printed"], "hi\n");
}

#[test]
fn sheet_and_render_json_report_what_was_printed_and_when_it_was_cut() {
    // Setup's output once, then each frame's, in frame order: the same on any
    // number of threads.
    let src = format!("{HEAD}(setup) =\n/scene {{ frame == }} def");
    let sheet = run_json("sheet_prints", &src, &["sheet", "--frame", "0,1,2"]);
    let render = run_json_as("render_prints", &src, &["render"], "gif");
    for r in [sheet, render] {
        let want: String = std::iter::once("setup".to_string())
            .chain((0..3).map(|k| k.to_string()))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let printed = r["printed"].as_str().unwrap();
        assert!(printed.starts_with(&want), "{printed}");
        assert_eq!(r["output_truncated"], false);
    }
    let loud = format!("{HEAD}/scene {{ 1 1 3000 {{ pop {LINE} }} for }} def");
    for r in [
        run_json("sheet_loud", &loud, &["sheet", "--count", "2"]),
        run_json_as("render_loud", &loud, &["render"], "gif"),
    ] {
        assert_eq!(r["output_truncated"], true);
        let printed = r["printed"].as_str().unwrap();
        assert!(printed.ends_with("[output truncated at 64 KB]\n"));
        assert!(printed.len() < 70_000);
    }
}

#[test]
fn check_finds_a_write_into_a_literal_string_in_a_procedure() {
    // `(abcd)` in a procedure body is one string for every call: writing into it
    // changes it for every later frame, so check names the procedure.
    let r = run_json(
        "literal_write",
        &format!("{HEAD}/stamp {{ (abcd) dup 0 frame 65 add put pop }} def /scene {{ stamp }} def"),
        &["check", "--count", "3"],
    );
    let w = warnings(&r);
    assert!(
        w.iter()
            .any(|w| w.contains("wrote into") && w.contains("`stamp{0}`")),
        "{w:?}"
    );
}

#[test]
fn check_json_says_when_output_was_cut() {
    let loud = run_json(
        "check_loud",
        &format!("{HEAD}/scene {{ 1 1 3000 {{ pop {LINE} }} for }} def"),
        &["check", "--count", "2"],
    );
    assert_eq!(loud["output_truncated"], true);
    assert!(
        loud["printed"]
            .as_str()
            .unwrap()
            .ends_with("[output truncated at 64 KB]\n")
    );
    let quiet = run_json(
        "check_quiet",
        &format!("{HEAD}/scene {{ 0.5 setgray 0 0 64 48 rectfill }} def"),
        &["check", "--count", "2"],
    );
    assert_eq!(quiet["output_truncated"], false);
}

fn warnings(result: &serde_json::Value) -> Vec<String> {
    result["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect()
}

fn preamble_warning(result: &serde_json::Value) -> Option<String> {
    warnings(result)
        .into_iter()
        .find(|w| w.contains("before `animation`"))
}

const SCENE: &str = "/scene { 0.5 setgray 0 0 64 48 rectfill } def";

#[test]
fn check_warns_about_a_slow_preamble() {
    // Two million iterations: well over the 50 ms threshold.
    let r = run_json(
        "preamble_slow",
        &format!("1 1 2000000 {{ 3 mul pop 1 }} for\n{HEAD}{SCENE}"),
        &["check", "--count", "2"],
    );
    let w = preamble_warning(&r).expect("a warning about the preamble");
    assert!(
        w.contains("runs twice") && w.contains("after `animation`"),
        "{w}"
    );
    assert!(w.contains("takes "), "{w}");
    assert!(r["movie"]["preamble_ms"].as_f64().unwrap() > 50.0);
}

#[test]
fn check_warns_about_a_printing_preamble_and_shows_the_print_once() {
    let r = run_json(
        "preamble_prints",
        &format!("(hello) =\n{HEAD}{SCENE}"),
        &["check", "--count", "2"],
    );
    let w = preamble_warning(&r).expect("a warning about the preamble");
    assert!(w.contains("prints") && w.contains("runs twice"), "{w}");
    assert_eq!(r["printed"], "hello\n");
}

#[test]
fn check_does_not_warn_about_definitions_before_animation() {
    let r = run_json(
        "preamble_clean",
        &format!("/dot {{ 2 2 rectfill }} def\n/pi2 {{ 6.2832 }} def\n{HEAD}{SCENE}"),
        &["check", "--count", "2"],
    );
    assert_eq!(preamble_warning(&r), None);
    // Slow or printing code after `animation` is not the preamble.
    let r = run_json(
        "preamble_after",
        &format!("{HEAD}(hello) =\n1 1 2000000 {{ 3 mul pop 1 }} for\n{SCENE}"),
        &["check", "--count", "2"],
    );
    assert_eq!(preamble_warning(&r), None);
}
