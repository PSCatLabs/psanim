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
