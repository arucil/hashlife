use indexmap::IndexSet;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::cell::Cell;
use itertools::Itertools;
use crate::node::*;
use crate::rule::*;

pub struct Universe {
  set: IndexSet<Box<Node>, BuildHasherDefault<FxHasher>>,
  root: NodeId,
  empty_nodes: Vec<NodeId>,
  level: u16,
}

impl Universe {
  pub fn new(rule: Rule) -> Self {
    let mut uni = Self {
      set: IndexSet::default(),
      root: INVALID_NODE_ID,
      empty_nodes: vec![INVALID_NODE_ID; 4],
      level: 0,
    };

    let root = uni.find_node(NodeKey::Leaf(LeafNodeKey {
      nw: 0,
      ne: 0,
      sw: 0,
      se: 0,
    }));
    uni.root = root;
    uni.level = 3;
    uni.empty_nodes[3] = root;
    uni
  }

  /// `num_gen` is number of generations.
  /*
  pub fn simulate(&mut self, mut root: Node, mut num_gen: usize) -> Node {
    while self.level(root) < 3 {
      root = self.expand(root);
    }

    while num_gen != 0 {
      let k = num_gen.trailing_zeros() as u16;

      let boundary = self.boundary(root, 0, 0);
      let mut subsub_radius = 1 << (self.level(root).checked_sub(3).unwrap_or(0));
      // we need to move forward `2 ^ min(k, level - 3)` steps, instead of
      // `2 ^ min(k, level - 2)` steps, because the latter can cause the leakage
      // of state of the RESULT macro-cell.
      while self.level(root) < k + 3 ||
        boundary.0 < -subsub_radius || boundary.1 < -subsub_radius ||
        boundary.2 > subsub_radius || boundary.3 > subsub_radius
      {
        root = self.expand(root);
        subsub_radius = 1 << (self.level(root) - 3);
      }

      num_gen &= num_gen - 1;
      root = self.step(root, k);
    }

    root
  }
  */

  fn find_node(&mut self, key: NodeKey) -> NodeId {
    match self.set.get(&key) {
      Some(node) => {
        NodeId(&**node as *const Node as u64)
      }
      None => {
        let node = match key {
          NodeKey::Internal(key) => {
            Box::new(Node::Internal {
              key,
              result: Cell::new(INVALID_NODE_ID),
              mark: false,
            })
          }
          NodeKey::Leaf(key) => {
            Box::new(Node::Leaf {
              key,
              results: [0, 1],
              mark: false,
            })
          }
        };
        let id = NodeId(&*node as *const Node as u64);
        let new = self.set.insert(node);
        debug_assert!(new);
        id
      }
    }
  }

  pub fn set(&mut self, x: i64, y: i64, alive: bool) {
    let mut radius = 1 << self.level - 1;
    while x < -radius || x >= radius ||
      y < -radius || y >= radius
    {
      self.expand();
      radius <<= 1;
    }

    let root = self.root;
    let level = self.level;
    let root = self.set_rec(root, level, x, y, alive);
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
      Node::Leaf { key, .. } => {
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
      Node::Internal { key, .. } => {
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
      Node::Leaf { key, .. } => {
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
      Node::Internal { key, .. } => {
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
    self.root = self.find_node(NodeKey::Internal(InternalNodeKey {
      nw, ne, sw, se,
    }));
    self.level += 1;
  }

  fn find_empty_node(&mut self, level: u16) -> NodeId {
    let len = self.empty_nodes.len() ;
    if len < level as usize + 1 {
      for i in len..=level as usize {
        let prev = self.empty_nodes[i - 1];
        let node = self.find_node(NodeKey::Internal(InternalNodeKey {
          nw: prev,
          ne: prev,
          sw: prev,
          se: prev,
        }));
        match node_ref(node) {
          Node::Internal { result, .. } => {
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

  // Move forward `2 ^ min(k, level - 2)` steps.
  /*
  fn step(
    &mut self,
    Node(n): Node,
    k: u16
  ) -> Node {
    if n & INLINE_NODE_MASK != 0 {
      self.empty_subnode(n)
    } else {
      let n = node_value_ref_mut(n);
      let level = n.level;
      let k = k.min(level - 2);
      let result = n.memo_results[k as usize];
      if result != INVALID_NODE {
        return result;
      }

      let key = &n.key;
      if level == 2 {
        self.one_step_level2(n)
      } else {
        let n0 = self.step(key.nw, k);

        let n1 = self.horizontal_center_node(key.nw, key.ne);
        let n1 = self.step(n1, k);

        let n2 = self.step(key.ne, k);

        let n3 = self.vertical_center_node(key.nw, key.sw);
        let n3 = self.step(n3, k);

        let n4 = self.center_node(key.nw, key.ne, key.sw, key.se);
        let n4 = self.step(n4, k);

        let n5 = self.vertical_center_node(key.ne, key.se);
        let n5 = self.step(n5, k);

        let n6 = self.step(key.sw, k);

        let n7 = self.horizontal_center_node(key.sw, key.se);
        let n7 = self.step(n7, k);

        let n8 = self.step(key.se, k);

        let mut nw;
        let mut ne;
        let mut sw;
        let mut se;
        if k == level - 2 {
          nw = self.new_node(level - 1, NodeKey {
            nw: n0,
            ne: n1,
            sw: n3,
            se: n4,
          });
          ne = self.new_node(level - 1, NodeKey {
            nw: n1,
            ne: n2,
            sw: n4,
            se: n5,
          });
          sw = self.new_node(level - 1, NodeKey {
            nw: n3,
            ne: n4,
            sw: n6,
            se: n7,
          });
          se = self.new_node(level - 1, NodeKey {
            nw: n4,
            ne: n5,
            sw: n7,
            se: n8,
          });

          nw = self.step(nw, k);
          ne = self.step(ne, k);
          sw = self.step(sw, k);
          se = self.step(se, k);
        } else {
          nw = self.center_node(n0, n1, n3, n4);
          ne = self.center_node(n1, n2, n4, n5);
          sw = self.center_node(n3, n4, n6, n7);
          se = self.center_node(n4, n5, n7, n8);
        }

        let result = self.new_node(level - 1, NodeKey {
          nw, ne, sw, se,
        });
        n.memo_results[k as usize] = result;
        result
      }
    }
  }
  */

  /// Returns (left, top, right, bottom), where right and bottom are exclusive.
  fn boundary_rec(&self, node: NodeId, level: u16, ox: i64, oy: i64) -> Boundary {
    if self.empty_nodes.len() > level as usize &&
      node == self.empty_nodes[level as usize]
    {
      return EMPTY_BOUNDARY
    } else {
      match node_ref(node) {
        Node::Leaf { key, .. } => {
        }
      }
      let n = node_value_ref(n);
      let key = &n.key;
      let level = n.level;
      let sub_radius = 1 << (level - 2);
      let nw_bound = self.boundary(key.nw,
        center_x - sub_radius, center_y - sub_radius);
      let ne_bound = self.boundary(key.ne,
        center_x + sub_radius, center_y - sub_radius);
      let sw_bound = self.boundary(key.sw,
        center_x - sub_radius, center_y + sub_radius);
      let se_bound = self.boundary(key.se,
        center_x + sub_radius, center_y + sub_radius);

      let x0 = nw_bound.0.min(ne_bound.0).min(sw_bound.0).min(se_bound.0);
      let x1 = nw_bound.2.max(ne_bound.2).max(sw_bound.2).max(se_bound.2);
      let y0 = nw_bound.1.min(ne_bound.1).min(sw_bound.1).min(se_bound.1);
      let y1 = nw_bound.3.max(ne_bound.3).max(sw_bound.3).max(se_bound.3);

      (x0, y0, x1, y1)
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
    match node_ref(node) {
      Node::Leaf { key, .. } => {
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
      Node::Internal { key, .. } => {
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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_debug_level3() {
    let mut uni = Universe::new(crate::rule::GAME_OF_LIFE);
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
    let mut uni = Universe::new(crate::rule::GAME_OF_LIFE);
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

  /*
  #[test]
  fn test_boundary() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(3);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    assert_eq!(uni.boundary(node, 0, 0), (-2, -1, 1, 2));
  }

  #[test]
  fn test_expand() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(2);
    let node = uni.set(node, -2, -2);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, 0);
    let node = uni.set(node, 1, 1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -2, 1);
    let node = uni.set(node, 1, -2);
    let node = uni.expand(node);
    assert_eq!(&uni.debug(node), r"
        
        
  #  #  
   ##   
   ##   
  #  #  
        
        ".trim_start_matches('\n'));

  }

  #[test]
  fn test_one_step_level2() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(2);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.step(node, 0);
    assert_eq!(node, Node(0b11 << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK));
  }

  #[test]
  fn test_one_step() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(3);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.step(node, 0);
    assert_eq!(&uni.debug(node), r"
    
### 
#   
##  ".trim_start_matches('\n'));


    let node = uni.expand(node);
    let node = uni.step(node, 0);
    assert_eq!(&uni.debug(node), r"
 #  
##  
  # 
##  ".trim_start_matches('\n'));

  }

  #[test]
  fn test_level2_result() {
    assert_eq!(LEVEL2_RESULTS[0b0010_0011_0110_0000], 0b11);
  }

  #[test]
  fn test_horizontal_center_node() {
    let mut uni = Universe::new();

    let node1 = uni.new_empty_node(2);
    let node1 = uni.set(node1, -2, -2);
    let node1 = uni.set(node1, -1, -1);
    let node1 = uni.set(node1, 0, 0);
    let node1 = uni.set(node1, 1, 1);
    let node1 = uni.set(node1, 1, -2);
    let node1 = uni.set(node1, 0, -1);
    let node1 = uni.set(node1, -1, 0);
    let node1 = uni.set(node1, -2, 1);

    let node2 = uni.new_empty_node(2);
    let node2 = uni.set(node2, -2, -2);
    let node2 = uni.set(node2, -1, -1);
    let node2 = uni.set(node2, 0, 0);
    let node2 = uni.set(node2, 1, 1);
    let node2 = uni.set(node2, 1, -2);
    let node2 = uni.set(node2, 0, -1);
    let node2 = uni.set(node2, -1, 0);
    let node2 = uni.set(node2, -2, 1);
    let node2 = uni.set(node2, -1, -2);

    let node = uni.horizontal_center_node(node1, node2);

    assert_eq!(uni.debug(node), r"
 ###
#  #
#  #
 ## ".trim_start_matches('\n'));
  }

  #[test]
  fn test_vertical_center_node() {
    let mut uni = Universe::new();

    let node1 = uni.new_empty_node(2);
    let node1 = uni.set(node1, -2, -2);
    let node1 = uni.set(node1, -1, -1);
    let node1 = uni.set(node1, 0, 0);
    let node1 = uni.set(node1, 1, 1);
    let node1 = uni.set(node1, 1, -2);
    let node1 = uni.set(node1, 0, -1);
    let node1 = uni.set(node1, -1, 0);
    let node1 = uni.set(node1, -2, 1);

    let node2 = uni.new_empty_node(2);
    let node2 = uni.set(node2, -2, -2);
    let node2 = uni.set(node2, -1, -1);
    let node2 = uni.set(node2, 0, 0);
    let node2 = uni.set(node2, 1, 1);
    let node2 = uni.set(node2, 1, -2);
    let node2 = uni.set(node2, 0, -1);
    let node2 = uni.set(node2, -1, 0);
    let node2 = uni.set(node2, -2, 1);
    let node2 = uni.set(node2, -1, -2);

    let node = uni.vertical_center_node(node1, node2);

    assert_eq!(uni.debug(node), r"
 ## 
#  #
## #
 ## ".trim_start_matches('\n'));
  }

  #[test]
  fn test_center_node() {
    let mut uni = Universe::new();

    let node1 = uni.new_empty_node(2);
    let node1 = uni.set(node1, 0, 0);
    let node1 = uni.set(node1, 1, 1);

    let node2 = uni.new_empty_node(2);
    let node2 = uni.set(node2, -2, 1);
    let node2 = uni.set(node2, -1, 0);

    let node3 = uni.new_empty_node(2);
    let node3 = uni.set(node3, 0, -1);
    let node3 = uni.set(node3, 1, -2);

    let node4 = uni.new_empty_node(2);
    let node4 = uni.set(node4, -2, -2);
    let node4 = uni.set(node4, -1, -1);

    let node = uni.center_node(node1, node2, node3, node4);

    assert_eq!(uni.debug(node), r"
#  #
 ## 
 ## 
#  #".trim_start_matches('\n'));
  }

  #[test]
  fn test_big_step_level3() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(3);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.step(node, 1);
    assert_eq!(&uni.debug(node), r"
 #  
##  
  # 
##  ".trim_start_matches('\n'));

  }

  #[test]
  fn test_big_step_level4() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(4);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    let node = uni.step(node, 2);
    assert_eq!(&uni.debug(node), r"
        
        
  # #   
 #  #   
 #  #   
  ##    
        
        ".trim_start_matches('\n'));

  }

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