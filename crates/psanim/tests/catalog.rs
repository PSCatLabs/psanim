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
            assert!(has_ink(f.premultiplied), "/{name} frame {k} is empty");
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

/// Every glyph in the entry is the right height and a rectangle, and the fonts
/// between them have every printable character.
#[test]
fn the_pixel_font_is_complete_and_regular() {
    let src = psanim::catalog::source("ledmatrix").unwrap();
    let mut fonts: std::collections::BTreeMap<usize, std::collections::BTreeSet<u8>> =
        Default::default();
    let mut height = 0;
    for line in src.lines() {
        if line.starts_with("/glyphs") {
            height = if line.starts_with("/glyphs7") { 7 } else { 5 };
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
}
