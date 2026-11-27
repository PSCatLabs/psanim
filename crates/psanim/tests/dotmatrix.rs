//! The dot-matrix printer of the catalog: the head lays dots down as it passes,
//! the paper steps, the page tears, and every frame is a function of the time.

use psanim::{Movie, Options};

/// The name pkpixel's table gives the face in PSCat's `lib/fonts/{file}.ps`
/// (the first, if the file defines several), without the slash.
fn face(file: &str) -> &'static str {
    let path = format!("lib/fonts/{file}.ps");
    psanim::pixel_faces()
        .into_iter()
        .find(|(_, f)| *f == path)
        .map(|(n, _)| n)
        .unwrap_or_else(|| panic!("no face for {path} in pkpixel's table"))
}

const W: usize = 720;
const H: usize = 640;

struct Frame {
    px: Vec<u8>,
}

impl Frame {
    /// Pixels (x, y) that are red ink: the tests print in a red ribbon on plain
    /// paper so that nothing else in the picture is mistaken for it.
    fn ink(&self) -> Vec<(usize, usize)> {
        // (not the printer's red power light, down in its body)
        self.pixels(|r, g, b| r > 120 && g < 90 && b < 90)
            .into_iter()
            .filter(|q| q.1 < 560)
            .collect()
    }
    fn green(&self) -> usize {
        self.pixels(|r, g, b| g > r + 15 && g > b + 10).len()
    }
    fn pixels(&self, pred: impl Fn(u8, u8, u8) -> bool) -> Vec<(usize, usize)> {
        self.px
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, p)| pred(p[0], p[1], p[2]))
            .map(|(i, _)| (i % W, i / W))
            .collect()
    }
}

fn movie(params: &str) -> Movie {
    let src = format!(
        "<< /Size [{W} {H}] /Duration 60 /FPS 30 /Background [0 0 0] >> animation
         /x /dotmatrix << /Params << {params} >> >> instance
         /scene {{ /x << /Box [0 0 {W} {H}] /After /hold >> place }} def"
    );
    Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"))
}

fn at(m: &mut Movie, t: f64) -> Frame {
    Frame {
        px: m
            .render_at(t)
            .unwrap_or_else(|e| panic!("{e}"))
            .premultiplied
            .to_vec(),
    }
}

fn eval(params: &str, expr: &str) -> String {
    let src = format!(
        "<< /Size [100 100] /Duration 1 >> animation
         /x /dotmatrix << /Params << {params} >> >> instance
         /scene {{ {expr} }} def"
    );
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.take_output();
    m.render_at(0.0).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

fn duration(params: &str) -> f64 {
    eval(params, "/x componentduration ==").parse().unwrap()
}

const RED: &str = "/Ribbon [1 0 0] /Ink 0 /Paper /plain /Backdrop false /Top 0 /Start 0.2";

#[test]
fn the_head_lays_the_dots_down_as_it_passes_not_a_line_at_once() {
    // One line of 20 capitals at 20 characters a second: the sweep takes a second.
    let p = format!("{RED} /Text (MMMMMMMMMMMMMMMMMMMM) /Speed 20 /Tear /none");
    let mut m = movie(&p);
    assert!(
        at(&mut m, 0.1).ink().is_empty(),
        "nothing before the head moves"
    );
    let mut right = |t: f64| {
        let f = at(&mut m, t).ink();
        assert!(!f.is_empty(), "no ink at {t}");
        (
            f.iter().map(|q| q.0).min().unwrap(),
            f.iter().map(|q| q.0).max().unwrap(),
        )
    };
    let (l1, r1) = right(0.45);
    let (l2, r2) = right(0.7);
    let (l3, r3) = right(1.2);
    assert!(
        r1 + 40 < r2 && r2 + 40 < r3,
        "the ink grows rightward: {r1} {r2} {r3}"
    );
    assert!(
        l1.abs_diff(l3) < 6 && l2.abs_diff(l3) < 6,
        "from the left: {l1} {l2} {l3}"
    );
}

#[test]
fn a_bidirectional_printer_prints_the_next_line_on_the_way_back() {
    let line = "MMMMMMMMMMMMMMMMMMMM\\nMMMMMMMMMMMMMMMMMMMM";
    let mut spans = Vec::new();
    for bidir in ["true", "false"] {
        let p = format!("{RED} /Text ({line}) /Speed 20 /Tear /none /Bidirectional {bidir}");
        let d = duration(&p);
        let mut m = movie(&p);
        // Line one takes 0.2 to 1.2 s, then a feed; the middle of the second sweep:
        let mut found = None;
        for k in 0..200 {
            let t = 0.2 + f64::from(k) * 0.02;
            let ink = at(&mut m, t).ink();
            let Some(ymax) = ink.iter().map(|q| q.1).max() else {
                continue;
            };
            // the lowest line of ink, the one at the head
            let low: Vec<usize> = ink
                .iter()
                .filter(|q| q.1 + 14 > ymax)
                .map(|q| q.0)
                .collect();
            let all: Vec<usize> = ink.iter().map(|q| q.0).collect();
            let (lo, hi) = (*low.iter().min().unwrap(), *low.iter().max().unwrap());
            let full = (*all.iter().min().unwrap(), *all.iter().max().unwrap());
            // a second line begun but not finished
            if t > 1.6 && hi - lo > 150 && hi - lo + 100 < full.1 - full.0 {
                found = Some((lo, hi, full));
                break;
            }
        }
        spans.push((bidir, d, found));
    }
    let (_, d_bi, bi) = &spans[0];
    let (_, d_uni, uni) = &spans[1];
    let (lo, _, full) = bi.expect("a half-printed second line, bidirectional");
    assert!(
        lo > full.0 + 100,
        "the second line starts at the right: {lo} vs {}",
        full.0
    );
    let (lo, _, full) = uni.expect("a half-printed second line, one way");
    assert!(
        lo < full.0 + 8,
        "the second line starts at the left: {lo} vs {}",
        full.0
    );
    assert!(d_bi < d_uni, "no blank return saves time: {d_bi} {d_uni}");
}

#[test]
fn near_letter_quality_is_a_second_pass_that_fills_between_the_dots() {
    let text = "/Text (Quality jumps) /Speed 200 /Tear /none";
    let draft = at(&mut movie(&format!("{RED} {text} /Quality /Draft")), 5.0);
    let nlq = at(&mut movie(&format!("{RED} {text} /Quality /NLQ")), 5.0);
    assert!(draft.px != nlq.px, "the two qualities print differently");
    // it takes two passes of the head
    let d1 = duration(&format!("{RED} {text} /Quality /Draft"));
    let d2 = duration(&format!("{RED} {text} /Quality /NLQ"));
    assert!(d2 > d1 + 0.05, "{d2} {d1}");
}

#[test]
fn the_pinpoint_face_prints_its_own_letters_in_the_same_cells() {
    let text = "/Text (MOW 0042 gjpqy) /Speed 200 /Tear /none";
    let own = at(&mut movie(&format!("{RED} {text}")), 5.0).ink();
    let pin = at(
        &mut movie(&format!("{RED} {text} /Face /{}", face("pinpoint"))),
        5.0,
    )
    .ink();
    assert!(!own.is_empty() && !pin.is_empty());
    assert!(own != pin, "the faces print differently");
    // the same characters across: the line spans the same cells, give or take one
    let span = |v: &[(usize, usize)]| {
        let xs = v.iter().map(|p| p.0);
        xs.clone().max().unwrap() - xs.min().unwrap()
    };
    let (a, b) = (span(&own) as f64, span(&pin) as f64);
    assert!((a - b).abs() < 15.0, "{a} {b}");
}

#[test]
fn an_unknown_face_is_an_error_that_names_the_faces() {
    let src = "<< /Size [100 100] /Duration 1 >> animation
         /x /dotmatrix << /Params << /Face /Courier >> >> instance
         /scene { } def";
    let e = Movie::load(src, &Options::default())
        .err()
        .expect("a font that is not a pixel face must fail")
        .to_string();
    assert!(e.contains(&format!("/{}", face("pinpoint"))), "{e}");
}

#[test]
fn green_bar_paper_has_bars_and_plain_does_not() {
    let base = "/Ribbon [1 0 0] /Backdrop false /Text (HELLO) /Speed 200 /Tear /none";
    let g = at(&mut movie(&format!("{base} /Paper /greenbar")), 5.0).green();
    let p = at(&mut movie(&format!("{base} /Paper /plain")), 5.0).green();
    assert!(g > 5000 && p < 500, "green {g} plain {p}");
}

#[test]
fn the_page_tears_off_at_the_end_and_not_when_asked_not_to() {
    let base = format!("{RED} /Text (MMMMMMMMMM\\nMMMMMMMMMM\\nMMMMMMMMMM) /Speed 100");
    let kept = {
        let p = format!("{base} /Tear /none");
        let d = duration(&p);
        (at(&mut movie(&p), d - 0.1).ink().len(), d)
    };
    assert!(
        kept.0 > 500,
        "the printed lines stay when nothing tears: {kept:?}"
    );
    let p = format!("{base} /Tear /end");
    let d = duration(&p);
    assert!(d > kept.1 + 1.0, "the tear takes time");
    let mut m = movie(&p);
    let before = at(&mut m, d - 3.0).ink().len();
    let after = at(&mut m, d - 0.1).ink().len();
    assert!(
        before > 500,
        "the text is on the page before the tear: {before}"
    );
    assert_eq!(after, 0, "the page has gone: {after}");
}

#[test]
fn a_frame_is_a_function_of_the_time_and_the_ink_of_the_seed() {
    let p = "/Text (Quick brown fox\\njumps over) /Speed 40 /Tear /end";
    let mut m = movie(p);
    let a = at(&mut m, 2.3).px;
    let _ = at(&mut m, 9.0);
    let _ = at(&mut m, 0.5);
    assert!(
        a == at(&mut m, 2.3).px,
        "same time, same picture, in any order"
    );
    assert!(a == at(&mut movie(p), 2.3).px, "and in a fresh movie");
    // The ribbon is uneven as the seed says: another instance seed, other tones.
    let src = |seed: u32, ink: &str| {
        format!(
            "<< /Size [{W} {H}] /Duration 60 /FPS 30 >> animation
             /x /dotmatrix << /Seed {seed} /Params << /Text (Quick brown fox) /Speed 40 /Ink {ink} >> >> instance
             /scene {{ /x << /Box [0 0 {W} {H}] /After /hold >> place }} def"
        )
    };
    let frame = |seed: u32, ink: &str| {
        let mut m = Movie::load(&src(seed, ink), &Options::default()).unwrap();
        m.render_at(1.2).unwrap().premultiplied.to_vec()
    };
    assert!(
        frame(1, "0.6") != frame(2, "0.6"),
        "another seed, another ribbon"
    );
    assert!(
        frame(1, "0") == frame(2, "0"),
        "an even ribbon does not depend on the seed"
    );
}

#[test]
fn a_banner_is_printed_sideways_with_the_tops_of_the_letters_at_the_right() {
    // The T has only its top row lit in its first column: the first printed lines
    // of the page have ink at the right edge only.
    let mut m = movie(&format!(
        "{RED} /Sheet true /Text () /Banner (T) /Window [{W} {H}] /PageLines 18"
    ));
    let ink = at(&mut m, 0.5).ink();
    let band: Vec<usize> = ink.iter().map(|q| q.0).collect();
    assert!(band.len() > 500, "the page is there: {}", band.len());
    // the first lines (the top of the sheet): the right side of the printable area only
    let top = ink.iter().map(|q| q.1).min().unwrap();
    let first: Vec<usize> = ink.iter().filter(|q| q.1 < top + 40).map(|q| q.0).collect();
    assert!(
        first.iter().all(|&x| x > W / 2),
        "the top row of the letter is at the right"
    );
    // and the stem of the T, a few lines down, runs the other way across the page
    let down: Vec<usize> = ink
        .iter()
        .filter(|q| q.1 > top + 150)
        .map(|q| q.0)
        .collect();
    assert!(
        down.iter().any(|&x| x < W / 2 - 100),
        "the stem reaches across"
    );
}

#[test]
fn long_lines_wrap_at_the_papers_width() {
    // 10 columns: 25 characters make three printed lines, and a lone newline is a
    // blank line that is only fed past.
    let short = duration("/Cols 10 /Text (AAAAAAAAAA) /Top 0");
    let wrapped = duration("/Cols 10 /Text (AAAAAAAAAAAAAAAAAAAAAAAAA) /Top 0");
    let none = duration("/Cols 10 /Text (AAAAAAAAAA\\nAAAAAAAAAA\\nAAAAA) /Top 0");
    assert!((wrapped - none).abs() < 1e-9, "{wrapped} {none}");
    assert!(wrapped > short + 0.5, "{wrapped} {short}");
}
