use std::fs;
use hashlife::universe::*;
use hashlife::rle;
use hashlife::export;

fn main() {
    let mut uni = Universe::new();
    let src = fs::read_to_string("Breeder.lif").unwrap();
    let node = rle::read(src, &mut uni);

    let node = uni.simulate(node, 10000);

    let rle = rle::write(&uni, node);
    fs::write("suck.rle", rle).unwrap();
    export::save_image(&uni, node, "suck.bmp");
}
