use std::collections::HashMap;
use std::hash::Hash;
use std::i32;
use itertools::Itertools;
use std::iter;

/// Represents a node in a quadtree.
///
/// # Internals
/// If the internal value is non-negative, it's the index of an internal node,
/// whose level is greater than or equal to 2.
/// 
/// If the internal value is negative, take the bitwise negation, and if the
/// result has the `EMPTY_NODE_MASK` bit set, it's an empty node, and the rest
/// bit pattern (i.e. excluding `EMPTY_NODE_MASK`) represents the level of the
/// node.  
/// Otherwise, the result represents the bit pattern of a node of level one, e.g.
/// bit 0 represents the `(0, 0)` cell, bit 2 the `(0, 1)` cell, etc.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Node(i64);

pub struct Universe {
  map: HashMap<NodeKey, i64>,
  vec: Vec<NodeValue>,
  counter: i64,
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

  /// `level` >= 2
  level: u16,

  /// The memoized results of `2 ^ k` steps, where `k` is the key of the `HashMap`.
  ///
  /// Maximal `k` is `level - 2`.
  results: HashMap<u16, Node>,
}

const EMPTY_NODE_MASK: i64 = 0x4000_0000_0000_0000;

impl Universe {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
      vec: vec![],
      counter: 0,
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
    if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK != 0 {
        n as u16
      } else {
        1
      }
    } else {
      self.vec[n as usize].level
    }
  }

  pub fn new_empty_node(&self, level: u16) -> Node {
    match level {
      0 => {
        panic!("level == 0")
      }
      1 => {
        Node(!0)
      }
      _ => {
        Node(!(level as i64 | EMPTY_NODE_MASK))
      }
    }
  }

  pub fn new_node(&mut self, level: u16, key: NodeKey) -> Node {
    if key.nw.0 < 0 && key.ne.0 < 0 && key.sw.0 < 0 && key.se.0 < 0 {
      if !key.nw.0 & EMPTY_NODE_MASK != 0 &&
        !key.ne.0 & EMPTY_NODE_MASK != 0 &&
        !key.sw.0 & EMPTY_NODE_MASK != 0 &&
        !key.se.0 & EMPTY_NODE_MASK != 0
      {
        let level = !key.nw.0 as u16;
        return self.new_empty_node(level + 1);
      } else if !key.nw.0 == 0 &&
        !key.ne.0 == 0 &&
        !key.sw.0 == 0 &&
        !key.se.0 == 0
      {
        return self.new_empty_node(2);
      }
    }

    let counter = self.counter;
    let new_node = *self.map.entry(key.clone()).or_insert(counter);
    if new_node == counter {
      self.counter += 1;
      self.vec.push(NodeValue {
        key,
        level,
        results: HashMap::new(),
      })
    }
    Node(new_node)
  }

  /// `(x, y)` are coordinate relative to center of the node.
  pub fn set(&mut self, Node(n): Node, x: i32, y: i32) -> Node {
    let (old_key, level) = if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK != 0 {
        let level = n as u16;
        let sub_empty = self.new_empty_node(level - 1);
        (NodeKey {
          nw: sub_empty,
          ne: sub_empty,
          sw: sub_empty,
          se: sub_empty,
        }, level)
      } else {
        assert!(x >= -1 && x < 1 && y >= -1 && y < 1);
        return Node(!(n | 1 << (x + 1 + (y + 1) * 2)));
      }
    } else {
      let n = n as usize;
      (self.vec[n].key.clone(), self.vec[n].level)
    };

    let radius = 1 << (level - 1);
    let sub_radius = radius >> 1;
    assert!(x >= -radius && x < radius && y >= -radius && y < radius);

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

    self.new_node(level, new_key)
  }

  pub fn expand(&mut self, Node(n): Node) -> Node {
    if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK != 0 {
        let level = n as u16;
        self.new_empty_node(level + 1)
      } else {
        let bits = n as u8 as i64;
        let nw = Node(!((bits & 0b0001) << 3));
        let ne = Node(!((bits & 0b0010) << 1));
        let sw = Node(!((bits & 0b0100) >> 1));
        let se = Node(!((bits & 0b1000) >> 3));
        self.new_node(2, NodeKey {
          nw, ne, sw, se,
        })
      }
    } else {
      let key = self.vec[n as usize].key.clone();
      let level = self.vec[n as usize].level;
      let empty = self.new_empty_node(level - 1);
      let nw = self.new_node(level, NodeKey {
        nw: empty,
        ne: empty,
        sw: empty,
        se: key.nw,
      });
      let ne = self.new_node(level, NodeKey {
        nw: empty,
        ne: empty,
        sw: key.ne,
        se: empty,
      });
      let sw = self.new_node(level, NodeKey {
        nw: empty,
        ne: key.sw,
        sw: empty,
        se: empty,
      });
      let se = self.new_node(level, NodeKey {
        nw: key.se,
        ne: empty,
        sw: empty,
        se: empty,
      });
      self.new_node(level + 1, NodeKey {
        nw, ne, sw, se,
      })
    }
  }

  pub fn mem(&self) -> usize {
    self.map.len()
  }

  /// Move forward `2 ^ min(k, level - 2)` steps.
  pub fn step(&mut self, Node(n): Node, k: u16) -> Node {
    if n < 0 {
      self.empty_subnode(n)
    } else {
      let value = &self.vec[n as usize];
      let level = value.level;
      let k = k.min(level - 2);
      if let Some(&result) = value.results.get(&k) {
        return result;
      }

      let key = value.key.clone();
      if level == 2 {
        self.one_step_level2(n, key)
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
        self.vec[n as usize].results.insert(k, result);
        result
      }
    }
  }

  fn horizontal_center_node(&mut self, Node(n1): Node, Node(n2): Node) -> Node {
    if n1 < 0 && n2 < 0 {
      return Node(n1)
    } else {
      let level = if n1 < 0 {
        self.vec[n2 as usize].level
      } else {
        self.vec[n1 as usize].level
      };

      let nw = self.quadrant(Node(n1), |key| key.ne);
      let sw = self.quadrant(Node(n1), |key| key.se);
      let ne = self.quadrant(Node(n2), |key| key.nw);
      let se = self.quadrant(Node(n2), |key| key.sw);

      return self.new_node(level, NodeKey {
        nw, ne, sw, se,
      });
    }
  }

  fn vertical_center_node(&mut self, Node(n1): Node, Node(n2): Node) -> Node {
    if n1 < 0 && n2 < 0 {
      return Node(n1)
    } else {
      let level = if n1 < 0 {
        self.vec[n2 as usize].level
      } else {
        self.vec[n1 as usize].level
      };

      let nw = self.quadrant(Node(n1), |key| key.sw);
      let ne = self.quadrant(Node(n1), |key| key.se);
      let sw = self.quadrant(Node(n2), |key| key.nw);
      let se = self.quadrant(Node(n2), |key| key.ne);

      return self.new_node(level, NodeKey {
        nw, ne, sw, se,
      });
    }
  }

  fn center_node(
    &mut self,
    Node(n1): Node,
    Node(n2): Node,
    Node(n3): Node,
    Node(n4): Node,
  ) -> Node {
    if n1 < 0 && n2 < 0 && n3 < 0 && n4 < 0 {
      if !n1 & EMPTY_NODE_MASK != 0 {
        assert!(!n2 & EMPTY_NODE_MASK != 0);
        assert!(!n3 & EMPTY_NODE_MASK != 0);
        assert!(!n4 & EMPTY_NODE_MASK != 0);
        return Node(n1);
      } else {
        let n1 = !n1 as u8;
        let n2 = !n2 as u8;
        let n3 = !n3 as u8;
        let n4 = !n4 as u8;
        let bits = n1 >> 3 & 1 | n2 >> 1 & 2 | n3 << 1 & 4 | n4 << 3 & 8;
        return Node(!(bits as i64));
      }
    } else {
      let level = if n1 >= 0 {
        self.vec[n1 as usize].level
      } else if n2 >= 0 {
        self.vec[n2 as usize].level
      } else if n3 >= 0 {
        self.vec[n3 as usize].level
      } else {
        self.vec[n4 as usize].level
      };

      let nw = self.quadrant(Node(n1), |key| key.se);
      let ne = self.quadrant(Node(n2), |key| key.sw);
      let sw = self.quadrant(Node(n3), |key| key.ne);
      let se = self.quadrant(Node(n4), |key| key.nw);

      return self.new_node(level, NodeKey {
        nw, ne, sw, se,
      });
    }
  }

  fn quadrant<F>(
    &self,
    Node(n): Node,
    f: F,
  ) -> Node
  where
    F: FnOnce(&NodeKey) -> Node
  {
    if n < 0 {
      self.empty_subnode(n)
    } else {
      f(&self.vec[n as usize].key)
    }
  }

  fn empty_subnode(&self, n: i64) -> Node {
    let n = !n;
    if n & EMPTY_NODE_MASK != 0 {
      let level = n as u16;
      self.new_empty_node(level - 1)
    } else {
      unreachable!()
    }
  }

  fn one_step_level2(&mut self, n: i64, key: NodeKey) -> Node {
    let nw = !key.nw.0 as u64;
    let ne = !key.ne.0 as u64;
    let sw = !key.sw.0 as u64;
    let se = !key.se.0 as u64;
    assert!(nw < 16);
    assert!(ne < 16);
    assert!(sw < 16);
    assert!(se < 16);

    let lv2_bits = (nw & 0b11) | (ne & 0b11) << 2;
    let lv2_bits = lv2_bits | (nw & 0b1100) << 2 | (ne & 0b1100) << 4;
    let lv2_bits = lv2_bits | (sw & 0b11) << 8 | (se & 0b11) << 10;
    let lv2_bits = lv2_bits | (sw & 0b1100) << 10 | (se & 0b1100) << 12;
    let bits = LEVEL2_RESULTS[lv2_bits as usize] as i64;

    let result = Node(!bits);
    self.vec[n as usize].results.insert(0, result);
    result
  }

  /// Returns (left, top, right, bottom), right and bottom are exclusive.
  pub fn boundary(&self, Node(n): Node, center_x: i32, center_y: i32) -> Boundary {
    if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK != 0 {
        EMPTY_BOUNDARY
      } else {
        let b@(x0, y0, x1, y1) = LEVEL1_BOUNDARIES[n as u16 as usize];
        if b == EMPTY_BOUNDARY {
          b
        } else {
          (center_x + x0, center_y + y0, center_x + x1, center_y + y1)
        }
      }
    } else {
      let key = &self.vec[n as usize].key;
      let level = self.vec[n as usize].level;
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
    center_x: i32,
    center_y: i32,
    boundary: Boundary,
    f: &mut F,
  )
  where
    F: FnMut(i32, i32)
  {
    if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK == 0 {
        let bits = n as u16;
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
      let key = &self.vec[n as usize].key;
      let level = self.vec[n as usize].level;
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
    if n < 0 {
      let n = !n;
      if n & EMPTY_NODE_MASK != 0 {
        let n = n as u16;
        let row = iter::repeat(' ').take(1 << n).collect::<String>();
        iter::repeat(row).take(1 << n).join("\n")
      } else {
        let bits = n as u8;
        format!("{}{}\n{}{}",
          if bits & 1 != 0 { '#' } else { ' ' },
          if bits & 2 != 0 { '#' } else { ' ' },
          if bits & 4 != 0 { '#' } else { ' ' },
          if bits & 8 != 0 { '#' } else { ' ' },
        )
      }
    } else {
      let key = &self.vec[n as usize].key;

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

type Boundary = (i32, i32, i32, i32);

static LEVEL2_RESULTS: [u8; 65536] = compute_level2_results();

const EMPTY_BOUNDARY: Boundary = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);

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
      (i32::MAX, i32::MIN)
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
      (i32::MAX, i32::MIN)
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
    assert_eq!(node, Node(!0b11));
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
