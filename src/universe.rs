use indexmap::IndexMap;
use std::hash::Hash;
use itertools::Itertools;
use std::iter;

/// Represents a node in a quadtree.
///
/// # Internals
///
/// If the lowest two bits of the internal value is zero, it's a pointer to an
/// internal node, whose level is greater than or equal to 2.
/// 
/// If the lowest two bits are set, it's an empty node, and the rest bits
/// represents the level of the node.  
/// If only the lowest bit is set, it's an node of level one, i.e. an 2x2 node,
/// and the rest bits represents the cells of the node, e.g., bit 2 represents
/// the `(0, 0)` cell, bit 4 the `(0, 1)` cell, etc.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Node(u64);

pub struct Universe {
  map: IndexMap<NodeKey, Box<NodeValue>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NodeKey {
  nw: Node,
  ne: Node,
  sw: Node,
  se: Node,
}

#[derive(Clone)]
struct NodeValue {
  key: NodeKey,

  /// index of `Universe::map`.
  index: usize,
  refcount: u64,

  /// `level` >= 2
  level: u16,

  /// The memoized results of `2 ^ k` steps, where `k` is the index of the `Vec`.
  ///
  /// Maximal `k` is `level - 2`.
  results: Box<[Node]>,
}

const EMPTY_NODE_MASK: u64 = 0x2;
const INLINE_NODE_MASK: u64 = 0x1;
const INLINE_NODE_BIT_SHIFT: usize = 2;
const INVALID_NODE: Node = Node(0);

impl Universe {
  pub fn new() -> Self {
    Self {
      map: IndexMap::default(),
    }
  }

  /// `num_gen` is number of generations.
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

  fn level(&self, Node(n): Node) -> u16 {
    if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK != 0 {
        (n >> INLINE_NODE_BIT_SHIFT) as u16
      } else {
        1
      }
    } else {
      node_value_ref(n).level
    }
  }

  pub fn new_empty_node(&self, level: u16) -> Node {
    match level {
      0 => {
        panic!("level == 0")
      }
      1 => {
        Node(INLINE_NODE_MASK)
      }
      _ => {
        Node((level as u64) << INLINE_NODE_BIT_SHIFT | EMPTY_NODE_MASK | INLINE_NODE_MASK)
      }
    }
  }

  fn new_node1(&mut self, level: u16, key: NodeKey) -> Node {
    let n = self.new_node_raw(level, key);
    self.inc_tree_rc(n);
    n
  }

  /// Doesn't increment the refcount.
  fn new_node_raw(&mut self, level: u16, key: NodeKey) -> Node {
    if key.nw.0 & EMPTY_NODE_MASK != 0 &&
      key.ne.0 & EMPTY_NODE_MASK != 0 &&
      key.sw.0 & EMPTY_NODE_MASK != 0 &&
      key.se.0 & EMPTY_NODE_MASK != 0
    {
      let level = (key.nw.0 >> INLINE_NODE_BIT_SHIFT) as u16;
      return self.new_empty_node(level + 1);
    }

    if key.nw.0 == INLINE_NODE_MASK &&
      key.ne.0 == INLINE_NODE_MASK &&
      key.sw.0 == INLINE_NODE_MASK &&
      key.se.0 == INLINE_NODE_MASK
    {
      return self.new_empty_node(2);
    }

    let len = self.map.len();
    let new_node = self.map.entry(key.clone()).or_insert_with(|| {
      Box::new(NodeValue {
        key,
        level,
        index: len,
        refcount: 0,
        results: vec![INVALID_NODE; (level - 1) as usize].into_boxed_slice(),
      })
    });

    let n = new_node.as_ref() as *const NodeValue as u64;
    debug_assert!(n & 3 == 0);

    Node(n)
  }

  /// `(x, y)` are coordinate relative to center of the node.
  pub fn set(&mut self, node@Node(n): Node, x: i64, y: i64) -> Node {
    let (old_key, level) = if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK != 0 {
        let level = (n >> INLINE_NODE_BIT_SHIFT) as u16;
        let sub_empty = self.new_empty_node(level - 1);
        (NodeKey {
          nw: sub_empty,
          ne: sub_empty,
          sw: sub_empty,
          se: sub_empty,
        }, level)
      } else {
        debug_assert!(x >= -1 && x < 1 && y >= -1 && y < 1);
        let shift = (x + 1 + (y + 1) * 2) as usize + INLINE_NODE_BIT_SHIFT;
        return Node(n | 1 << shift);
      }
    } else {
      let n = node_value_ref(n);
      let key = n.key.clone();
      let level = n.level;
      self.dec_node_rc(node);
      (key, level)
    };

    let radius = 1 << (level - 1);
    let sub_radius = radius >> 1;
    debug_assert!(x >= -radius && x < radius && y >= -radius && y < radius);

    let new_key = if y < 0 {
      if x < 0 {
        NodeKey {
          nw: self.set(old_key.nw, x + sub_radius, y + sub_radius),
          ..old_key
        }
      } else {
        NodeKey {
          ne: self.set(old_key.ne, x - sub_radius, y + sub_radius),
          ..old_key
        }
      }
    } else {
      if x < 0 {
        NodeKey {
          sw: self.set(old_key.sw, x + sub_radius, y - sub_radius),
          ..old_key
        }
      } else {
        NodeKey {
          se: self.set(old_key.se, x - sub_radius, y - sub_radius),
          ..old_key
        }
      }
    };

    let new_node = self.new_node_raw(level, new_key);
    self.inc_node_rc(new_node);
    new_node
  }

  pub fn expand(&mut self, node@Node(n): Node) -> Node {
    if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK != 0 {
        let level = (n >> INLINE_NODE_BIT_SHIFT) as u16;
        self.new_empty_node(level + 1)
      } else {
        let bits = (n >> INLINE_NODE_BIT_SHIFT) as u8 as u64;
        let nw = Node(((bits & 0b0001) << 3) << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
        let ne = Node(((bits & 0b0010) << 1) << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
        let sw = Node(((bits & 0b0100) >> 1) << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
        let se = Node(((bits & 0b1000) >> 3) << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
        let new_node = self.new_node_raw(2, NodeKey {
          nw, ne, sw, se,
        });
        // The children are leaves, no need to call inc_tree_rc().
        self.inc_node_rc(new_node);
        new_node
      }
    } else {
      let n = node_value_ref(n);
      let key = &n.key;
      let level = n.level;
      let empty = self.new_empty_node(level - 1);

      let nw = self.new_node_raw(level, NodeKey {
        nw: empty,
        ne: empty,
        sw: empty,
        se: key.nw,
      });
      self.inc_node_rc(nw);
      let ne = self.new_node_raw(level, NodeKey {
        nw: empty,
        ne: empty,
        sw: key.ne,
        se: empty,
      });
      self.inc_node_rc(ne);
      let sw = self.new_node_raw(level, NodeKey {
        nw: empty,
        ne: key.sw,
        sw: empty,
        se: empty,
      });
      self.inc_node_rc(sw);
      let se = self.new_node_raw(level, NodeKey {
        nw: key.se,
        ne: empty,
        sw: empty,
        se: empty,
      });
      self.inc_node_rc(se);

      self.dec_node_rc(node);

      let new_node = self.new_node_raw(level + 1, NodeKey {
        nw, ne, sw, se,
      });
      self.inc_node_rc(new_node);
      new_node
    }
  }

  pub fn mem(&self) -> usize {
    self.map.len()
  }

  fn inc_tree_rc(&self, Node(n): Node) {
    if n & INLINE_NODE_MASK != 0 {
      return;
    }

    let mut stk = vec![n];
    while let Some(n) = stk.pop() {
      let n = node_value_ref_mut(n);
      n.refcount += 1;

      if n.key.nw.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.nw.0);
      }
      if n.key.ne.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.ne.0);
      }
      if n.key.sw.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.sw.0);
      }
      if n.key.se.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.se.0);
      }
    }
  }

  fn inc_node_rc(&self, Node(n): Node) {
    if n & INLINE_NODE_MASK != 0 {
      return;
    }

    let n = node_value_ref_mut(n);
    n.refcount += 1;
  }

  fn dec_tree_rc(&mut self, Node(n): Node) {
    if n & INLINE_NODE_MASK != 0 {
      return;
    }

    let mut stk = vec![n];

    while let Some(n) = stk.pop() {
      let n = node_value_ref_mut(n);
      // Doesn't reclaim nodes of level 2.
      if n.level == 2 {
        continue;
      }
      debug_assert!(n.refcount != 0);
      n.refcount -= 1;

      if n.key.nw.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.nw.0);
      }
      if n.key.ne.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.ne.0);
      }
      if n.key.sw.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.sw.0);
      }
      if n.key.se.0 & INLINE_NODE_MASK == 0 {
        stk.push(n.key.se.0);
      }

      if n.refcount == 0 {
        self.remove_node(n);
      }
    }
  }

  /// Doesn't reclaim entire tree.
  fn dec_node_rc(&mut self, Node(n): Node) {
    if n & INLINE_NODE_MASK != 0 {
      return;
    }

    let n = node_value_ref_mut(n);
    // Doesn't reclaim node of level 2.
    if n.level == 2 {
      return;
    }
    debug_assert!(n.refcount != 0);
    n.refcount -= 1;

    if n.refcount == 0 {
      self.remove_node(n);
    }
  }

  /// Doesn't remove entire tree.
  fn remove_node(&mut self, n: &NodeValue) {
    let last = self.map.len() - 1;
    self.map[last].index = n.index;
    self.map.swap_remove_index(n.index);
  }

  /// Move forward `2 ^ min(k, level - 2)` steps.
  pub fn step(&mut self, Node(n): Node, k: u16) -> Node {
    if n & INLINE_NODE_MASK != 0 {
      self.empty_subnode(n)
    } else {
      let n = node_value_ref_mut(n);
      let level = n.level;
      let k = k.min(level - 2);
      let result = n.results[k as usize];
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

        let result = self.new_node_raw(level - 1, NodeKey {
          nw, ne, sw, se,
        });
        self.inc_node_rc(result);
        n.results[k as usize] = result;
        result
      }
    }
  }

  fn horizontal_center_node(&mut self, Node(n1): Node, Node(n2): Node) -> Node {
    if n1 & INLINE_NODE_MASK != 0 && n2 & INLINE_NODE_MASK != 0 {
      Node(n1)
    } else {
      let level = if n1 & INLINE_NODE_MASK != 0 {
        node_value_ref(n2).level
      } else {
        node_value_ref(n1).level
      };

      let nw = self.quadrant(Node(n1), |key| key.ne);
      let sw = self.quadrant(Node(n1), |key| key.se);
      let ne = self.quadrant(Node(n2), |key| key.nw);
      let se = self.quadrant(Node(n2), |key| key.sw);

      let new_node = self.new_node_raw(level, NodeKey {
        nw, ne, sw, se,
      });
      self.inc_node_rc(new_node);
      new_node
    }
  }

  fn vertical_center_node(&mut self, Node(n1): Node, Node(n2): Node) -> Node {
    if n1 & INLINE_NODE_MASK != 0 && n2 & INLINE_NODE_MASK != 0 {
      Node(n1)
    } else {
      let level = if n1 & INLINE_NODE_MASK != 0 {
        node_value_ref(n2).level
      } else {
        node_value_ref(n1).level
      };

      let nw = self.quadrant(Node(n1), |key| key.sw);
      let ne = self.quadrant(Node(n1), |key| key.se);
      let sw = self.quadrant(Node(n2), |key| key.nw);
      let se = self.quadrant(Node(n2), |key| key.ne);

      let new_node = self.new_node_raw(level, NodeKey {
        nw, ne, sw, se,
      });
      self.inc_node_rc(new_node);
      new_node
    }
  }

  fn center_node(
    &mut self,
    Node(n1): Node,
    Node(n2): Node,
    Node(n3): Node,
    Node(n4): Node,
  ) -> Node {
    if n1 & INLINE_NODE_MASK != 0 &&
      n2 & INLINE_NODE_MASK != 0 &&
      n3 & INLINE_NODE_MASK != 0 &&
      n4 & INLINE_NODE_MASK != 0
    {
      if n1 & EMPTY_NODE_MASK != 0 {
        debug_assert!(n2 & EMPTY_NODE_MASK != 0);
        debug_assert!(n3 & EMPTY_NODE_MASK != 0);
        debug_assert!(n4 & EMPTY_NODE_MASK != 0);
        return Node(n1);
      } else {
        let n1 = (n1 >> INLINE_NODE_BIT_SHIFT) as u8;
        let n2 = (n2 >> INLINE_NODE_BIT_SHIFT) as u8;
        let n3 = (n3 >> INLINE_NODE_BIT_SHIFT) as u8;
        let n4 = (n4 >> INLINE_NODE_BIT_SHIFT) as u8;
        let bits = n1 >> 3 & 1 | n2 >> 1 & 2 | n3 << 1 & 4 | n4 << 3 & 8;
        return Node((bits as u64) << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
      }
    } else {
      let level = if n1 & INLINE_NODE_MASK == 0 {
        node_value_ref(n1).level
      } else if n2 & INLINE_NODE_MASK == 0 {
        node_value_ref(n2).level
      } else if n3 & INLINE_NODE_MASK == 0 {
        node_value_ref(n3).level
      } else {
        node_value_ref(n4).level
      };

      let nw = self.quadrant(Node(n1), |key| key.se);
      let ne = self.quadrant(Node(n2), |key| key.sw);
      let sw = self.quadrant(Node(n3), |key| key.ne);
      let se = self.quadrant(Node(n4), |key| key.nw);

      let new_node = self.new_node_raw(level, NodeKey {
        nw, ne, sw, se,
      });
      self.inc_node_rc(new_node);
      new_node
    }
  }

  #[inline]
  fn quadrant<F>(
    &self,
    Node(n): Node,
    f: F,
  ) -> Node
  where
    F: FnOnce(&NodeKey) -> Node
  {
    if n & INLINE_NODE_MASK != 0 {
      self.empty_subnode(n)
    } else {
      f(&node_value_ref(n).key)
    }
  }

  fn empty_subnode(&self, n: u64) -> Node {
    if n & EMPTY_NODE_MASK != 0 {
      let level = (n >> INLINE_NODE_BIT_SHIFT) as u16;
      self.new_empty_node(level - 1)
    } else {
      unreachable!()
    }
  }

  fn one_step_level2(&mut self, n: &mut NodeValue) -> Node {
    let nw = n.key.nw.0 as u64;
    let ne = n.key.ne.0 as u64;
    let sw = n.key.sw.0 as u64;
    let se = n.key.se.0 as u64;
    debug_assert!(nw & INLINE_NODE_MASK != 0);
    debug_assert!(ne & INLINE_NODE_MASK != 0);
    debug_assert!(sw & INLINE_NODE_MASK != 0);
    debug_assert!(se & INLINE_NODE_MASK != 0);
    let nw = nw >> INLINE_NODE_BIT_SHIFT;
    let ne = ne >> INLINE_NODE_BIT_SHIFT;
    let sw = sw >> INLINE_NODE_BIT_SHIFT;
    let se = se >> INLINE_NODE_BIT_SHIFT;

    let lv2_bits = (nw & 0b11) | (ne & 0b11) << 2;
    let lv2_bits = lv2_bits | (nw & 0b1100) << 2 | (ne & 0b1100) << 4;
    let lv2_bits = lv2_bits | (sw & 0b11) << 8 | (se & 0b11) << 10;
    let lv2_bits = lv2_bits | (sw & 0b1100) << 10 | (se & 0b1100) << 12;
    let bits = LEVEL2_RESULTS[lv2_bits as usize] as u64;

    let result = Node(bits << INLINE_NODE_BIT_SHIFT | INLINE_NODE_MASK);
    n.results[0] = result;
    result
  }

  /// Returns (left, top, right, bottom), right and bottom are exclusive.
  pub fn boundary(&self, Node(n): Node, center_x: i64, center_y: i64) -> Boundary {
    if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK != 0 {
        EMPTY_BOUNDARY
      } else {
        let n = (n >> INLINE_NODE_BIT_SHIFT) as u16;
        let b@(x0, y0, x1, y1) = LEVEL1_BOUNDARIES[n as usize];
        if b == EMPTY_BOUNDARY {
          b
        } else {
          (center_x + x0, center_y + y0, center_x + x1, center_y + y1)
        }
      }
    } else {
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

  pub fn debug(&self, Node(n): Node) -> String {
    if n & INLINE_NODE_MASK != 0 {
      if n & EMPTY_NODE_MASK != 0 {
        let n = (n >> INLINE_NODE_BIT_SHIFT) as u16;
        let row = iter::repeat(' ').take(1 << n).collect::<String>();
        iter::repeat(row).take(1 << n).join("\n")
      } else {
        let bits = (n >> INLINE_NODE_BIT_SHIFT) as u8;
        format!("{}{}\n{}{}",
          if bits & 1 != 0 { '#' } else { ' ' },
          if bits & 2 != 0 { '#' } else { ' ' },
          if bits & 4 != 0 { '#' } else { ' ' },
          if bits & 8 != 0 { '#' } else { ' ' },
        )
      }
    } else {
      let key = &node_value_ref(n).key;

      let nw = self.debug(key.nw);
      let ne = self.debug(key.ne);
      let sw = self.debug(key.sw);
      let se = self.debug(key.se);

      let mut lines: Vec<_> = nw.lines().zip(ne.lines())
        .map(|(a,b)| format!("{}{}", a, b))
        .collect();
      lines.extend(sw.lines().zip(se.lines())
        .map(|(a,b)| format!("{}{}", a, b)));

      lines.join("\n")
    }
  }
}

fn node_value_ref(n: u64) -> &'static NodeValue {
  unsafe { std::mem::transmute(n) }
}

fn node_value_ref_mut(n: u64) -> &'static mut NodeValue {
  unsafe { std::mem::transmute(n) }
}

type Boundary = (i64, i64, i64, i64);

static LEVEL2_RESULTS: [u8; 65536] = compute_level2_results();

const EMPTY_BOUNDARY: Boundary = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);

static LEVEL1_BOUNDARIES: [Boundary; 16] = compute_level1_boundaries();

const fn compute_level1_boundaries() -> [Boundary; 16] {
  let mut output = [EMPTY_BOUNDARY; 16];

  let mut i = 1;
  while i < 16 {
    let (x0, x1) = if i & 0b0101 != 0 {
      if i & 0b1010 != 0 {
        (-1, 1)
      } else {
        (-1, 0)
      }
    } else if i & 0b1010 != 0 {
      (0, 1)
    } else {
      (i64::MAX, i64::MIN)
    };

    let (y0, y1) = if i & 0b0011 != 0 {
      if i & 0b1100 != 0 {
        (-1, 1)
      } else {
        (-1, 0)
      }
    } else if i & 0b1100 != 0 {
      (0, 1)
    } else {
      (i64::MAX, i64::MIN)
    };

    output[i] = (x0, y0, x1, y1);
    i += 1;
  }

  output
}

const fn compute_level2_results() -> [u8; 65536] {
  let mut output = [0u8; 65536];

  let mut i = 1usize;
  while i < 65536 {

    const fn new_bit(i: usize, mask: usize, offset: usize) -> u8 {
      let num = (i & mask).count_ones();
      if num == 3 {
        1
      } else if num == 2 {
        ((i >> offset) & 1) as u8
      } else {
        0
      }
    }

    let b0 = new_bit(i, 0b0111_0101_0111, 5);
    let b1 = new_bit(i, 0b1110_1010_1110, 6);
    let b2 = new_bit(i, 0b0111_0101_0111_0000, 9);
    let b3 = new_bit(i, 0b1110_1010_1110_0000, 10);
    output[i] = b0 | b1 << 1 | b2 << 2 | b3 << 3;
    i += 1;
  }

  output
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_debug() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(2);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, 0);
    assert_eq!(&uni.debug(node), r"
    
 #  
  # 
    ".trim_start_matches('\n'));
  }

  #[test]
  fn test_debug_level3() {
    let mut uni = Universe::new();
    let node = uni.new_empty_node(3);
    let node = uni.set(node, -1, -1);
    let node = uni.set(node, 0, -1);
    let node = uni.set(node, -2, 0);
    let node = uni.set(node, -1, 0);
    let node = uni.set(node, -1, 1);
    assert_eq!(&uni.debug(node), r"
        
        
        
   ##   
  ##    
   #    
        
        ".trim_start_matches('\n'));
  }

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
}
