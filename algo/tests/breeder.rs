use std::fs;
use algo::rle;

#[test]
fn gen10000() {
  let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
  let mut uni = rle::read(src);
  let expected = fs::read_to_string("tests/fixtures/Breeder_gen10000.rle").unwrap();

  uni.simulate(10000);

  let actual = rle::write(&uni);

  assert_eq!(expected, actual);
}
