use std::fs;

#[test]
fn gen10000() {
  let src = fs::read_to_string("tests/fixtures/Breeder.rle").unwrap();
  let mut uni = algo::rle::read(src).unwrap();
  let expected = fs::read_to_string("tests/fixtures/Breeder_gen10000.rle").unwrap();

  uni.simulate(10000);

  let actual = algo::rle::write(&uni);

  assert_eq!(expected, actual);
}

#[test]
fn gen515() {
  let src = fs::read_to_string("tests/fixtures/Breeder.rle").unwrap();
  let mut uni = algo::rle::read(src).unwrap();
  let expected = fs::read_to_string("tests/fixtures/Breeder_gen515.rle").unwrap();

  uni.simulate(515);

  let actual = algo::rle::write(&uni);

  assert_eq!(expected, actual);
}