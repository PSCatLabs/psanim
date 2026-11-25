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
