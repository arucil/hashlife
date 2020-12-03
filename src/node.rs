use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct NodeId(u64);

pub(crate) union Node {
  pub(crate) internal: InternalNode,
  pub(crate) leaf: LeafNode,
}

#[repr(C)]
pub(crate) struct InternalNode {
  pub(crate) nw: NodeId,
  pub(crate) ne: NodeId,
  pub(crate) sw: NodeId,
  pub(crate) se: NodeId,
  pub(crate) result: Cell<NodeId>,
}

#[repr(C)]
pub(crate) struct LeafNode {
  pub(crate) _leaf: LeafTag,
  pub(crate) nw: u16,
  pub(crate) ne: u16,
  pub(crate) sw: u16,
  pub(crate) se: u16,
  /// Results after one generation and two generations.
  pub(crate) results: [u16; 2],
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum NodeKey {
  Internal {
    nw: NodeId,
    ne: NodeId,
    sw: NodeId,
    se: NodeId,
  },
  Leaf {
    nw: u16,
    ne: u16,
    sw: u16,
    se: u16,
  }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LeafTag(u64);

pub(crate) const INVALID_NODE_ID: NodeId = NodeId(0);