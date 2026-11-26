//! The catalog: every entry is a movie on its own and a component in another.

use psanim::{CATALOG, Movie, Options, catalog};

fn has_ink(px: &[u8]) -> bool {
    px.chunks_exact(4).any(|p| p[3] > 0)
}

#[test]
fn every_entry_runs_on_its_own_and_says_what_it_is() {
    assert!(!CATALOG.is_empty());
    for (name, src) in CATALOG {
        let mut m = Movie::load(src, &Options::default())
            .unwrap_or_else(|e| panic!("/{name} does not load: {e}"));
        let meta = m.meta().clone();
        assert!(meta.title.is_some(), "/{name} declares no /Title");
        assert!(
            meta.description.is_some(),
            "/{name} declares no /Description"
        );
        let notes = catalog::param_notes(src);
        for p in meta.params.keys() {
            assert!(
                notes.get(p).is_some_and(|n| !n.is_empty()),
                "/{name}: parameter /{p} has no comment saying what it is"
            );
        }
        for k in psanim::sheet::spread(meta.frames, 5) {
            let f = m
                .render_frame(k)
                .unwrap_or_else(|e| panic!("/{name} frame {k}: {e}"));
            // A film may open from nothing: only the later frames must show something.
            assert!(
                k == 0 || has_ink(f.premultiplied),
                "/{name} frame {k} is empty"
            );
        }
    }
}

#[test]
fn every_entry_places_as_a_component_inside_its_box() {
    for (name, _) in CATALOG {
        let src = format!(
            "<< /Size [400 200] /Duration 2 /FPS 10 /Background /transparent >> animation
             /x /{name} component
             /scene {{ /x << /Box [200 0 200 200] /After /hold >> place }} def"
        );
        let mut m = Movie::load(&src, &Options::default())
            .unwrap_or_else(|e| panic!("/{name} as a component: {e}"));
        let f = m.render_at(1.0).unwrap_or_else(|e| panic!("/{name}: {e}"));
        let w = f.width as usize;
        let left_empty = f
            .premultiplied
            .chunks_exact(4)
            .enumerate()
            .all(|(i, p)| i % w >= w / 2 || p[3] == 0);
        assert!(left_empty, "/{name} drew outside its box");
        assert!(has_ink(f.premultiplied), "/{name} drew nothing in its box");
    }
}

#[test]
fn the_reference_lists_every_entry() {
    let md = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../skills/psanim/references/catalog.md"),
    )
    .unwrap();
    for (name, _) in CATALOG {
        assert!(
            md.contains(&format!("## `/{name}`")),
            "catalog.md does not list /{name}: run scripts/gen_reference.sh"
        );
    }
}

#[test]
fn an_unknown_entry_names_the_ones_there_are() {
    let e = Movie::load(
        "<< /Size [10 10] >> animation /x /nosuch component /scene { } def",
        &Options::default(),
    )
    .err()
    .expect("an error")
    .to_string();
    assert!(e.contains("/spinner"), "{e}");
}

// ---- /fire ------------------------------------------------------------------

/// A movie that places one fire instance, over `fill` seconds of it.
fn fire_movie(style: &str, seed: u32, params: &str, duration: u32) -> Movie {
    let src = format!(
        "<< /Size [300 400] /Duration {duration} /FPS 10 /Background [0 0 0] >> animation
         /fire /fire component
         /f /fire << /Seed {seed} /Params << /Style /{style} {params} >> >> instance
         /scene {{ /f << /Box [0 0 300 400] >> place }} def"
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("fire {style}: {e}"))
}

fn mean_diff(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(x.abs_diff(*y)))
        .sum::<f64>()
        / a.len() as f64
}

#[test]
fn fire_loops_exactly_in_both_styles() {
    for style in ["realistic", "stylized"] {
        let mut m = fire_movie(style, 3, "", 9);
        // The entry's period is 4 s: the same moment one loop later is the same picture.
        for t in [0.0, 1.3, 2.9] {
            let a = m.render_at(t).unwrap().premultiplied.to_vec();
            let b = m.render_at(t + 4.0).unwrap().premultiplied.to_vec();
            assert!(
                mean_diff(&a, &b) < 0.05,
                "{style}: t={t} differs from t={} by {}",
                t + 4.0,
                mean_diff(&a, &b)
            );
        }
        // And it moves: half a second apart is not the same picture.
        let a = m.render_at(1.0).unwrap().premultiplied.to_vec();
        let b = m.render_at(1.5).unwrap().premultiplied.to_vec();
        assert!(mean_diff(&a, &b) > 0.3, "{style} does not move");
    }
}

#[test]
fn fire_seeds_flicker_differently() {
    for style in ["realistic", "stylized"] {
        let a = fire_movie(style, 1, "", 4)
            .render_at(1.2)
            .unwrap()
            .premultiplied
            .to_vec();
        let b = fire_movie(style, 2, "", 4)
            .render_at(1.2)
            .unwrap()
            .premultiplied
            .to_vec();
        assert!(mean_diff(&a, &b) > 0.5, "{style}: two seeds look the same");
    }
}

#[test]
fn fire_stays_inside_its_box_at_the_extremes() {
    // Tallest, widest, windiest, in both directions, in the right half of a wide movie.
    for style in ["realistic", "stylized"] {
        for wind in ["1", "-1"] {
            let src = format!(
                "<< /Size [600 400] /Duration 4 /FPS 10 /Background /transparent >> animation
                 /fire /fire component
                 /f /fire << /Params << /Style /{style} /Height 0.8 /Width 0.8 /Wind {wind}
                    /Intensity 2 /Embers 3 /Smoke 1 >> >> instance
                 /scene {{ /f << /Box [300 0 300 400] >> place }} def"
            );
            let mut m = Movie::load(&src, &Options::default()).unwrap();
            for t in [0.4, 1.7, 3.1] {
                let f = m.render_at(t).unwrap();
                let w = f.width as usize;
                // The glow is light: its faint tail may reach past the box, but nothing solid does.
                assert!(
                    f.premultiplied
                        .chunks_exact(4)
                        .enumerate()
                        .all(|(i, p)| i % w + 40 >= w / 2 || p[3] < 12),
                    "{style} wind {wind} t={t}: drew outside its box"
                );
                assert!(has_ink(f.premultiplied));
            }
        }
    }
}

#[test]
fn fire_reaches_a_candle_a_campfire_and_a_torch() {
    let extremes = [
        "/Height 0.26 /Width 0.07 /Intensity 0.45 /Embers 0 /Smoke 0.1",
        "/Height 0.5 /Width 0.8 /Intensity 1.3 /Embers 1.5 /Smoke 0.5",
        "/Height 0.6 /Width 0.3 /Wind 0.55 /Embers 0.8 /Smoke 0.2",
    ];
    let mut lit = Vec::new();
    for params in extremes {
        let mut m = fire_movie("realistic", 5, params, 4);
        let f = m.render_at(1.0).unwrap();
        let n = f
            .premultiplied
            .chunks_exact(4)
            .filter(|p| p[0] > 128)
            .count();
        assert!(n > 100, "{params}: nearly nothing drawn");
        lit.push(n);
    }
    // A candle is much smaller than a campfire.
    assert!(lit[0] * 3 < lit[1], "{lit:?}");
}

#[test]
fn a_celebrations_bursts_follow_its_bursts_param() {
    // A single shell bursting at 2 s: nothing before it rises, stars after.
    let src = "<< /Size [320 180] /Duration 4 /Background [0 0 0] >> animation
        /fw /fireworks << /Params << /Style /single /Bursts [2] /Message () /Detail () /Sky false >> >> instance
        /scene { /fw << >> place } def";
    let mut m = Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    let ink = |m: &mut Movie, t: f64| {
        let f = m.render_at(t).unwrap_or_else(|e| panic!("{e}"));
        f.premultiplied
            .chunks(4)
            .filter(|p| p[0] > 40 || p[1] > 40 || p[2] > 40)
            .count()
    };
    let before = ink(&mut m, 0.3);
    let after = ink(&mut m, 2.3);
    assert_eq!(before, 0);
    assert!(after > 200, "{after} lit pixels after the burst");
    // Confetti blasts on its own track too.
    let src = "<< /Size [320 180] /Duration 4 /Background [0 0 0] >> animation
        /cf /confetti << /Params << /Style /burst /Bursts [1.5] /Message () /Detail () /Backdrop /none >> >> instance
        /scene { /cf << >> place } def";
    let mut m = Movie::load(src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ink(&mut m, 1.4), 0);
    assert!(ink(&mut m, 1.8) > 200);
}

/// A movie with one Twiddle placed with these params, and its signals.
fn twiddle(params: &str, csv: Option<&str>) -> Movie {
    let src = format!(
        "<< /Size [200 200] /Duration 12 /FPS 10 /Background [0 0 0] >> animation
         /tw /twiddle << /Params << {params} >> >> instance
         /scene {{ /tw << /After /hold >> place }} def"
    );
    let mut opts = Options::default();
    if let Some(csv) = csv {
        opts.signals = psanim::Signals::from_csv(csv).unwrap();
    }
    Movie::load(&src, &opts).unwrap_or_else(|e| panic!("{e}"))
}

fn frame_at(m: &mut Movie, t: f64) -> Vec<u8> {
    m.render_at(t)
        .unwrap_or_else(|e| panic!("{e}"))
        .premultiplied
        .to_vec()
}

#[test]
fn twiddle_dances_only_to_music_and_sleeps_still_without_it() {
    // To a tempo he moves.
    let mut m = twiddle("", None);
    assert_ne!(frame_at(&mut m, 1.1), frame_at(&mut m, 1.3));
    // With no music he sleeps, and the picture is still.
    for p in ["/Tempo 0", "/State /sleep", "/Beat /beat"] {
        let mut m = twiddle(p, None);
        let a = frame_at(&mut m, 1.0);
        assert_eq!(a, frame_at(&mut m, 2.7), "{p}: asleep should be still");
        assert_eq!(a, frame_at(&mut m, 9.0), "{p}: asleep should be still");
    }
    // When the music stops he settles, then is asleep and still.
    let mut m = twiddle("/Until 4", None);
    assert_ne!(frame_at(&mut m, 3.1), frame_at(&mut m, 3.3));
    assert_eq!(frame_at(&mut m, 6.0), frame_at(&mut m, 9.0));
    // An event track from --signals is a beat too; when it stops, so does he.
    let csv = "t,level,beat!\n0,0.9,x\n0.5,0.5,x\n1,0.9,x\n1.5,0.5,x\n2,0.9,x\n2.5,0.5,x\n";
    let mut m = twiddle("/Beat /beat /Level /level", Some(csv));
    assert_ne!(frame_at(&mut m, 1.1), frame_at(&mut m, 1.3));
    assert_eq!(frame_at(&mut m, 6.0), frame_at(&mut m, 9.0));
}

#[test]
fn twiddle_on_a_splash_waves_but_never_hops() {
    let mut m = twiddle("/State /wave", None);
    // His feet stay on the ground: the lowest painted row never changes.
    let lowest = |px: &[u8]| {
        (0..200)
            .rev()
            .find(|y| (0..200).any(|x| px[(y * 200 + x) * 4 + 1] > 20))
            .unwrap()
    };
    let first = frame_at(&mut m, 0.0);
    let mut moved = false;
    for k in 1..20 {
        let f = frame_at(&mut m, k as f64 * 0.23);
        assert_eq!(lowest(&f), lowest(&first), "he hopped on a splash");
        moved |= f != first;
    }
    assert!(moved, "on a splash he still waves and blinks");
}

#[test]
fn there_is_only_one_twiddle() {
    let src = "<< /Size [200 100] /Duration 1 >> animation
        /a /twiddle component
        /scene { /a << /Box [0 0 100 100] >> place /a << /Box [100 0 100 100] >> place } def";
    let mut m = Movie::load(src, &Options::default()).unwrap();
    let e = m
        .render_at(0.5)
        .map(|_| ())
        .expect_err("two Twiddles")
        .to_string();
    assert!(e.contains("only one Twiddle"), "{e}");
}

/// One catalog entry placed alone in a 800x300 movie with these params,
/// drawn at `t`.
fn entry_at(entry: &str, params: &str, t: f64) -> Vec<u8> {
    entry_sized(entry, "800 300", params, t)
}

fn entry_sized(entry: &str, size: &str, params: &str, t: f64) -> Vec<u8> {
    let src = format!(
        "<< /Size [{size}] /Duration 30 /FPS 10 >> animation
         /x /{entry} << /Params << {params} >> >> instance
         /scene {{ /x << /After /hold >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.render_at(t)
        .unwrap_or_else(|e| panic!("{e}"))
        .premultiplied
        .to_vec()
}

#[test]
fn a_splitflap_board_settles_to_the_same_picture_however_it_got_there() {
    // Shown from the start, and flipped there from blank: once the last flap has
    // landed (and bounced) the two boards are the same picture.
    let direct = entry_at("splitflap", "/States [ [ 0 [ (HELLO 42) (OK) ] ] ]", 0.0);
    let flipped = entry_at("splitflap", "/States [ [ 1 [ (HELLO 42) (OK) ] ] ]", 29.0);
    assert!(
        direct == flipped,
        "a settled board differs from a fresh one"
    );
    // Mid-flip it is something else, and before its first state it is blank.
    let mid = entry_at("splitflap", "/States [ [ 1 [ (HELLO 42) (OK) ] ] ]", 1.5);
    let blank = entry_at("splitflap", "/States [ [ 1 [ (HELLO 42) (OK) ] ] ]", 0.5);
    assert!(mid != direct && mid != blank && blank != direct);
}

#[test]
fn a_splitflap_cell_flips_forward_through_its_drum_only() {
    // 'A' to 'B' is one flap; 'B' to 'A' goes all the way round, so a moment
    // after both start, the second is still turning while the first has landed.
    let params = |a: &str, b: &str| {
        format!("/States [ [ 0 [ ({a}) ] ] [ 1 [ ({b}) ] ] ] /Scatter 0 /Rate 10")
    };
    let one = entry_at("splitflap", &params("A", "B"), 1.45);
    let one_done = entry_at("splitflap", &params("A", "B"), 5.0);
    let far = entry_at("splitflap", &params("B", "A"), 1.45);
    let far_done = entry_at("splitflap", &params("B", "A"), 5.0);
    assert!(one == one_done, "one flap should have landed by 0.45 s");
    assert!(
        far != far_done,
        "a full lap of the drum is still turning at 0.45 s"
    );
}

#[test]
fn an_odometer_carries_only_as_the_wheels_below_roll_past_nine() {
    // Four wheels, no commas: the movie is the entry's own 200x88.
    let at = |v: &str, t: f64| {
        let p = format!("/From {v} /To {v} /Digits 4 /Commas false /Over 1");
        entry_sized("odometer", "200 88", &p, t)
    };
    // The thousands wheel is the first wheel's columns, x = 14..54.
    let thousands = |px: &[u8]| -> Vec<u8> {
        (0..88)
            .flat_map(|y| px[(y * 200 + 14) * 4..(y * 200 + 54) * 4].to_vec())
            .collect()
    };
    let rest = thousands(&at("1998", 0.0));
    assert!(
        rest == thousands(&at("1998.5", 0.0)),
        "the thousands wheel moved before the carry"
    );
    assert!(
        rest != thousands(&at("1999.5", 0.0)),
        "the thousands wheel did not start to carry"
    );
    assert!(at("1999", 0.0) != at("2000", 0.0));
    // A number counted to settles to the same picture as one set there.
    let p = "/From 0 /To 1234 /Digits 4 /Commas false /Over 2";
    assert!(entry_sized("odometer", "200 88", p, 29.0) == at("1234", 0.0));
}

// ---- /ledmatrix ----------------------------------------------------------------

/// A raw LED movie of a grid, as the host's parameters would give it.
fn raw_led(cols: u32, rows: u32, extra: &[(&str, psanim::Param)]) -> Movie {
    let mut o = Options::default();
    for (k, v) in [("Raw", 1.0), ("Cols", cols as f64), ("Rows", rows as f64)] {
        o.params.insert(k.into(), psanim::Param::Number(v));
    }
    for (k, v) in extra {
        o.params.insert((*k).into(), v.clone());
    }
    Movie::load(psanim::catalog::source("ledmatrix").unwrap(), &o).unwrap()
}

/// The picture of a raw LED movie at `t` as rows of '#' (lit) and '.'.
fn led_rows(cols: u32, rows: u32, extra: &[(&str, psanim::Param)], t: f64) -> Vec<String> {
    let mut m = raw_led(cols, rows, extra);
    let f = m.render_at(t).unwrap();
    let w = cols as usize;
    (0..rows as usize)
        .map(|y| {
            (0..w)
                .map(|x| {
                    if f.premultiplied[(y * w + x) * 4] > 0 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}

fn text(s: &str) -> psanim::Param {
    psanim::Param::Text(s.into())
}

#[test]
fn a_raw_led_movie_is_the_grid_itself() {
    let src = psanim::catalog::source("ledmatrix").unwrap();
    let mut o = Options::default();
    for (k, v) in [("Raw", 1.0), ("Cols", 65.0), ("Rows", 5.0)] {
        o.params.insert(k.into(), psanim::Param::Number(v));
    }
    let mut m = Movie::load(src, &o).unwrap();
    assert_eq!((m.meta().width, m.meta().height), (65, 5));
    let f = m.render_at(0.3).unwrap();
    assert_eq!((f.width, f.height), (65, 5));
    // One canvas unit an LED: pure on or off, one colour, nothing in between.
    let colours: std::collections::BTreeSet<[u8; 4]> = f
        .premultiplied
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect();
    assert!(
        colours.len() <= 2,
        "raw frames hold more than on and off: {colours:?}"
    );
    assert!(colours.iter().all(|c| c[3] == 255));
}

#[test]
fn the_pixel_font_sets_text_exactly() {
    let still = |t: &str, rows: u32| {
        led_rows(
            7,
            rows,
            &[("Text", text(t)), ("Effect", text("static"))],
            0.1,
        )
    };
    assert_eq!(
        still("A", 7),
        [
            ".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"
        ]
        .map(|r| format!(".{r}."))
    );
    // Five rows use the lower font; lower case falls back to capitals there.
    assert_eq!(still("a", 5), still("A", 5));
    assert!(still("a", 5).iter().any(|r| r.contains('#')));
}

#[test]
fn a_marquee_is_a_seamless_loop_of_the_texts_own_length() {
    let mut m = raw_led(20, 7, &[("Text", text("HI THERE"))]);
    assert!(m.meta().looping);
    let dur = m.meta().duration;
    let first = m.render_at(0.0).unwrap().premultiplied.to_vec();
    let again = m.render_at(dur).unwrap().premultiplied.to_vec();
    let mid = m.render_at(dur / 2.0).unwrap().premultiplied.to_vec();
    assert!(
        first == again,
        "the marquee does not come back to where it began"
    );
    assert!(first != mid);
}

#[test]
fn a_repeating_message_list_closes_its_loop_exactly() {
    // /Items repeat by default: the movie is exactly one round long, so the
    // frame after the last is the first. (A list cannot come in through the
    // command line, so the entry's own default is replaced in its source.)
    let src = psanim::catalog::source("ledmatrix").unwrap().replace(
        "/Items null param",
        "/Items [ [ (AB) /fade 1 ] [ (CD) /static 0.95 ] ] param",
    );
    let mut o = Options::default();
    for (k, v) in [("Raw", 1.0), ("Cols", 14.0), ("Rows", 7.0)] {
        o.params.insert(k.into(), psanim::Param::Number(v));
    }
    let mut m = Movie::load(&src, &o).unwrap();
    assert!(m.meta().looping);
    let dur = m.meta().duration;
    // The round is 4.11 s, not a tenth: rounded up to 4.2 the last frames would
    // replay the start of the next round and the loop would stutter.
    assert!((dur - 4.11).abs() < 1e-9, "duration {dur}");
    let first = m.render_at(0.0).unwrap().premultiplied.to_vec();
    let again = m.render_at(dur).unwrap().premultiplied.to_vec();
    assert!(first == again, "the message list does not close its loop");
}

#[test]
fn messages_follow_one_another_and_effects_settle_to_the_plain_text() {
    let items = |fx: &str| {
        // A list needs a real array, which only a component's /Params can carry.
        format!("/Items [ [ (AB) /{fx} 1 ] [ (CD) /static 1 ] ] /Between 0.5 /Repeat false")
    };
    let picture = |fx: &str, t: f64| -> Vec<u8> {
        entry_sized(
            "ledmatrix",
            "14 7",
            &format!("/Raw true /Grid [14 7] {}", items(fx)),
            t,
        )
    };
    let plain = |t: &str| {
        entry_sized(
            "ledmatrix",
            "14 7",
            &format!("/Raw true /Grid [14 7] /Text ({t}) /Effect /static"),
            0.1,
        )
    };
    let dark = [0u8, 0, 0, 255].repeat(14 * 7);
    assert!(picture("static", 0.5) == plain("AB"));
    assert!(picture("static", 1.2) == dark, "not dark between messages");
    assert!(picture("static", 1.8) == plain("CD"));
    // Every effect begins as something else, and ends (after its entry) as the text.
    for fx in ["drop", "wipe", "sparkle", "flash", "fade"] {
        assert!(
            picture(fx, 0.1) != plain("AB"),
            "/{fx} begins as the plain text"
        );
        assert!(
            picture(fx, 1.3) == plain("AB"),
            "/{fx} did not settle to the text"
        );
    }
}

#[test]
fn a_cell_procedure_draws_a_pattern_on_the_grid() {
    // A checkerboard: lit where column + row is odd.
    let px = entry_sized(
        "ledmatrix",
        "4 3",
        "/Raw true /Grid [4 3] /Text () /Cell { add 2 mod }",
        0.1,
    );
    let rows: Vec<String> = (0..3)
        .map(|y| {
            (0..4)
                .map(|x| if px[(y * 4 + x) * 4] > 0 { '#' } else { '.' })
                .collect()
        })
        .collect();
    // Row 0 is the top; lit where col + row is odd.
    assert_eq!(rows, [".#.#", "#.#.", ".#.#"]);
}

/// Every glyph in the pkpixel library is the right height and a rectangle, and
/// the 5 and 7 row fonts have every printable character (the 9-row table only
/// replaces the descenders of the 7-row one).
#[test]
fn the_pixel_font_is_complete_and_regular() {
    let src = psanim::LIBRARIES
        .iter()
        .find(|l| l.0 == "pkpixel")
        .expect("the library")
        .1;
    let mut fonts: std::collections::BTreeMap<usize, std::collections::BTreeSet<u8>> =
        Default::default();
    let mut height = 0;
    for line in src.lines() {
        if line.starts_with("/.pxglyphs") {
            height = match &line[10..11] {
                "7" => 7,
                "5" => 5,
                _ => 9,
            };
            continue;
        }
        let Some(rest) = line.strip_prefix("  [ (") else {
            continue;
        };
        if height == 0 {
            continue;
        }
        // Strings of the line: a backslash escapes the next character.
        let mut strings = Vec::new();
        let mut cur = String::new();
        let mut open = true;
        let mut esc = false;
        let mut bytes_key: Option<u8> = None;
        for ch in rest.chars() {
            if open {
                if esc {
                    esc = false;
                    if ch.is_ascii_digit() {
                        cur.push(ch);
                        continue;
                    }
                    cur.push(ch);
                } else if ch == '\\' {
                    esc = true;
                } else if ch == ')' {
                    strings.push(std::mem::take(&mut cur));
                    open = false;
                } else {
                    cur.push(ch);
                }
            } else if ch == '(' {
                open = true;
            }
        }
        let key = &strings[0];
        let rows = &strings[1..];
        let code = if key.len() == 3 && key.chars().all(|c| c.is_ascii_digit()) {
            u8::from_str_radix(key, 8).unwrap()
        } else {
            assert_eq!(key.len(), 1, "glyph key {key:?}");
            key.as_bytes()[0]
        };
        bytes_key.replace(code);
        assert_eq!(rows.len(), height, "glyph {key:?} has {} rows", rows.len());
        assert!(
            rows.iter().all(|r| r.len() == rows[0].len()),
            "glyph {key:?} is ragged"
        );
        assert!(
            rows.iter().all(|r| r.chars().all(|c| c == '.' || c == '#')),
            "glyph {key:?}"
        );
        assert!(
            fonts.entry(height).or_default().insert(code),
            "glyph {key:?} twice"
        );
    }
    for c in 32u8..=126 {
        assert!(
            fonts[&7].contains(&c),
            "the 7-row font lacks {:?}",
            c as char
        );
        if !c.is_ascii_lowercase() {
            assert!(
                fonts[&5].contains(&c),
                "the 5-row font lacks {:?}",
                c as char
            );
        }
    }
    assert!(
        fonts[&7].contains(&176) && fonts[&5].contains(&176),
        "no degree sign"
    );
    assert!(
        fonts[&9].is_subset(&fonts[&7]),
        "a 9-row glyph with no 7-row one to replace"
    );
}

// --- chat ------------------------------------------------------------------

fn chat_duration(params: &str) -> f64 {
    let src = format!(
        "<< /Size [40 80] /Duration 1 >> animation
         /x /chat << /Params << {params} >> >> instance
         /scene {{ /x componentduration = }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.take_output();
    m.render_at(0.0).unwrap();
    m.take_output().trim().parse().unwrap()
}

#[test]
fn chat_length_follows_the_script_and_the_pace() {
    // explicit time: arrives at 2, then rests for the hold
    let at = chat_duration("/Script [ [ /them (hi) << /At 2 /Typing 1 >> ] ] /Hold 1");
    assert!((at - 3.0).abs() < 1e-9, "{at}");
    // typing time follows the length of the message ...
    let short = chat_duration("/Script [ [ /them (ok) ] ] /Hold 0");
    let long = chat_duration(
        "/Script [ [ /them (this one goes on for quite a while, with plenty to say about it) ] ] /Hold 0",
    );
    assert!(long > short + 1.0, "{short} {long}");
    // ... and is overridden, and /Pace speeds up the default but not the override
    let fixed = chat_duration(
        "/Script [ [ /them (this one goes on and on and on and on) << /Typing 0.5 >> ] ] /Hold 0 /Lead 0",
    );
    assert!((fixed - 0.5).abs() < 1e-9, "{fixed}");
    let slow =
        chat_duration("/Script [ [ /them (a fairly long message to be typed) ] ] /Hold 0 /Pace 1");
    let quick =
        chat_duration("/Script [ [ /them (a fairly long message to be typed) ] ] /Hold 0 /Pace 2");
    assert!(quick < slow, "{quick} {slow}");
    // a numeric third element is the pause before the next one starts typing
    let a = chat_duration("/Script [ [ /them (hi) ] [ /me (yo) 0.5 ] ] /Hold 0");
    let b = chat_duration("/Script [ [ /them (hi) ] [ /me (yo) 3.5 ] ] /Hold 0");
    assert!((b - a - 3.0).abs() < 1e-9, "{a} {b}");
}

fn chat_frame(params: &str, t: f64) -> Vec<u8> {
    let src = format!(
        "<< /Size [200 400] /Duration 20 /FPS 10 /Background [0 0 0] >> animation
         /x /chat << /Params << {params} >> >> instance
         /scene {{ /x << >> place }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.render_at(t).unwrap().premultiplied.to_vec()
}

#[test]
fn chat_shows_typing_then_the_message_then_a_reaction() {
    let plain = "/Script [ [ /them (see you at eight :party:) << /At 3 /Typing 1.5 >> ] ] /Hold 4";
    let reacted = "/Script [ [ /them (see you at eight :party:) << /At 3 /Typing 1.5 /React /heart /ReactAt 1 >> ] ] /Hold 4";
    let before = chat_frame(plain, 0.2);
    let dots = chat_frame(plain, 2.0);
    let arrived = chat_frame(plain, 3.8);
    assert_ne!(before, dots, "the typing indicator did not appear");
    assert_ne!(dots, arrived, "the message did not arrive");
    // nothing more happens to a plain message once it has arrived ...
    assert_eq!(chat_frame(plain, 3.8), chat_frame(plain, 5.5));
    // ... but a reaction (arrived 3 + 1, popped by 4.5) appears on the bubble
    assert_eq!(chat_frame(reacted, 3.6), chat_frame(reacted, 3.8));
    assert_ne!(chat_frame(reacted, 3.8), chat_frame(reacted, 5.5));
}

#[test]
fn chat_scrolls_to_keep_the_newest_message_in_view() {
    // twelve messages cannot fit: the first must have gone by the end, and the frame
    // just before an arrival differs from the frame just after it everywhere it scrolls.
    let msgs: String = (0..12)
        .map(|i| {
            format!(
                "[ {} (message number {i} with a few words) << /At {} /Typing 0.5 >> ]",
                if i % 2 == 0 { "/them" } else { "/me" },
                2 + i * 2
            )
        })
        .collect();
    let p = format!("/Script [ {msgs} ] /Hold 2 /Frame false /Header false /Composer false");
    let start = chat_frame(&p, 2.6);
    let end = chat_frame(&p, 27.0);
    // the bubble at the top of the list is a "them" bubble at first; at the end the
    // top rows hold later messages, so the two frames differ at the top
    let w = 200 * 4;
    let top = |f: &[u8]| f[w * 20..w * 60].to_vec();
    assert_ne!(top(&start), top(&end));
    // smooth: a run of frames across one arrival changes a little at a time
    let mut prev = chat_frame(&p, 21.0);
    let mut biggest = 0usize;
    for k in 1..=20 {
        let f = chat_frame(&p, 21.0 + k as f64 * 0.05);
        let d = f.iter().zip(&prev).filter(|(a, b)| a != b).count();
        biggest = biggest.max(d);
        prev = f;
    }
    assert!(
        biggest < 200 * 400 * 4 / 2,
        "a jump: {biggest} bytes changed in one frame"
    );
}

#[test]
fn chat_themes_restyle_and_a_dictionary_overrides_one_colour() {
    let s = "/Script [ [ /me (hello) << /At 1 /Typing 0.5 >> ] ] /Hold 1";
    let light = chat_frame(&format!("{s} /Theme /light"), 3.0);
    let dark = chat_frame(&format!("{s} /Theme /dark"), 3.0);
    let custom = chat_frame(&format!("{s} /Theme << /me [1 0 0] >>"), 3.0);
    assert_ne!(light, dark);
    assert_ne!(light, custom);
}

#[test]
fn signal_entries_take_the_signal_name_as_text_and_value_entries_need_no_labels() {
    // As --param or the browser gives it: text, not a name.
    for entry in ["counter", "gauge", "stat"] {
        let mut opts = Options::default();
        opts.params
            .insert("Signal".into(), psanim::Param::Text("cpu".into()));
        opts.signals.set("cpu", 42.0);
        let src = psanim::catalog::source(entry).unwrap();
        let mut m = Movie::load(src, &opts).unwrap_or_else(|e| panic!("/{entry}: {e}"));
        m.render_at(1.0)
            .unwrap_or_else(|e| panic!("/{entry} with Signal=(cpu): {e}"));
    }
    // /Values alone, no /Signals and no /Labels.
    for entry in ["status", "timeseries"] {
        let src = format!(
            "<< /Size [400 300] /Duration 2 /FPS 10 >> animation
             /x /{entry} << /Params << /Values [[1 2 3] [3 2 1]] >> >> instance
             /scene {{ /x << >> place }} def"
        );
        let mut m = Movie::load(&src, &Options::default())
            .unwrap_or_else(|e| panic!("/{entry} with /Values only: {e}"));
        m.render_at(0.5).unwrap();
    }
}

#[test]
fn alignpath_refuses_an_outline_too_long_to_compare() {
    let mut m = Movie::load(
        "<< /Size [64 48] /Duration 1 /FPS 10 >> animation
         /big [ 0 1 9999 { dup } for ] def
         /scene { big big alignpath pop } def",
        &Options::default(),
    )
    .unwrap();
    let e = m.render_at(0.0).unwrap_err().to_string();
    assert!(e.contains("at most"), "{e}");
}

#[test]
fn theme_given_as_text_selects_the_theme() {
    for entry in [
        "stat",
        "gauge",
        "status",
        "counter",
        "horizon",
        "timeseries",
    ] {
        let mut opts = Options::default();
        opts.params
            .insert("Theme".into(), psanim::Param::Text("paper".into()));
        let src = psanim::catalog::source(entry).unwrap();
        let mut m =
            Movie::load(src, &opts).unwrap_or_else(|e| panic!("/{entry} Theme=(paper): {e}"));
        m.render_at(0.5).unwrap();
    }
}

// ---- /automaton -------------------------------------------------------------

fn automaton(params: &str) -> Movie {
    automaton_seeded(1, params)
}

fn automaton_seeded(seed: u32, params: &str) -> Movie {
    let src = format!(
        "<< /Size [320 200] /Duration 6 /FPS 10 /Background [0 0 0] >> animation
         /a /automaton << /Seed {seed} /Params << {params} >> >> instance
         /scene {{ /a << /Box [0 0 320 200] >> place }} def"
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn automaton_is_the_same_film_in_any_order() {
    let params = "/Pattern /rpentomino /Grid [80 50] /Edge /open /Rate 40 /Pitch 4";
    let mut seq = automaton(params);
    let mut late = None;
    for k in 0..=50 {
        let f = seq.render_at(f64::from(k) * 0.1).unwrap();
        if k == 50 {
            late = Some(f.premultiplied.to_vec());
        }
    }
    let mut cold = automaton(params);
    let at5 = cold.render_at(5.0).unwrap().premultiplied.to_vec();
    assert_eq!(
        late.unwrap(),
        at5,
        "seeking to 5 s gives the frame playing to it gives"
    );
    let back = cold.render_at(1.0).unwrap().premultiplied.to_vec();
    let fresh = automaton(params)
        .render_at(1.0)
        .unwrap()
        .premultiplied
        .to_vec();
    assert_eq!(back, fresh, "and seeking back gives the same again");
}

#[test]
fn automaton_pattern_and_seed_change_the_film() {
    let a = automaton("").render_at(2.0).unwrap().premultiplied.to_vec();
    let b = automaton_seeded(2, "")
        .render_at(2.0)
        .unwrap()
        .premultiplied
        .to_vec();
    assert_ne!(a, b, "two seeds, two soups");
    let gun = automaton("/Pattern /gun")
        .render_at(2.0)
        .unwrap()
        .premultiplied
        .to_vec();
    assert_ne!(a, gun);
}

#[test]
fn automaton_clock_replaces_the_rate() {
    let rate = automaton("/Pattern /acorn /Rate 6")
        .render_at(2.5)
        .unwrap()
        .premultiplied
        .to_vec();
    let clock = automaton("/Pattern /acorn /Clock { 6 mul }")
        .render_at(2.5)
        .unwrap()
        .premultiplied
        .to_vec();
    assert_eq!(rate, clock, "a /Clock of 6 t is a /Rate of 6");
    let held = automaton("/Pattern /acorn /Clock { pop 4 }")
        .render_at(2.5)
        .unwrap()
        .premultiplied
        .to_vec();
    assert_ne!(rate, held);
}
