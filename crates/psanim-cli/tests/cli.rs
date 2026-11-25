//! The `psanim` binary, run as a user would: what it prints, as JSON.

use std::path::PathBuf;
use std::process::Command;

/// Write `source` to a file of its own and run `psanim <command>` on it
/// with `--json`; return the parsed result.
fn run_json(name: &str, source: &str, command: &[&str]) -> serde_json::Value {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let ps = dir.join(format!("{name}.ps"));
    std::fs::write(&ps, source).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_psanim"))
        .args(command)
        .arg(&ps)
        .args(["--json", "-o"])
        .arg(dir.join(format!("{name}.png")))
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
