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
