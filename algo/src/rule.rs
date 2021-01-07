use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default)]
pub struct Rule {
  birth: NeighborMask,
  survival: NeighborMask,
}

pub(crate) type NeighborMask = u16;

pub const GAME_OF_LIFE: Rule = Rule {
  birth: 0b000001000,
  survival: 0b000001100,
};

pub(crate) fn compute_level2_results(rule: Rule) -> [u8; 65536] {
  let nexts = [rule.birth, rule.survival];

  let mut result = [0u8; 65536];
  for i in 0..65536usize {
    let j = i as u16;
    let nw = (nexts[i >> 10 & 1] >> (j & 0b_1110_1010_1110_0000).count_ones()) & 1;
    let ne = (nexts[i >> 9 & 1] >> (j & 0b_0111_0101_0111_0000).count_ones()) & 1;
    let sw = (nexts[i >> 6 & 1] >> (j & 0b_0000_1110_1010_1110).count_ones()) & 1;
    let se = (nexts[i >> 5 & 1] >> (j & 0b_0000_0111_0101_0111).count_ones()) & 1;
    let res = nw << 5 | ne << 4 | sw << 1 | se;
    result[i] = res as u8;
  }
  result
}

impl Rule {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn set_birth(&mut self, num: u8) {
    assert!(num < 9);
    if num == 0 {
      panic!("B0 is not allowed for HashLife");
    }
    self.birth |= 1 << num;
  }

  pub(crate) fn set_survival(&mut self, num: u8) {
    assert!(num < 9);
    self.survival |= 1 << num;
  }
}

impl Display for Rule {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "B")?;
    let mut b = self.birth;
    while b != 0 {
      write!(f, "{}", b.trailing_zeros())?;
      b &= b - 1;
    }
    write!(f, "/S")?;
    let mut s = self.survival;
    while s != 0 {
      write!(f, "{}", s.trailing_zeros())?;
      s &= s - 1;
    }
    Ok(())
  }
}