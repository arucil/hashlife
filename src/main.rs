use algo::rle;
use std::fs;

fn main() {
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let mut uni = rle::read(src);

    uni.simulate(10_0000_0000_0000);
    println!("done");

    //let rle = rle::write(&uni);
    //fs::write("f1.rle", rle).unwrap();
    image::save_image(&uni, "f1.bmp");
}
