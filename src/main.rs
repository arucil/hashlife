use hashlife::*;

fn main() {

    let mut uni = Universe::new();
    let mut node = uni.new_empty_node(30);
    for i in 0..81 {
        node = uni.set(node, 0, i);
    }
    node = uni.big_step(node);
    println!("{}", uni.mem());

    uni.save_image(node, "suck.bmp");
}
