use hashlife::*;

fn main() {

    let mut uni = Universe::new();
    let node = uni.new_empty_node(3);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);

    println!("{:?}", uni.boundary(node, 0, 0));
    uni.save_image(node, "fuck.jpg");
}
