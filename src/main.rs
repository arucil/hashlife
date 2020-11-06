use std::fs;
use hashlife::universe::*;
use hashlife::rle;

fn main() {
    let mut uni = Universe::new();
    let src = fs::read_to_string("Breeder.lif").unwrap();
    let node = rle::read(src, &mut uni);

    let node = uni.simulate(node, 1000);

    uni.save_image(node, "suck.bmp");
}
