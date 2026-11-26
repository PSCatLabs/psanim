//! Cellular automata, end to end: the words compute the rules right, the
//! famous patterns do what they are known to do, and the catalog entry is
//! a pure function of its time (the catalog's `/automaton` entry is
//! tested in `catalog.rs`).

use psanim::{Movie, Options};

const HEAD: &str = "<< /Size [100 100] /Duration 4 /FPS 10 /Background [0 0 0] >> animation
% a grid as text: 1 alive, 0 not
/bits { 3 dict begin /bs exch def /bo bs length string def
  0 1 bs length 1 sub { /bk exch def bo bk bs bk get 128 ge { 49 } { 48 } ifelse put } for bo end } def\n";

/// What `expr` prints (with `==` or `=`) in the first frame.
fn eval(setup: &str, expr: &str) -> String {
    let src = format!("{HEAD}{setup}\n/scene {{ {expr} }} def");
    let mut m = Movie::load(&src, &Options::default()).unwrap_or_else(|e| panic!("{e}"));
    m.take_output();
    m.render_at(0.0).unwrap_or_else(|e| panic!("{e}"));
    m.take_output().trim().to_string()
}

#[test]
fn a_glider_moves_one_cell_diagonally_every_four_generations() {
    let out = eval(
        "/g0 12 12 << /Pattern [ [/glider 4 4] ] >> cellgrid def
         /g1 12 12 << /Pattern [ [/glider 5 5] ] >> cellgrid def",
        "g0 12 /life 4 cellsteps bits g1 bits eq =
         g0 12 /life 3 cellsteps bits g1 bits eq =
         g0 12 /life 8 cellsteps cellcount =",
    );
    assert_eq!(out, "true\nfalse\n5");
}

#[test]
fn a_blinker_has_period_two_and_a_block_holds_still() {
    let out = eval(
        "/b 5 5 /blinker cellgrid def  /q 4 4 /block cellgrid def",
        "b 5 /life 2 cellsteps bits b bits eq =
         b 5 /life 1 cellsteps bits b bits eq =
         b 5 /life 1 cellsteps bits =
         q 4 /life 7 cellsteps bits q bits eq =",
    );
    assert_eq!(out, "true\nfalse\n0000000100001000010000000\ntrue");
}

#[test]
fn rule_30_from_one_cell_matches_the_known_rows() {
    let out = eval(
        "/h 11 1 /single cellgrid 30 5 cellrows def",
        "0 1 4 { 11 mul h exch 11 getinterval bits = } for",
    );
    assert_eq!(
        out,
        "00000100000\n00001110000\n00011001000\n00110111100\n01100100010"
    );
    // Rule 90 from one cell is Pascal's triangle mod 2.
    let out = eval(
        "/h 9 1 /single cellgrid 90 4 cellrows def",
        "0 1 3 { 9 mul h exch 9 getinterval bits = } for",
    );
    assert_eq!(out, "000010000\n000101000\n001000100\n010101010");
}

#[test]
fn the_gosper_gun_has_period_thirty_and_fires_a_glider_each_time() {
    // The gun's own box returns to its start every 30 generations, and
    // the population grows by one glider (5 cells) each period.
    let out = eval(
        "/g 100 60 << /Pattern [ [/gun 20 6] ] >> cellgrid def
         /box { /bg exch def () 0 1 8 { /r exch def bg r 2 add 100 mul 2 add 36 getinterval bits strcat } for } def
         /opts << /Rule /life /Edge /open >> def",
        "g cellcount =
         g box g 100 opts 30 cellsteps box eq =
         g box g 100 opts 90 cellsteps box eq =
         g 100 opts 120 cellsteps cellcount g 100 opts 90 cellsteps cellcount sub =",
    );
    assert_eq!(out, "36\ntrue\ntrue\n5");
}

#[test]
fn diehard_dies_at_generation_130() {
    let out = eval(
        "/d 80 60 /diehard cellgrid def",
        "d 80 /life 129 cellsteps cellcount 0 gt =  d 80 /life 130 cellsteps cellcount =",
    );
    assert_eq!(out, "true\n0");
}

#[test]
fn the_r_pentomino_settles_at_generation_1103_with_116_cells() {
    // Far enough from the edge that its six gliders are still on the grid.
    let out = eval(
        "/r 600 600 /rpentomino cellgrid def",
        "r 600 /life 1103 cellsteps cellcount =",
    );
    assert_eq!(out, "116");
}

#[test]
fn an_open_edge_lets_the_acorns_gliders_fly_off() {
    // On the open plane the acorn settles at generation 5206 with 633 cells,
    // 13 of them gliders. On a grid with an open edge the gliders leave and
    // the rest is the plane's: 633 - 13 x 5. It has settled: the next
    // generations repeat with period 2 (blinkers).
    let out = eval(
        "/a 300 380 /acorn cellgrid def /o << /Rule /life /Edge /open >> def",
        "a 300 o 5206 cellsteps cellcount =
         a 300 o 5206 cellsteps bits a 300 o 5208 cellsteps bits eq =",
    );
    assert_eq!(out, "568\ntrue");
}

#[test]
fn a_dead_edge_turns_a_glider_into_a_block() {
    let out = eval(
        "/g 20 20 << /Pattern [ [/glider 10 10] ] >> cellgrid def",
        "g 20 << /Rule /life /Edge /dead >> 200 cellsteps cellcount =
         g 20 << /Rule /life /Edge /open >> 200 cellsteps cellcount =
         g 20 << /Rule /life /Edge /wrap >> 200 cellsteps cellcount =",
    );
    assert_eq!(out, "4\n0\n5");
}

#[test]
fn brians_brain_and_other_rules_run() {
    let out = eval(
        "/s 30 30 << /Pattern /random /Seed 3 >> cellgrid def",
        "s 30 /brain 1 cellsteps bits s bits eq =
         s 30 (B36/S23) 10 cellsteps s 30 /highlife 10 cellsteps eq =
         s 30 /life 10 cellsteps s 30 /highlife 10 cellsteps eq =",
    );
    assert_eq!(out, "false\ntrue\nfalse");
}

#[test]
fn a_seed_changes_the_soup_and_the_same_seed_repeats_it() {
    let out = eval(
        "",
        "40 30 << /Pattern /random /Seed 1 >> cellgrid 40 30 << /Pattern /random /Seed 2 >> cellgrid eq =
         40 30 << /Pattern /random /Seed 1 >> cellgrid 40 30 << /Pattern /random /Seed 1 >> cellgrid eq =",
    );
    assert_eq!(out, "false\ntrue");
}

#[test]
fn rle_and_pictures_give_the_named_pattern() {
    let out = eval(
        "",
        "9 9 (bo$2bo$3o!) cellgrid 9 9 /glider cellgrid eq =
         9 9 [ (.O.) (..O) (OOO) ] cellgrid 9 9 /glider cellgrid eq =",
    );
    assert_eq!(out, "true\ntrue");
}

#[test]
fn dying_cells_fade_between_generations() {
    // A lone cell dies at once; its colour moves from /Trail toward /Ground.
    let out = eval(
        "/c 3 3 /single cellgrid 3 /life 1 cellsteps def
         /p << /Alive [1 1 1] /Trail [1 0 0] /Fade 4 /Ground [0 0 0] >> def",
        "c p 0 cellcolors 12 get =  c p 0.5 cellcolors 12 get =  c 3 /life 4 cellsteps p 0 cellcolors 12 get =",
    );
    let v: Vec<i64> = out.lines().map(|l| l.parse().unwrap()).collect();
    assert_eq!(v[0], 255, "red at the moment it dies");
    assert!(v[1] < 255 && v[1] > 0, "fading half a generation on: {v:?}");
    assert_eq!(v[2], 0, "gone after /Fade generations");
}

#[test]
fn bad_operands_say_what_is_wanted() {
    for (prog, want) in [
        ("10 10 /nosuch cellgrid", "no pattern /nosuch"),
        ("10 10 /glider cellgrid 10 /conway 1 cellsteps", "B3/S23"),
        ("10 1 /single cellgrid 300 4 cellrows", "0 to 255"),
        ("4000 4000 /random cellgrid", "1 to 2048"),
        // Never a panic: sizes are checked before anything is allocated.
        ("() 1e30 0 cellbounds", "1 to 2048"),
        ("() 5 0 cellbounds", "rows"),
        ("() 5 /life 3 cellsteps", "rows"),
        ("(abc) 1e30 /life 3 cellsteps", "1 to 2048"),
        ("1e30 1e30 /random cellgrid", "1 to 2048"),
        ("(.O.) 30 1e30 cellrows", "1 to"),
        ("(abc) 1e30 [0 0 0] cellmask", "1 to 2048"),
        ("() 4 [0 0 0] cellmask", "rows"),
    ] {
        let src = format!("{HEAD}/scene {{ {prog} pop }} def");
        let mut m = Movie::load(&src, &Options::default()).unwrap();
        let Err(e) = m.render_at(0.0) else {
            panic!("{prog}: no error")
        };
        let e = e.to_string();
        assert!(e.contains(want), "{prog}: {e}");
    }
}
