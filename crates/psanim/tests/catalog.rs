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
