//! Typing and terminal sessions: the plan words, and the `terminal` and
//! status entries of the catalog.

use psanim::{Movie, Options};

/// A rendered frame's pixels, kept after its movie is gone.
struct Frame {
    premultiplied: Vec<u8>,
}

const HEAD: &str = "<< /Size [200 120] /Duration 4 /FPS 10 >> animation\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

/// What `expr` prints (with `==`) in the scene at time `t`.
fn eval_at(setup: &str, expr: &str, t: f64) -> String {
    let mut m = load(&format!("{setup}\n/scene {{ {expr} }} def"));
    m.take_output();
    m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

#[test]
fn typing_is_a_pure_function_of_time_and_seed() {
    let setup = "/p (cargo build --release) 5 << /Typos 0.3 >> typingplan def";
    let end = eval_at(setup, "p p planlength 1 add typedat ==", 0.0);
    assert_eq!(end, "(cargo build --release)");
    // The same time asked in any order gives the same text.
    let a = eval_at(setup, "p 0.9 typedat ==", 0.0);
    let b = eval_at(setup, "p 0.9 typedat ==", 3.0);
    assert_eq!(a, b);
    let mid: String = a.trim_matches(|c| c == '(' || c == ')').to_string();
    assert!(!mid.is_empty() && mid.len() < 22, "{mid}");
    // Another seed types differently, so the length differs or the text does.
    let other = eval_at(
        "/p (cargo build --release) 6 << /Typos 0.3 >> typingplan def",
        "p planlength ==",
        0.0,
    );
    let first = eval_at(setup, "p planlength ==", 0.0);
    assert_ne!(other, first);
}

#[test]
fn a_slip_puts_wrong_text_on_screen_that_is_then_corrected() {
    // Ask for a slip at the third letter and look at every moment.
    let out = eval_at(
        "/p (hello world) 2 << /Typos 0 /Slips [2] >> typingplan def",
        "0 0.02 p planlength { dup p exch typedat == pop } for p p planlength typedat ==",
        0.0,
    );
    let states: Vec<&str> = out.lines().collect();
    assert!(
        states.iter().any(|s| !s.contains("(hello world)")
            && s.len() > 3
            && !"(hello world)".starts_with(s)),
        "no wrong state seen: {states:?}"
    );
    assert_eq!(*states.last().unwrap(), "(hello world)");
}

#[test]
fn a_session_gives_the_screen_at_any_time_and_marks_its_moments() {
    let setup = "/s [ [/type (ls)] [/lines (a\\nb\\nc) 0.1] [/mark /listed] [/wait 1] [/type (pwd)] [/out (/home)] ]
         9 << /Prompt (> ) >> sessionplan def";
    let end = eval_at(setup, "s s planlength 1 add 8 40 screenat pop pop ==", 0.0);
    assert!(
        end.contains("(> ls)") && end.contains("(a)") && end.contains("(/home)"),
        "{end}"
    );
    assert!(end.ends_with("(> )]"), "ends at a fresh prompt: {end}");
    // Scrolling: the screen holds only the last rows.
    let tail = eval_at(setup, "s s planlength 1 add 3 40 screenat pop pop ==", 0.0);
    assert!(
        !tail.contains("(> ls)") && tail.contains("(/home)"),
        "{tail}"
    );
    // The mark is a time inside the session, and a name it lacks is -1.
    let m: f64 = eval_at(setup, "s /listed planmark ==", 0.0)
        .parse()
        .unwrap();
    let len: f64 = eval_at(setup, "s planlength ==", 0.0).parse().unwrap();
    assert!(m > 0.5 && m < len, "{m} {len}");
    assert_eq!(eval_at(setup, "s /nope planmark ==", 0.0), "-1.0");
    // The cursor is solid while keys arrive: `last` is the time of the latest event.
    let last: f64 = eval_at(setup, "s 0.3 8 40 screenat exch pop exch pop ==", 0.0)
        .parse()
        .unwrap();
    assert!((0.0..=0.3).contains(&last), "{last}");
}

#[test]
fn a_bad_session_says_what_is_wrong() {
    let err = match Movie::load(
        &format!("{HEAD}/s [ [/dance (x)] ] 1 << >> sessionplan def /scene {{ }} def"),
        &Options::default(),
    ) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("a bad session loaded"),
    };
    assert!(err.contains("unknown kind /dance"), "{err}");
    let mut m = Movie::load(
        &format!("{HEAD}/scene {{ (not a plan) planlength }} def"),
        &Options::default(),
    )
    .unwrap();
    let err = match m.render_at(0.0) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("a string was taken for a plan"),
    };
    assert!(err.contains("plan"), "{err}");
}

// --- the catalog entries --------------------------------------------------

fn place(entry: &str, params: &str, at: f64) -> Frame {
    let src = format!(
        "<< /Size [400 300] /Duration 6 /FPS 10 /Background [0 0 0] >> animation
         /x /{entry} << /Params << {params} >> >> instance
         /scene {{ /x << /Box [0 0 400 300] /After /hold >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{entry}: {e}"));
    let f = m.render_at(at).unwrap_or_else(|e| panic!("{entry}: {e}"));
    Frame {
        premultiplied: f.premultiplied.to_vec(),
    }
}

fn lit(f: &Frame, pred: impl Fn(u8, u8, u8) -> bool) -> usize {
    f.premultiplied
        .chunks_exact(4)
        .filter(|p| pred(p[0], p[1], p[2]))
        .count()
}

fn mean(f: &Frame) -> f64 {
    f.premultiplied
        .chunks_exact(4)
        .map(|p| f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2]))
        .sum::<f64>()
        / f.premultiplied.len() as f64
}

#[test]
fn the_terminal_shows_the_session_as_it_unfolds_and_is_pure() {
    let params = "/Session [ [/type (ls)] [/out (one\\ntwo)] ] /Glow 0 /Grain 0";
    let early = place("terminal", params, 0.0);
    let late = place("terminal", params, 20.0);
    let (l, e) = (
        lit(&late, |_, g, _| g > 120),
        lit(&early, |_, g, _| g > 120),
    );
    assert!(l > e + 100, "more text late than early: {l} {e}");
    // The same time twice is the same picture.
    assert_eq!(
        place("terminal", params, 2.0).premultiplied,
        place("terminal", params, 2.0).premultiplied
    );
    // Phosphors differ in colour: green dominates one, red the amber.
    let green = place("terminal", &format!("{params} /Phosphor /green"), 20.0);
    let amber = place("terminal", &format!("{params} /Phosphor /amber"), 20.0);
    let g = lit(&green, |r, g, _| g > 150 && g > r);
    let a = lit(&amber, |r, g, _| r > 150 && r > g);
    assert!(g > 20 && a > 20, "{g} {a}");
    assert_eq!(lit(&green, |r, g, _| r > 150 && r > g + 40), 0);
    // The glass is dark where there is no text.
    assert!(mean(&late) < 60.0);
}

#[test]
fn persistence_leaves_a_fading_trace_when_the_screen_clears() {
    // Clear at about 1 s: with persistence the old text is still faintly there just after.
    let session = "/Session [ [/out (WWWWWWWWWWWWWWWWWWWW\\nWWWWWWWWWWWWWWWWWWWW)] [/wait 1] [/clear] [/wait 3] ] /Glow 0 /Grain 0 /Scanlines 0 /Typos 0";
    let probe = |persist: &str, t: f64| {
        mean(&place(
            "terminal",
            &format!("{session} /Persistence {persist}"),
            t,
        ))
    };
    // Find the moment of the clear from the plan.
    let setup = "/s [ [/out (W)] [/wait 1] [/clear] ] 1 << >> sessionplan def";
    let clear_at: f64 = eval_at(setup, "s planlength ==", 0.0).parse().unwrap();
    let t = clear_at + 0.06;
    let with = probe("1", t);
    let without = probe("0", t);
    assert!(
        with > without + 0.05,
        "persistence {with} vs none {without}"
    );
    // And it has gone a second later.
    let later = probe("1", clear_at + 1.5);
    assert!(later < with);
}

#[test]
fn the_font_is_one_parameter() {
    let a = place(
        "terminal",
        "/Session [ [/out (Hello)] ] /Font /Courier /Glow 0",
        5.0,
    );
    let b = place(
        "terminal",
        "/Session [ [/out (Hello)] ] /Font /Helvetica /Glow 0",
        5.0,
    );
    assert_ne!(a.premultiplied, b.premultiplied);
}

#[test]
fn a_plan_made_in_the_movie_drives_the_terminal_and_the_typist_follows_the_instance_seed() {
    let setup = "/p [ [/type (ls -la)] [/mark /m] [/out (x)] ] 4 << /Prompt (> ) /Tail 0.4 >> sessionplan def
        /a /terminal << /Params << /Plan p /Hold 2 >> >> instance
        /cmd [ [/type (git commit -m wip && git push origin main)] ] def
        /b /terminal << /Seed 1 /Params << /Session cmd >> >> instance
        /c /terminal << /Seed 2 /Params << /Session cmd >> >> instance";
    // The terminal lasts as long as the plan, plus its hold.
    let out = eval_at(setup, "/a componentduration == p planlength ==", 0.0);
    let v: Vec<f64> = out.lines().map(|l| l.parse().unwrap()).collect();
    assert!((v[0] - (v[1] + 2.0)).abs() < 1e-6, "{v:?}");
    // Two instances with other seeds type the same command at another pace.
    let out = eval_at(
        setup,
        "/b componentduration == /c componentduration ==",
        0.0,
    );
    let d: Vec<f64> = out.lines().map(|l| l.parse().unwrap()).collect();
    assert!((d[0] - d[1]).abs() > 1e-6, "{d:?}");
}
