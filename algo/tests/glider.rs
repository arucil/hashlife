
#[test]
fn multi_simulate() {
  let glider_0 = "x = 3, y = 3, rule = B3/S23\nbo$2bo$3o!\n";
  let glider_1 = "x = 3, y = 3, rule = B3/S23\nobo$b2o$bo!\n";
  let glider_2 = "x = 3, y = 3, rule = B3/S23\n2bo$obo$b2o!\n";
  let glider_3 = "x = 3, y = 3, rule = B3/S23\no$b2o$2o!\n";
  let mut uni = algo::rle::read(glider_0).unwrap();

  uni.simulate(170);

  let actual = algo::rle::write(&uni);

  assert_eq!(glider_2, &actual);

  uni.simulate(1);

  let actual = algo::rle::write(&uni);

  assert_eq!(glider_3, &actual);
}