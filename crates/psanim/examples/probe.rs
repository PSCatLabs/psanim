fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut m =
        psanim::Movie::load(&src, &psanim::Options::default()).unwrap_or_else(|e| panic!("{e}"));
    match m.render_frame(0) {
        Ok(_) => {}
        Err(e) => println!("{e}"),
    }
    print!("{}", m.take_output());
}
