use hashlife::rle;
use hashlife::export;
use std::fs;

fn main() {
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let mut uni = rle::read(src);

    uni.simulate(1_000_000);

    //let rle = rle::write(&uni);
    //fs::write("f1.rle", rle).unwrap();
    export::save_image(&uni, "f1.bmp");
}
