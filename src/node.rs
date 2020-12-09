use std::cell::Cell;
use std::hash::{Hash, Hasher};
use indexmap::Equivalent;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct NodeId(pub(crate) u64);

#[derive(Clone, Debug, Eq)]
pub(crate) enum Node {
  Internal(InternalNode),
  Leaf(LeafNode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LeafNode {
  pub(crate) key: LeafNodeKey,
  /// Results after one generation and two generations.
  pub(crate) results: [u16; 2],
  pub(crate) mark: Cell<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InternalNode {
  pub(crate) key: InternalNodeKey,
  pub(crate) result: Cell<NodeId>,
  /// `2 ^ level` cells on both sides of a root square.
  pub(crate) level: u16,
  pub(crate) mark: Cell<bool>,
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
      Node::Internal(InternalNode { key, .. }) => {
        key.hash(state);
        0.hash(state);
      }
      Node::Leaf(LeafNode { key, .. }) => {
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
      (NodeKey::Internal(key1), box Node::Internal(InternalNode { key: key2, .. })) => {
        key1 == key2
      }
      (NodeKey::Leaf(key1), box Node::Leaf(LeafNode { key: key2, .. })) => {
        key1 == key2
      }
      _ => false,
    }
  }
}

impl PartialEq<Node> for Node {
  fn eq(&self, other: &Node) -> bool {
    match (self, other) {
      (Node::Internal(InternalNode { key: key1, .. }),
        Node::Internal(InternalNode { key: key2, .. }))
      => {
        key1 == key2
      }
      (Node::Leaf(LeafNode { key: key1, .. }),
        Node::Leaf(LeafNode { key: key2, .. }))
      => {
        key1 == key2
      }
      _ => false,
    }
  }
}

impl Node {
  pub(crate) fn new_leaf(key: LeafNodeKey, results: [u16; 2]) -> Node {
    Node::Leaf(LeafNode {
      key,
      results,
      mark: Cell::new(false),
    })
  }

  pub(crate) fn new_internal(key: InternalNodeKey) -> Node {
    let level = node_ref(key.nw).level() + 1;
    Node::Internal(InternalNode {
      key,
      result: Cell::new(INVALID_NODE_ID),
      level,
      mark: Cell::new(false),
    })
  }

  pub(crate) fn unwrap_leaf_ref(&self) -> &LeafNode {
    match self {
      Node::Leaf(node) => node,
      Node::Internal(_) => panic!("internal node"),
    }
  }

  pub(crate) fn unwrap_internal_ref(&self) -> &InternalNode {
    match self {
      Node::Internal(node) => node,
      Node::Leaf(_) => panic!("leaf node"),
    }
  }

  pub(crate) fn level(&self) -> u16 {
    match self {
      Node::Internal(node) => node.level,
      Node::Leaf(_) => 3,
    }
  }

  pub(crate) fn mark(&self) -> &Cell<bool> {
    match self {
      Node::Leaf(node) => &node.mark,
      Node::Internal(node) => &node.mark,
    }
  }
}

impl NodeKey {
  pub(crate) fn new_leaf(nw: u16, ne: u16, sw: u16, se: u16) -> Self {
    Self::Leaf(LeafNodeKey { nw, ne, sw, se })
  }

  pub(crate) fn new_internal(
    nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId
  ) -> Self {
    Self::Internal(InternalNodeKey { nw, ne, sw, se })
  }
}

impl LeafNodeKey {
  pub(crate) fn center(&self) -> u16 {
    self.nw << 10 & 0xcc00 | self.ne << 6 & 0x3300 |
      self.sw >> 6 & 0x00cc | self.se >> 10 & 0x0033
  }
}

pub(crate) fn node_ref(NodeId(n): NodeId) -> &'static Node {
  unsafe { std::mem::transmute(n) }
}
