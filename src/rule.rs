
#[derive(Debug, Clone, Copy)]
pub struct Rule {
  pub(crate) birth: NeighborMask,
  pub(crate) survival: NeighborMask,
}

pub(crate) type NeighborMask = u16;

pub const GAME_OF_LIFE: Rule = Rule {
  birth: 0b000001000,
  survival: 0b000001100,
};