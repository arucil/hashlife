use std::fs;
use hashlife::universe::*;
use hashlife::rle;
use pretty_assertions::assert_eq;

#[test]
fn gen10000() {
  /*
  let mut uni = Universe::new();
  let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
  let node = rle::read(src, &mut uni);
  let expected = fs::read_to_string("tests/fixtures/Breeder_gen10000.rle").unwrap();

  let node = uni.simulate(node, 10000);

  let actual = rle::write(&uni, node);

  assert_eq!(expected, actual);
  */
}