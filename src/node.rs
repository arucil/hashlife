use std::cell::Cell;
use std::hash::{Hash, Hasher};
use indexmap::Equivalent;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct NodeId(pub(crate) u64);

#[derive(Clone, Debug, Eq)]
pub(crate) enum Node {
  Internal {
    key: InternalNodeKey,
    result: Cell<NodeId>,
    mark: bool,
  },
  Leaf {
    key: LeafNodeKey,
  /// Results after one generation and two generations.
    results: [u16; 2],
    mark: bool,
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct InternalNodeKey {
  pub(crate) nw: NodeId,
  pub(crate) ne: NodeId,
  pub(crate) sw: NodeId,
  pub(crate) se: NodeId,
}

/// Level 3 (8x8 square) node. Each quadrant represents a 4x4 square.
///
/// # Bit-cell correspondence in a quadrant
///
/// ```ignored
/// [15] [14] [13] [12]
/// [11] [10] [09] [08]
/// [07] [06] [05] [04]
/// [03] [02] [01] [00]
/// ```
///
/// I.e. bit 15 (the highest bit of `u16`) is the top left cell, bit 12 is the
/// top right cell, etc.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub(crate) struct LeafNodeKey {
  pub(crate) nw: u16,
  pub(crate) ne: u16,
  pub(crate) sw: u16,
  pub(crate) se: u16,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum NodeKey {
  Internal(InternalNodeKey),
  Leaf(LeafNodeKey),
}

pub(crate) const INVALID_NODE_ID: NodeId = NodeId(0);

impl Hash for Node {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Node::Internal { key, .. } => {
        key.hash(state);
        0.hash(state);
      }
      Node::Leaf { key, .. } => {
        key.hash(state);
        1.hash(state);
      }
    }
  }
}

impl Hash for NodeKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      NodeKey::Internal(key) => {
        key.hash(state);
        0.hash(state);
      }
      NodeKey::Leaf(key) => {
        key.hash(state);
        1.hash(state);
      }
    }
  }
}

impl Equivalent<Box<Node>> for NodeKey {
  fn equivalent(&self, key: &Box<Node>) -> bool {
    match (self, key) {
      (NodeKey::Internal(key1), box Node::Internal { key: key2, .. }) => {
        key1 == key2
      }
      (NodeKey::Leaf(key1), box Node::Leaf { key: key2, .. }) => {
        key1 == key2
      }
      _ => false,
    }
  }
}

impl PartialEq<Node> for Node {
  fn eq(&self, other: &Node) -> bool {
    match (self, other) {
      (Node::Internal { key: key1, .. }, Node::Internal { key: key2, .. }) => {
        key1 == key2
      }
      (Node::Leaf { key: key1, .. }, Node::Leaf { key: key2, .. }) => {
        key1 == key2
      }
      _ => false,
    }
  }
}