use indexmap::IndexSet;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use crate::node::*;
use crate::rule::*;

pub struct Universe {
  set: IndexSet<Box<Node>, BuildHasherDefault<FxHasher>>,
  root: NodeId,
  empty_nodes: Vec<NodeId>,
  /// result is a 2x2 square, whose cell is arranged as follows
  /// ```ignored
  /// bit 7 ...  5  4  3  2  1  0
  ///     -     NW NE  -  - SW SE
  /// ```
  level2_results: [u8; 65536],
  level: u16,
}

impl Universe {
  pub fn new(rule: Rule) -> Self {
    let level2_results = compute_level2_results(rule);
    let mut uni = Self {
      set: IndexSet::default(),
      root: INVALID_NODE_ID,
      empty_nodes: vec![INVALID_NODE_ID; 4],
      level2_results,
      level: 0,
    };

    let root = uni.find_node(NodeKey::new_leaf(0, 0, 0, 0));
    uni.root = root;
    uni.level = 3;
    uni.empty_nodes[3] = root;
    uni
  }

  /// `num_gen` is number of generations.
  pub fn simulate(&mut self, mut num_gen: usize) {
    while num_gen != 0 {
      let k = num_gen.trailing_zeros() as u16;

      // we need to advance `2 ^ min(k, level - 3)` generations, instead of
      // `2 ^ min(k, level - 2)` generations, because the latter can cause the
      // leakage of information of the RESULT macro-cell.
      while self.level < 4.max(k + 3) {
        self.expand();
      }

      num_gen &= num_gen - 1;
      self.root = self.step(self.root, self.level, k);

      self.clear_results(self.root, self.level, k);

      // shrink() must be called after clear_results(), because the nodes
      // dropped by shrink() may be reused later, and the cached result of these
      // nodes will be invalidated.
      self.shrink();
    }
  }

  fn find_node(&mut self, key: NodeKey) -> NodeId {
    match self.set.get(&key) {
      Some(node) => {
        NodeId(&**node as *const Node as u64)
      }
      None => {
        let node = match key {
          NodeKey::Internal(key) => {
            Box::new(Node::new_internal(key))
          }
          NodeKey::Leaf(key) => {
            let results = self.compute_level3_results(key.clone());
            Box::new(Node::new_leaf(key, results))
          }
        };
        let id = NodeId(&*node as *const Node as u64);
        let new = self.set.insert(node);
        debug_assert!(new);
        id
      }
    }
  }

  fn compute_level3_results(&self, key: LeafNodeKey) -> [u16; 2] {
    let n0 = self.level2_results[key.nw as usize];
    let nn = key.nw << 2 & 0xcccc | key.ne >> 2 & 0x3333;
    let n1 = self.level2_results[nn as usize];
    let n2 = self.level2_results[key.ne as usize];
    let ww = key.nw << 8 | key.sw >> 8;
    let n3 = self.level2_results[ww as usize];
    let n4 = self.level2_results[key.center() as usize];
    let ee = key.ne << 8 | key.se >> 8;
    let n5 = self.level2_results[ee as usize];
    let n6 = self.level2_results[key.sw as usize];
    let ss = key.sw << 2 & 0xcccc | key.se >> 2 & 0x3333;
    let n7 = self.level2_results[ss as usize];
    let n8 = self.level2_results[key.se as usize];

    fn level2_square_from_quadrant(
      nw: u8, ne: u8, sw: u8, se: u8
    ) -> u16 {
      (nw as u16) << 10 | (ne as u16) << 8 | (sw as u16) << 2 | (se as u16)
    }

    let r0 = level2_square_from_quadrant(n0, n1, n3, n4);
    let r1 = level2_square_from_quadrant(n1, n2, n4, n5);
    let r2 = level2_square_from_quadrant(n3, n4, n6, n7);
    let r3 = level2_square_from_quadrant(n4, n5, n7, n8);

    let result1_nw = r0 << 5 & 0xcc00;
    let result1_ne = r1 << 3 & 0x3300;
    let result1_sw = r2 >> 3 & 0x00cc;
    let result1_se = r3 >> 5 & 0x0033;
    let result1 = result1_nw | result1_ne | result1_sw | result1_se;

    let result2_nw = self.level2_results[r0 as usize];
    let result2_ne = self.level2_results[r1 as usize];
    let result2_sw = self.level2_results[r2 as usize];
    let result2_se = self.level2_results[r3 as usize];
    let result2 = level2_square_from_quadrant(
      result2_nw, result2_ne, result2_sw, result2_se);

    [result1, result2]
  }

  pub fn set(&mut self, x: i64, y: i64, alive: bool) {
    let mut radius = 1 << self.level - 1;
    while x < -radius || x >= radius ||
      y < -radius || y >= radius
    {
      self.expand();
      radius <<= 1;
    }

    let root = self.set_rec(self.root, self.level, x, y, alive);
    self.root = root;
  }

  /// `(x, y)` are coordinate relative to center of the node.
  fn set_rec(
    &mut self,
    node: NodeId,
    level: u16,
    x: i64,
    y: i64,
    alive: bool
  ) -> NodeId {
    match node_ref(node) {
      Node::Leaf(LeafNode { key, .. }) => {
        debug_assert!(x >= -4 && x < 4 && y >= -4 && y < 4);
        let mut new_key = key.clone();
        let bits = if x < 0 {
          if y < 0 {
            &mut new_key.nw
          } else {
            &mut new_key.sw
          }
        } else {
          if y < 0 {
            &mut new_key.ne
          } else {
            &mut new_key.se
          }
        };
        let mask = 1u16 << (3 - (x & 3)) + 4 * (3 - (y & 3));
        if alive {
          *bits |= mask;
        } else {
          *bits &= !mask;
        }

        self.find_node(NodeKey::Leaf(new_key))
      }
      Node::Internal(InternalNode { key, .. }) => {
        let r = 1i64 << level - 2;
        let mut new_key = key.clone();
        if y < 0 {
          if x < 0 {
            new_key.nw = self.set_rec(key.nw, level - 1, x + r, y + r, alive);
          } else {
            new_key.ne = self.set_rec(key.ne, level - 1, x - r, y + r, alive);
          }
        } else {
          if x < 0 {
            new_key.sw = self.set_rec(key.sw, level - 1, x + r, y - r, alive);
          } else {
            new_key.se = self.set_rec(key.se, level - 1, x - r, y - r, alive);
          }
        }

        self.find_node(NodeKey::Internal(new_key))
      }
    }
  }

  fn expand(&mut self) {
    let nw;
    let ne;
    let sw;
    let se;
    match node_ref(self.root) {
      Node::Leaf(LeafNode { key, .. }) => {
        nw = self.find_node(NodeKey::Leaf(LeafNodeKey {
          se: key.nw,
          ..Default::default()
        }));
        ne = self.find_node(NodeKey::Leaf(LeafNodeKey {
          sw: key.ne,
          ..Default::default()
        }));
        sw = self.find_node(NodeKey::Leaf(LeafNodeKey {
          ne: key.sw,
          ..Default::default()
        }));
        se = self.find_node(NodeKey::Leaf(LeafNodeKey {
          nw: key.se,
          ..Default::default()
        }));
      }
      Node::Internal(InternalNode { key, .. }) => {
        let level = self.level;
        let empty = self.find_empty_node(level - 1);
        nw = self.find_node(NodeKey::Internal(InternalNodeKey {
          nw: empty,
          ne: empty,
          sw: empty,
          se: key.nw,
        }));
        ne = self.find_node(NodeKey::Internal(InternalNodeKey {
          nw: empty,
          ne: empty,
          sw: key.ne,
          se: empty,
        }));
        sw = self.find_node(NodeKey::Internal(InternalNodeKey {
          nw: empty,
          ne: key.sw,
          sw: empty,
          se: empty,
        }));
        se = self.find_node(NodeKey::Internal(InternalNodeKey {
          nw: key.se,
          ne: empty,
          sw: empty,
          se: empty,
        }));
      }
    }
    self.root = self.find_node(NodeKey::new_internal(nw, ne, sw, se));
    self.level += 1;
  }

  fn shrink(&mut self) {
    if self.empty_nodes.len() <= (self.level - 2) as usize {
      return;
    }

    while self.level > 4 {
      let root = node_ref(self.root).unwrap_internal_ref();
      let nw = node_ref(root.key.nw).unwrap_internal_ref();
      let ne = node_ref(root.key.ne).unwrap_internal_ref();
      let sw = node_ref(root.key.sw).unwrap_internal_ref();
      let se = node_ref(root.key.se).unwrap_internal_ref();
      let empty = self.empty_nodes[(self.level - 2) as usize];

      if nw.key.nw == empty && nw.key.ne == empty && nw.key.sw == empty &&
        ne.key.nw == empty && ne.key.ne == empty && ne.key.se == empty &&
        sw.key.nw == empty && sw.key.sw == empty && sw.key.se == empty &&
        se.key.ne == empty && se.key.sw == empty && se.key.se == empty
      {
        self.root = self.find_node(NodeKey::new_internal(
          nw.key.se, ne.key.sw, sw.key.ne, se.key.nw));
        self.level -= 1;
      } else {
        break;
      }
    }
  }

  fn clear_results(&self, node: NodeId, level: u16, k: u16) {
    if level >= 4 && k < level - 2 {
      let node = node_ref(node).unwrap_internal_ref();
      node.result.set(INVALID_NODE_ID);
      if level > 4 {
        self.clear_results(node.key.nw, level - 1, k);
        self.clear_results(node.key.ne, level - 1, k);
        self.clear_results(node.key.sw, level - 1, k);
        self.clear_results(node.key.se, level - 1, k);
      }
    }
  }

  fn find_empty_node(&mut self, level: u16) -> NodeId {
    let len = self.empty_nodes.len() ;
    if len < level as usize + 1 {
      for i in len..=level as usize {
        let prev = self.empty_nodes[i - 1];
        let node = self.find_node(NodeKey::new_internal(prev, prev, prev, prev));
        match node_ref(node) {
          Node::Internal(InternalNode { result, .. }) => {
            result.set(prev);
          }
          _ => unreachable!(),
        }
        self.empty_nodes.push(node);
      }
    }
    self.empty_nodes[level as usize]
  }

  pub fn mem(&self) -> usize {
    self.set.len()
  }

  // Advance `2 ^ min(k, level - 2)` generations.
  fn step(
    &mut self,
    node: NodeId,
    level: u16,
    k: u16
  ) -> NodeId {
    let node = node_ref(node).unwrap_internal_ref();
    let result = node.result.get();
    if result != INVALID_NODE_ID {
      return result;
    }

    if level == 4 {
      return self.leaf_step(node, k);
    }

    let nw = node_ref(node.key.nw).unwrap_internal_ref();
    let ne = node_ref(node.key.ne).unwrap_internal_ref();
    let sw = node_ref(node.key.sw).unwrap_internal_ref();
    let se = node_ref(node.key.se).unwrap_internal_ref();

    let n0 = self.step(node.key.nw, level - 1, k);
    let nn = self.find_node(NodeKey::new_internal(
      nw.key.ne, ne.key.nw, nw.key.se, ne.key.sw
    ));
    let n1 = self.step(nn, level - 1, k);
    let n2 = self.step(node.key.ne, level - 1, k);
    let ww = self.find_node(NodeKey::new_internal(
      nw.key.sw, nw.key.se, sw.key.nw, sw.key.ne
    ));
    let n3 = self.step(ww, level - 1, k);
    let ee = self.find_node(NodeKey::new_internal(
      ne.key.sw, ne.key.se, se.key.nw, se.key.ne
    ));
    let cc = self.find_node(NodeKey::new_internal(
      nw.key.se, ne.key.sw, sw.key.ne, se.key.nw
    ));
    let n4 = self.step(cc, level - 1, k);
    let n5 = self.step(ee, level - 1, k);
    let n6 = self.step(node.key.sw, level - 1, k);
    let ss = self.find_node(NodeKey::new_internal(
      sw.key.ne, se.key.nw, sw.key.se, se.key.sw
    ));
    let n7 = self.step(ss, level - 1, k);
    let n8 = self.step(node.key.se, level - 1, k);

    let nw;
    let ne;
    let sw;
    let se;
    if k >= level - 2 {
      let r0 = self.find_node(NodeKey::new_internal(n0, n1, n3, n4));
      let r1 = self.find_node(NodeKey::new_internal(n1, n2, n4, n5));
      let r2 = self.find_node(NodeKey::new_internal(n3, n4, n6, n7));
      let r3 = self.find_node(NodeKey::new_internal(n4, n5, n7, n8));
      nw = self.step(r0, level - 1, k);
      ne = self.step(r1, level - 1, k);
      sw = self.step(r2, level - 1, k);
      se = self.step(r3, level - 1, k);
    } else {
      let n0 = node_ref(n0).unwrap_internal_ref();
      let n1 = node_ref(n1).unwrap_internal_ref();
      let n2 = node_ref(n2).unwrap_internal_ref();
      let n3 = node_ref(n3).unwrap_internal_ref();
      let n4 = node_ref(n4).unwrap_internal_ref();
      let n5 = node_ref(n5).unwrap_internal_ref();
      let n6 = node_ref(n6).unwrap_internal_ref();
      let n7 = node_ref(n7).unwrap_internal_ref();
      let n8 = node_ref(n8).unwrap_internal_ref();
      nw = self.find_node(NodeKey::new_internal(
        n0.key.se, n1.key.sw, n3.key.ne, n4.key.sw
      ));
      ne = self.find_node(NodeKey::new_internal(
        n1.key.se, n2.key.sw, n4.key.ne, n5.key.nw
      ));
      sw = self.find_node(NodeKey::new_internal(
        n3.key.se, n4.key.sw, n6.key.ne, n7.key.nw
      ));
      se = self.find_node(NodeKey::new_internal(
        n4.key.se, n5.key.sw, n7.key.ne, n8.key.nw
      ));
    }

    let result = self.find_node(NodeKey::new_internal(nw, ne, sw, se));
    node.result.set(result);
    result
  }

  fn leaf_step(&mut self, node: &InternalNode, k: u16) -> NodeId {
    let nw = node_ref(node.key.nw).unwrap_leaf_ref();
    let ne = node_ref(node.key.ne).unwrap_leaf_ref();
    let sw = node_ref(node.key.sw).unwrap_leaf_ref();
    let se = node_ref(node.key.se).unwrap_leaf_ref();

    let quad_result_ix = (k > 0) as usize;
    let n0 = nw.results[quad_result_ix];
    let n1 = node_ref(self.find_node(NodeKey::new_leaf(
      nw.key.ne, ne.key.nw, nw.key.se, nw.key.sw)))
      .unwrap_leaf_ref()
      .results[quad_result_ix];
    let n2 = ne.results[quad_result_ix];
    let n3 = node_ref(self.find_node(NodeKey::new_leaf(
      nw.key.sw, nw.key.se, sw.key.nw, sw.key.ne)))
      .unwrap_leaf_ref()
      .results[quad_result_ix];
    let n4 = node_ref(self.find_node(NodeKey::new_leaf(
      nw.key.se, ne.key.sw, sw.key.ne, se.key.nw)))
      .unwrap_leaf_ref()
      .results[quad_result_ix];
    let n5 = node_ref(self.find_node(NodeKey::new_leaf(
      ne.key.sw, ne.key.se, se.key.nw, se.key.ne)))
      .unwrap_leaf_ref()
      .results[quad_result_ix];
    let n6 = sw.results[quad_result_ix];
    let n7 = node_ref(self.find_node(NodeKey::new_leaf(
      sw.key.ne, se.key.nw, sw.key.se, sw.key.sw)))
      .unwrap_leaf_ref()
      .results[quad_result_ix];
    let n8 = se.results[quad_result_ix];

    let nw;
    let ne;
    let sw;
    let se;
    if k >= 2 {
      nw = node_ref(self.find_node(NodeKey::new_leaf(n0, n1, n3, n4)))
        .unwrap_leaf_ref()
        .results[1];
      ne = node_ref(self.find_node(NodeKey::new_leaf(n1, n2, n4, n5)))
        .unwrap_leaf_ref()
        .results[1];
      sw = node_ref(self.find_node(NodeKey::new_leaf(n3, n4, n6, n7)))
        .unwrap_leaf_ref()
        .results[1];
      se = node_ref(self.find_node(NodeKey::new_leaf(n4, n5, n7, n8)))
        .unwrap_leaf_ref()
        .results[1];
    } else {
      nw = LeafNodeKey { nw: n0, ne: n1, sw: n3, se: n4 }.center();
      ne = LeafNodeKey { nw: n1, ne: n2, sw: n4, se: n5 }.center();
      sw = LeafNodeKey { nw: n3, ne: n4, sw: n6, se: n7 }.center();
      se = LeafNodeKey { nw: n4, ne: n5, sw: n7, se: n8 }.center();
    }

    let result = self.find_node(NodeKey::new_leaf(nw, ne, sw, se));
    node.result.set(result);
    result
  }

  fn boundary(&self) -> Boundary {
    self.boundary_rec(self.root, self.level, 0, 0)
  }

  /// Returns (left, top, right, bottom), where right and bottom are exclusive.
  fn boundary_rec(&self, node: NodeId, level: u16, ox: i64, oy: i64) -> Boundary {
    if self.empty_nodes.len() > level as usize &&
      node == self.empty_nodes[level as usize]
    {
      EMPTY_BOUNDARY
    } else {
      match node_ref(node) {
        Node::Leaf(LeafNode { key, .. }) => {
          let w = key.nw | key.sw;
          let w = (w >> 8 | w >> 4 | w | w << 4 ) & 0xf0;
          let e = key.ne | key.se;
          let e = (e >> 12 | e >> 8 | e >> 4 | e) & 0xf;
          let row = w | e;
          let (left, right) = BYTE_RANGE[row as usize];

          let n = key.nw | key.ne;
          let n = n | n >> 1 | n >> 2 | n >> 3;
          let s = key.sw | key.se;
          let s = s | s >> 1 | s >> 2 | s >> 3;
          let col = n >> 5 & 0x80 | n >> 2 & 0x40 | n << 1 & 0x20 | n << 4 & 0x10 |
            s >> 9 & 0x8 | s >> 6 & 0x4 | s >> 3 & 0x2 | s & 0x1;
          let (top, bottom) = BYTE_RANGE[col as usize];

          (left + ox, top + oy, right + ox, bottom + oy)
        }
        Node::Internal(InternalNode { key, .. }) => {
          let r = 1 << level - 2;
          let nw_bound = self.boundary_rec(key.nw, level - 1, ox - r, oy - r);
          let ne_bound = self.boundary_rec(key.ne, level - 1, ox + r, oy - r);
          let sw_bound = self.boundary_rec(key.sw, level - 1, ox - r, oy + r);
          let se_bound = self.boundary_rec(key.se, level - 1, ox + r, oy + r);
          let left = nw_bound.0.min(ne_bound.0).min(sw_bound.0).min(se_bound.0);
          let right = nw_bound.2.max(ne_bound.2).max(sw_bound.2).max(se_bound.2);
          let top = nw_bound.1.min(ne_bound.1).min(sw_bound.1).min(se_bound.1);
          let bottom = nw_bound.3.max(ne_bound.3).max(sw_bound.3).max(se_bound.3);
          (left, top, right, bottom)
        }
      }
    }
  }

  /*
  pub fn write_cells<F>(
    &self,
    Node(n): Node,
    center_x: i64,
    center_y: i64,
    boundary: Boundary,
    f: &mut F,
  )
  where
    F: FnMut(i64, i64)
  {
    if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK == 0 {
        let bits = (n >> INLINE_NODE_BIT_SHIFT) as u16;
        if bits & 1 != 0 {
          f(center_x - 1, center_y - 1)
        }
        if bits & 2 != 0 {
          f(center_x, center_y - 1)
        }
        if bits & 4 != 0 {
          f(center_x - 1, center_y)
        }
        if bits & 8 != 0 {
          f(center_x, center_y)
        }
      }
    } else {
      let n = node_value_ref(n);
      let key = &n.key;
      let level = n.level;
      let radius = 1 << (level - 1);
      if center_x + radius <= boundary.0 ||
        center_x - radius >= boundary.2 ||
        center_y + radius <= boundary.1 ||
        center_y - radius >= boundary.3
      {
        return;
      }

      let sub_radius = 1 << (level - 2);
      self.write_cells(key.nw,
        center_x - sub_radius, center_y - sub_radius, boundary, f);
      self.write_cells(key.ne,
        center_x + sub_radius, center_y - sub_radius, boundary, f);
      self.write_cells(key.sw,
        center_x - sub_radius, center_y + sub_radius, boundary, f);
      self.write_cells(key.se,
        center_x + sub_radius, center_y + sub_radius, boundary, f);
    }
  }
  */

  #[cfg(test)]
  fn debug(&self, node: NodeId, level: u16) -> Vec<u128> {
    use itertools::Itertools;

    match node_ref(node) {
      Node::Leaf(LeafNode { key, .. }) => {
        vec![
          (key.nw >> 8 & 0xf0 | key.ne >> 12 & 0xf) as u128,
          (key.nw >> 4 & 0xf0 | key.ne >>  8 & 0xf) as u128,
          (key.nw >> 0 & 0xf0 | key.ne >>  4 & 0xf) as u128,
          (key.nw << 4 & 0xf0 | key.ne >>  0 & 0xf) as u128,
          (key.sw >> 8 & 0xf0 | key.se >> 12 & 0xf) as u128,
          (key.sw >> 4 & 0xf0 | key.se >>  8 & 0xf) as u128,
          (key.sw >> 0 & 0xf0 | key.se >>  4 & 0xf) as u128,
          (key.sw << 4 & 0xf0 | key.se >>  0 & 0xf) as u128,
        ]
      }
      Node::Internal(InternalNode { key, .. }) => {
        let r = 1 << level - 1;
        let nw = self.debug(key.nw, level - 1);
        let ne = self.debug(key.ne, level - 1);
        let sw = self.debug(key.sw, level - 1);
        let se = self.debug(key.se, level - 1);
        nw.into_iter().zip(ne)
          .chain(sw.into_iter().zip(se))
          .map(|(x, y)| x << r | y)
          .collect_vec()
      }
    }
  }
}

fn node_ref(NodeId(n): NodeId) -> &'static Node {
  unsafe { std::mem::transmute(n) }
}

type Boundary = (i64, i64, i64, i64);

const EMPTY_BOUNDARY: Boundary = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);

const BYTE_RANGE: [(i64, i64); 256] = compute_byte_range();

const fn compute_byte_range() -> [(i64, i64); 256] {
  let mut result = [(0i64, 0i64); 256];
  let mut i = 1;
  result[0] = (i64::MAX, i64::MIN);
  while i < 256 {
    let low = (i as u8).leading_zeros() as i64 - 4;
    let high = 4 - (i as u8).trailing_zeros() as i64;
    result[i as usize] = (low, high);
    i += 1;
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_debug_level3() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-1, -1, true);
    uni.set(0, 0, true);
    assert_eq!(uni.debug(uni.root, uni.level),
      vec![
        0b_0000_0000,
        0b_0000_0000,
        0b_0000_0000,
        0b_0001_0000,

        0b_0000_1000,
        0b_0000_0000,
        0b_0000_0000,
        0b_0000_0000,
      ]);
  }

  #[test]
  fn test_debug_level4() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-7, -7, true);
    uni.set(0, -6, true);
    uni.set(-3, 0, true);
    uni.set(-1, 0, true);
    uni.set(-3, 1, true);
    uni.set(3, 1, true);
    uni.set(6, 3, true);
    uni.set(4, 6, true);
    assert_eq!(uni.debug(uni.root, uni.level),
      vec![
        0b_0000_0000_0000_0000,
        0b_0100_0000_0000_0000,
        0b_0000_0000_1000_0000,
        0b_0000_0000_0000_0000,

        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0000,

        0b_0000_0101_0000_0000,
        0b_0000_0100_0001_0000,
        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0010,

        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_0000,
        0b_0000_0000_0000_1000,
        0b_0000_0000_0000_0000,
      ]);
  }

  #[test]
  fn test_boundary_level3() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-3, -1, true);
    uni.set(0, -2, true);
    uni.set(-2, 0, true);
    uni.set(-1, 0, true);
    uni.set(-1, 2, true);
    assert_eq!(uni.boundary(), (-3, -2, 1, 3));
  }

  #[test]
  fn test_boundary_level4() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-3, -1, true);
    uni.set(0, -2, true);
    uni.set(-2, 0, true);
    uni.set(4, 0, true);
    uni.set(-6, 3, true);
    uni.set(2, 1, true);
    uni.set(-1, 0, true);
    uni.set(-1, 2, true);
    assert_eq!(uni.boundary(), (-6, -2, 5, 4));
  }

  #[test]
  fn test_level2_result() {
    let uni = Universe::new(GAME_OF_LIFE);
    assert_eq!(uni.level2_results[0b_0000_0110_1100_0100], 0b_11_0000);
    assert_eq!(uni.level2_results[0b_1100_0100_0000_0000], 0b_10_0000);
  }

  #[test]
  fn test_level3_result1() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    let node = uni.find_node(NodeKey::new_leaf(
      0b_0000_0000_0000_0001,
      0b_0000_0000_0000_1000,
      0b_0011_0001_0000_0000,
      0b_0000_0000_0000_0000,
    ));
    let node = node_ref(node).unwrap_leaf_ref();
    assert_eq!(node.results[0], 0b_0000_1110_1000_1100);

    let node = uni.find_node(NodeKey::new_leaf(
      0b_0000_0000_0001_0011,
      0b_0000_0000_0000_0000,
      0b_0100_0011_0000_0000,
      0b_1000_0000_0000_0000,
    ));
    let node = node_ref(node).unwrap_leaf_ref();
    assert_eq!(node.results[0], 0b_1100_1110_0010_1100);
  }

  #[test]
  fn test_level3_result2() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    let node = uni.find_node(NodeKey::new_leaf(
      0b_0000_0000_0000_0001,
      0b_0000_0000_0000_1000,
      0b_0011_0001_0000_0000,
      0b_0000_0000_0000_0000,
    ));
    let node = node_ref(node).unwrap_leaf_ref();
    assert_eq!(node.results[1], 0b_0100_1100_0010_1100);

    let node = uni.find_node(NodeKey::new_leaf(
      0b_0000_0000_0000_0000,
      0b_0000_0000_0000_0000,
      0b_0000_0000_0001_0000,
      0b_0100_1100_0010_1100,
    ));
    let node = node_ref(node).unwrap_leaf_ref();
    assert_eq!(node.results[1], 0b_0000_0000_0010_0100);
  }

  #[test]
  fn test_level4_result1() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-1, -1, true);
    uni.set(0, -1, true);
    uni.set(-2, 0, true);
    uni.set(-1, 0, true);
    uni.set(-1, 1, true);
    uni.expand();
    let node = uni.step(uni.root, 4, 0);
    assert_eq!(uni.debug(node, 3),
      vec![
        0b_0000_0000,
        0b_0000_0000,
        0b_0000_0000,
        0b_0011_1000,
        0b_0010_0000,
        0b_0011_0000,
        0b_0000_0000,
        0b_0000_0000,
      ]);
  }

  #[test]
  fn test_level4_result2() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-1, -1, true);
    uni.set(0, -1, true);
    uni.set(-2, 0, true);
    uni.set(-1, 0, true);
    uni.set(-1, 1, true);
    uni.expand();
    let node = uni.step(uni.root, 4, 1);
    assert_eq!(uni.debug(node, 3),
      vec![
        0b_0000_0000,
        0b_0000_0000,
        0b_0001_0000,
        0b_0011_0000,
        0b_0100_1000,
        0b_0011_0000,
        0b_0000_0000,
        0b_0000_0000,
      ]);
  }

  #[test]
  fn test_level4_result4() {
    let mut uni = Universe::new(GAME_OF_LIFE);
    uni.set(-1, -1, true);
    uni.set(0, -1, true);
    uni.set(-2, 0, true);
    uni.set(-1, 0, true);
    uni.set(-1, 1, true);
    uni.expand();
    let node = uni.step(uni.root, 4, 2);
    assert_eq!(uni.debug(node, 3),
      vec![
        0b_0000_0000,
        0b_0000_0000,
        0b_0010_1000,
        0b_0100_1000,
        0b_0100_1000,
        0b_0011_0000,
        0b_0000_0000,
        0b_0000_0000,
      ]);
  }

  /*
  #[test]
  fn test_simulation_2_steps() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(2);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.simulate(node, 2);
    assert_eq!(&uni.debug(node), r"
        
        
   #    
  ##    
 #  #   
  ##    
        
        ".trim_start_matches('\n'));

  }

  #[test]
  fn test_simulation_7_steps() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(2);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.simulate(node, 7);
    assert_eq!(&uni.debug(node), r"
                
                
                
                
                
       #        
     ## ##      
     #          
     #   #      
      ###       
                
                
                
                
                
                ".trim_start_matches('\n'));

  }

  #[test]
  fn test_gc() {
    use std::fs;
    use crate::rle;

    let mut uni = Universe::new();
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let node = rle::read(src, &mut uni);


    let _node = uni.simulate(node, 1);

  }
  */
}