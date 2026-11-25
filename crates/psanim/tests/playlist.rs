//! Playlists: scenes that cycle, with transitions, a schedule and a safe area.

use psanim::{Movie, Options};

const HEAD: &str = "<< /Size [200 100] /Duration 12 /FPS 10 >> animation\n";

fn load(body: &str) -> Movie {
    Movie::load(&format!("{HEAD}{body}"), &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

fn load_err(body: &str) -> String {
    match Movie::load(&format!("{HEAD}{body}"), &Options::default()) {
        Ok(_) => panic!("expected an error"),
        Err(e) => format!("{e}"),
    }
}

fn printed(m: &mut Movie, t: f64) -> String {
    m.take_output();
    m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

/// A scene that says its name and its clock when drawn.
fn probe(name: &str) -> String {
    format!("/{name} << /Size [20 10] /Duration 100 /Scene {{ [ /{name} now ] == }} >> component\n")
}

fn pixels(m: &mut Movie, t: f64) -> Vec<u8> {
    m.render_at(t).unwrap().premultiplied.to_vec()
}

#[test]
fn a_playlist_loops_without_a_seam_and_knows_its_length() {
    // Two moving scenes, a second of dissolve after each: 3 + 1 + 2 + 1.
    let mut m = load(
        "/a << /Size [100 100] /Duration 3 /Scene { 1 0 0 setrgbcolor 50 50 translate now 90 mul rotate -30 -10 60 20 rectfill } >> component
         /b << /Size [100 100] /Duration 3 /Scene { 0 1 0 setrgbcolor 50 50 translate now 45 mul rotate -10 -30 20 60 rectfill } >> component
         /show [ [3 /a] [2 /b] ] << >> playlist
         /scene { /show << >> place } def
         /d /show playlistduration def
         d 7 ne { /wrongduration missing } if",
    );
    for t in [0.0, 0.4, 2.5, 3.2, 3.9, 5.5, 6.9] {
        let a = pixels(&mut m, t);
        let b = pixels(&mut m, t + 7.0);
        let c = pixels(&mut m, t + 70.0);
        assert_eq!(a, b, "t = {t}");
        assert_eq!(a, c, "t = {t}");
    }
    // The join is as smooth as the steps beside it.
    let dist = |a: &[u8], b: &[u8]| -> u64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum()
    };
    let (e1, e0, s0, s1) = (
        pixels(&mut m, 6.98),
        pixels(&mut m, 6.99),
        pixels(&mut m, 7.0),
        pixels(&mut m, 7.01),
    );
    let (before, across, after) = (dist(&e1, &e0), dist(&e0, &s0), dist(&s0, &s1));
    assert!(
        across <= before.max(after) * 2,
        "seam {across}, steps {before} and {after}"
    );
}

#[test]
fn each_scene_runs_its_own_clock_through_the_hand_over() {
    let mut m = load(&format!(
        "{}{}/show [ [2 /a] [1 /dissolve] [2 /b] [1 /dissolve] ] << >> playlist
         /scene {{ /show << >> place }} def",
        probe("a"),
        probe("b")
    ));
    // Round: a holds 0..2 and hands over 2..3; b holds 3..5, hands over 5..6.
    // a's clock has been running since the last hand-over began, a second
    // before the loop's start.
    assert_eq!(printed(&mut m, 0.5), "[/a 1.5]");
    assert_eq!(printed(&mut m, 2.5), "[/a 3.5]\n[/b 0.5]");
    assert_eq!(printed(&mut m, 4.0), "[/b 2.0]");
    // The last hand-over shows a again, with the clock it will have next round.
    assert_eq!(printed(&mut m, 5.5), "[/b 3.5]\n[/a 0.5]");
    assert_eq!(printed(&mut m, 6.5), "[/a 1.5]");
}

#[test]
fn a_cut_is_nothing_in_between() {
    let mut m = load(&format!(
        "{}{}/show [ [2 /a] [0 /cut] [2 /b] ] << /Transition [0 /cut] >> playlist
         /scene {{ /show << >> place }} def
         /show playlistduration 4 ne {{ /wrong missing }} if",
        probe("a"),
        probe("b")
    ));
    assert_eq!(printed(&mut m, 1.75), "[/a 1.75]");
    assert_eq!(printed(&mut m, 2.25), "[/b 0.25]");
    assert_eq!(printed(&mut m, 4.25), "[/a 0.25]");
}

#[test]
fn a_schedule_chooses_the_scenes_when_a_round_starts() {
    // a is on the list from 09:00 to 17:00 only; b is always. The movie
    // starts at 08:59:50, so the first rounds are b alone, 5 s each, until
    // the round that starts at or after 09:00, which is the one at t = 10.
    let src = format!(
        "{}{}/show [ [4 /a << /From (9:00) /To 17 >>] [1 /cut] [4 /b] [1 /cut] ] << /Time (8:59:50) >> playlist
         << /Size [200 100] /Duration 60 /FPS 10 >> animation
         /scene {{ /show << >> place }} def",
        probe("a"),
        probe("b")
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    // b alone: its clock has run since the hand-over a second before.
    assert_eq!(printed(&mut m, 1.0), "[/b 2.0]");
    assert_eq!(printed(&mut m, 6.0), "[/b 2.0]");
    // The last hand-over before nine goes to a, the first scene of the next
    // round, and a's clock carries straight on into that round.
    assert_eq!(printed(&mut m, 9.5), "[/a 0.5]");
    assert_eq!(printed(&mut m, 10.5), "[/a 1.5]");
    assert_eq!(printed(&mut m, 16.0), "[/b 2.0]");
}

#[test]
fn days_and_an_empty_list() {
    // Only on weekends. The movie starts on Friday at 23:00.
    let src = format!(
        "{}/show [ [4 /a << /Days /weekend >>] ] << /Day /fri /Time 23 /Background [0 0 1] >> playlist
         /show componentduration 604800 ne {{ /wrong missing }} if /show playlistduration 0 ne {{ /wrong missing }} if
         << /Size [20 10] /Duration 4000 /FPS 1 >> animation
         /scene {{ /show << >> place }} def",
        probe("a")
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    // Friday night: nothing scheduled, just the ground.
    assert_eq!(printed(&mut m, 1.0), "");
    let f = m.render_at(1.0).unwrap();
    assert_eq!(&f.premultiplied[..4], &[0, 0, 255, 255]);
    // An hour later it is Saturday and the first round starts clean.
    assert_eq!(printed(&mut m, 3601.0), "[/a 1.0]");
}

#[test]
fn weektime_is_the_playlists_time_inside_it() {
    let src = "/clock << /Size [10 10] /Duration 100 /Scene { weektime == } >> component
        /show [ [10 /clock] ] << /Day /tue /Time (1:30) >> playlist
        << /Size [20 10] /Duration 10 >> animation
        /scene { /weektime signal == /show << >> place } def";
    let mut opts = Options::default();
    opts.signals.set("weektime", 777.0);
    let mut m = Movie::load(src, &opts).unwrap();
    // Tuesday is day 1: 86400 + 1.5 hours + the movie's clock. A playlist
    // of one scene has a transition of its own (a second of dissolve) and
    // the clock tile is drawn twice then; at 2 s it is drawn once.
    let out = printed(&mut m, 2.0);
    assert_eq!(out.lines().next().unwrap(), "777.0");
    assert_eq!(out.lines().nth(1).unwrap(), "91802.0");
}

#[test]
fn the_safe_area_keeps_scenes_inside_and_orientation_filters() {
    let mut m = load(
        "/red << /Size [10 10] /Duration 5 /Scene { 1 0 0 setrgbcolor 0 0 width height rectfill } >> component
         /show [ [5 /red << /Fill true >>] [5 { 0 1 0 setrgbcolor 0 0 width height rectfill } << /Orientation /portrait >>] ]
           << /Safe 0.1 /Background [0 0 0] >> playlist
         /scene { /show << >> place } def
         /show playlistduration 6 ne { /wrong missing } if",
    );
    // Safe 10%: the scene fills 20..180 by 10..90 of the 200 by 100 canvas
    // and the ground fills the rest. The portrait scene is not on a
    // landscape list, so the red one is alone, held 5 s plus its dissolve.
    let f = m.render_at(2.0).unwrap();
    let px = |x: u32, y: u32| {
        let i = ((y * f.width + x) * 4) as usize;
        (f.premultiplied[i], f.premultiplied[i + 1])
    };
    assert_eq!(px(100, 50), (255, 0));
    assert_eq!(px(5, 50), (0, 0), "outside the safe area is the ground");
    assert_eq!(px(100, 3), (0, 0));
    assert_eq!(px(30, 50), (255, 0));
}

#[test]
fn a_playlist_file_is_data_beside_the_movie() {
    let dir = std::env::temp_dir().join(format!("psanim-playlist-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("a.ps"),
        "<< /Size [20 10] /Duration 2 >> animation
         /Label (none) param /label exch def
         /scene { [ label now ] == } def",
    )
    .unwrap();
    std::fs::write(
        dir.join("one.list"),
        "% what plays, in the order it plays\n[ [2 (a.ps) << /Params << /Label (hello) >> >>] [1 /dissolve] ]\n",
    )
    .unwrap();
    std::fs::write(dir.join("two.list"), "[ ] [ ]\n").unwrap();
    let opts = Options {
        base_dir: Some(dir.clone()),
        read_roots: Some(vec![dir.clone()]),
        ..Options::default()
    };
    let mut m = Movie::load(
        &format!("{HEAD}/show (one.list) << >> playlist /scene {{ /show << >> place }} def"),
        &opts,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(printed(&mut m, 0.5), "[(hello) 1.5]");
    let e = Movie::load(
        &format!("{HEAD}/show (two.list) << >> playlist /scene {{ }} def"),
        &opts,
    )
    .err()
    .unwrap();
    assert!(format!("{e}").contains("one array"), "{e}");
    let e = Movie::load(
        &format!("{HEAD}/show (missing.list) << >> playlist /scene {{ }} def"),
        &opts,
    )
    .err()
    .unwrap();
    assert!(!format!("{e}").is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn mistakes_are_explained() {
    let e = load_err("/s [ [1 /dissolve] [2 { }] ] << >> playlist");
    assert!(e.contains("after a scene"), "{e}");
    let e = load_err("/s [ [2 /nosuchthing] ] << >> playlist");
    assert!(e.contains("nosuchthing") && e.contains("catalog"), "{e}");
    let e = load_err("/s [ [2 { }] ] << /Safe 0.9 >> playlist");
    assert!(e.contains("/Safe"), "{e}");
    let e = load_err("/s [ [2 { } << /Days /someday >>] ] << >> playlist");
    assert!(e.contains("a day is"), "{e}");
    let e = load_err("/s [ [2 { }] [1 /dissolve] ] << /Transition [1 /nothing] >> playlist");
    assert!(e.contains("/Transition"), "{e}");
    let e = load_err("/s [ ] << >> playlist");
    assert!(e.contains("no scenes"), "{e}");
    let e = load_err("/s [ [0 { }] ] << >> playlist");
    assert!(e.contains("more than 0"), "{e}");
}

#[test]
fn a_catalog_entry_and_a_procedure_are_scenes() {
    let mut m = load(
        "/hello { 1 1 1 setrgbcolor 0 0 width height rectfill } def
         /show [ [2 /spinner << /Params << /Style /dots >> >>] [1 /iris] [2 /hello] ] << >> playlist
         /scene { /show << >> place } def",
    );
    let a = pixels(&mut m, 1.0);
    let b = pixels(&mut m, 4.0);
    assert_ne!(a, b);
}

#[test]
fn a_live_host_can_supply_the_time() {
    let src = "/clock << /Size [10 10] /Duration 100 /Scene { weektime == } >> component
        /show [ [10 /clock] ] << /Clock /signal >> playlist
        << /Size [20 10] /Duration 10 >> animation
        /scene { /show << >> place } def";
    let mut opts = Options::default();
    opts.signals.set("weektime", 5000.5);
    let mut m = Movie::load(src, &opts).unwrap();
    assert_eq!(printed(&mut m, 2.0), "5000.5");
}

#[test]
fn a_programs_own_transition_and_a_lunchtime_offset() {
    // `t-` names are transitions even before they are defined.
    let src = format!(
        "{}{}/show [ [2 /a] [1 /t-mine] [2 /b] [1 /t-mine] ] << >> playlist
         transitions /t-mine {{ pop pop exec }} put
         << /Size [200 100] /Duration 6 /FPS 10 >> animation
         /scene {{ /show << >> place }} def",
        probe("a"),
        probe("b")
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(printed(&mut m, 2.5), "[/a 3.5]");
    // A scheduled playlist a week long, opened on a Wednesday at 12:00 as a
    // page would (the player seeks to the present): the lunch scene is on.
    let src = format!(
        "{}{}/show [ [4 /a] [1 /cut] [4 /b << /Days /weekdays /From (11:30) /To (14:00) >>] [1 /cut] ] << >> playlist
         << /Size [200 100] /Duration /show componentduration /Loop true /FPS 10 >> animation
         /scene {{ /show << >> place }} def",
        probe("a"),
        probe("b")
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    let wed_noon = 2.0 * 86_400.0 + 12.0 * 3600.0;
    // Rounds of a and b are 10 s: 4 + 1 + 4 + 1. Inside the first round that
    // starts at noon, b is on its way in.
    assert_eq!(printed(&mut m, wed_noon + 7.0), "[/b 3.0]");
    // The same offset on a Saturday has only a.
    let sat_noon = 5.0 * 86_400.0 + 12.0 * 3600.0;
    assert_eq!(printed(&mut m, sat_noon + 2.0), "[/a 3.0]");
}
