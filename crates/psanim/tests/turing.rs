//! The /turing catalog entry and the gallery piece built on it: the machine
//! really runs its table, and the runs come out as they must.

use std::collections::BTreeMap;

use psanim::{Movie, Options, Param};

/// One line of what `/Report true` prints: `turing: NAME steps N ones N halted B tape T`.
#[derive(Debug, PartialEq)]
struct Run {
    steps: u64,
    ones: u64,
    halted: bool,
    tape: String,
}

fn runs(printed: &str) -> Vec<Run> {
    printed
        .lines()
        .filter(|l| l.starts_with("turing: "))
        .map(|l| {
            let word = |key: &str| {
                let at = l
                    .rfind(&format!(" {key} "))
                    .unwrap_or_else(|| panic!("{l}"));
                l[at + key.len() + 2..]
                    .split(' ')
                    .next()
                    .unwrap()
                    .to_string()
            };
            Run {
                steps: word("steps").parse().unwrap(),
                ones: word("ones").parse().unwrap(),
                halted: word("halted") == "true",
                tape: word("tape"),
            }
        })
        .collect()
}

fn run(steps: u64, ones: u64, halted: bool, tape: &str) -> Run {
    Run {
        steps,
        ones,
        halted,
        tape: tape.to_string(),
    }
}

/// Make one instance of the entry with these parameters and collect its report.
fn entry(params: &str) -> Result<Vec<Run>, String> {
    let src = format!(
        "<< /Size [320 160] /Duration 1 >> animation
         /m /turing << /Params << {params} /Report true >> >> instance
         /scene {{ /m << /After /hold >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).map_err(|e| e.to_string())?;
    Ok(runs(&m.take_output()))
}

#[test]
fn the_gallery_piece_adds_and_runs_the_busy_beaver() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../gallery/turing/turing.ps");
    let src = std::fs::read_to_string(&path).unwrap();
    let mut params = BTreeMap::new();
    params.insert("report".to_string(), Param::Number(1.0));
    let opts = Options {
        params,
        ..Options::default()
    };
    let mut m = Movie::load(&src, &opts).unwrap_or_else(|e| panic!("{e}"));
    let got = runs(&m.take_output());
    assert_eq!(
        got,
        vec![
            // 2 + 3 in unary: five marks and nothing else
            run(8, 5, true, "11111"),
            // the four-state busy beaver: 107 steps, 13 ones
            run(107, 13, true, "10111111111111"),
        ]
    );
    // and the film draws it: the last frame renders
    let last = m.meta().frames - 1;
    m.render_frame(last).unwrap();
}

#[test]
fn the_default_table_is_the_busy_beaver() {
    assert_eq!(
        entry("").unwrap(),
        vec![run(107, 13, true, "10111111111111")]
    );
}

#[test]
fn another_table_is_another_correct_run() {
    // the two-state busy beaver: 6 steps, 4 ones
    let bb2 = "/Table [ [/A 0 1 /R /B] [/A 1 1 /L /B] [/B 0 1 /L /A] [/B 1 1 /R /H] ]";
    assert_eq!(entry(bb2).unwrap(), vec![run(6, 4, true, "1111")]);
    // binary increment, three symbols (0 blank, 1 for a 0 bit, 2 for a 1 bit): 1011 + 1 = 1100
    let inc = "/Tape [2 1 2 2] /Head 3 /Table [ [/A 2 1 /L /A] [/A 1 2 /N /H] [/A 0 2 /N /H] ]";
    assert_eq!(entry(inc).unwrap(), vec![run(3, 4, true, "2211")]);
    // a state and symbol with no row halts too
    let stuck = "/Tape [1 1] /Table [ [/A 1 0 /R /A] ]";
    assert_eq!(entry(stuck).unwrap(), vec![run(2, 0, true, "")]);
}

#[test]
fn a_machine_that_never_halts_stops_at_max_steps() {
    let forever = "/Table [ [/A 0 1 /R /A] ] /MaxSteps 20";
    assert_eq!(
        entry(forever).unwrap(),
        vec![run(20, 20, false, &"1".repeat(20))]
    );
}

#[test]
fn a_run_that_misses_expect_fails_setup() {
    let e = entry("/Expect << /Steps 106 /Ones 13 >>").unwrap_err();
    assert!(e.contains("turing-run-differs-from-Expect"), "{e}");
    assert!(entry("/Expect << /Steps 107 /Ones 13 /Tape (10111111111111) >>").is_ok());
    let e = entry("/Table [ [/A 0 1 /R /A] ] /MaxSteps 5 /Expect << /Steps 5 >>").unwrap_err();
    assert!(e.contains("turing-run-differs-from-Expect"), "{e}");
}

#[test]
fn a_machine_that_travels_far_still_draws_its_tape() {
    // The head ends past cell 999: the tape's labels need more than three digits.
    // The host runs longer than the machine, which then holds its last step.
    let src = "<< /Size [320 160] /Duration 5000 /FPS 1 >> animation
         /m /turing << /Params << /Table [ [/A 0 1 /R /A] ] /MaxSteps 1000 >> >> instance
         /scene { /m << /After /hold >> place } def";
    let mut m = Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.render_at(4999.0).unwrap_or_else(|e| panic!("{e}"));
}
