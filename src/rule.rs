
#[derive(Debug, Clone, Copy)]
pub struct Rule {
  pub(crate) birth: NumNeighborMask,
  pub(crate) survival: NumNeighborMask,
}

pub(crate) type NumNeighborMask = u16;

pub const GAME_OF_LIFE: Rule = Rule {
  birth: 0b000001000,
  survival: 0b000001100,
};