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
            .all(|(i, p)| i % w >= w / 2 || p[3] <= 2); // a blurred halo's faintest tail (<= 2/255) may pass the clip
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
