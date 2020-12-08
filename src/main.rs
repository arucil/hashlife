use hashlife::universe::*;
use hashlife::rle;
use hashlife::export;
use hashlife::rule;

fn main() {
    let mut uni = Universe::new(rule::GAME_OF_LIFE);
    uni.set(0, 0, true);
    uni.set(1, 1, true);
    uni.set(2, 1, true);
    uni.set(3, 1, true);
    uni.set(-2, 1, true);
    uni.set(-3, 1, true);
    uni.set(-2, -1, true);
    uni.simulate(1);
    export::save_image(&uni, "f2.bmp");
    /*
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let mut uni = rle::read(src);

    let node = uni.simulate(node, 100_0000);
    // 75639
    println!("{}", uni.mem());
    */

    //let rle = rle::write(&uni, node);
    //fs::write("suck.rle", rle).unwrap();
    //export::save_image(&uni, node, "suck.bmp");
}
