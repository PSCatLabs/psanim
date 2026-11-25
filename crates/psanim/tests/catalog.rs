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
