use regex::Regex;
use crate::universe::*;

/// Read a Life pattern from a RLE string.
///
/// RLE format: <https://www.conwaylife.com/wiki/Run_Length_Encoded>.
pub fn read(
  src: impl AsRef<str>,
  univ: &mut Universe,
) {
  /*
  let header_re = Regex::new(r"^x = (\d+), y = (\d+)\b").unwrap();
  let mut src = src.as_ref();

  let width: u32;
  let height: u32;
  if let Some(caps) = header_re.captures(src) {
    width = caps.get(1).unwrap().as_str().parse().unwrap();
    height = caps.get(2).unwrap().as_str().parse().unwrap();
    assert!(width > 0 && height > 0);
  } else {
    panic!("invalid header line");
  }

  src = &src[src.find('\n').unwrap_or(src.len())..];

  let level = width.max(height);
  let level = 32 + (level & level - 1 != 0) as u32 - level.leading_zeros();
  let mut node = univ.new_empty_node(level as u16);

  let mut x = 0;
  let mut y = 0;
  loop {
    src = src.trim_start();

    if src.is_empty() {
      panic!("unexpected EOF");
    }

    let b0 = src.as_bytes()[0];
    if b0 == b'!' {
      break;
    }

    let mut num = 1;
    if b0 >= b'0' && b0 <= b'9' {
      let num_len = src.find(|c: char| !c.is_ascii_digit()).unwrap_or(src.len());
      num = src[..num_len].parse().unwrap();
      src = &src[num_len..];
    }

    match src.as_bytes()[0] {
      b'b' => {
        x += num;
      }
      b'o' => {
        for i in 0..num {
          node = univ.set(node, x + i, y);
        }
        x += num;
      }
      b'$' => {
        x = 0;
        y += num;
      }
      _ => {
        panic!("invalid character {:?}", src.chars().next().unwrap());
      }
    }

    src = &src[1..];
  }

  node
}

/// Write a Life pattern to a RLE string.
///
/// RLE format: <https://www.conwaylife.com/wiki/Run_Length_Encoded>.
pub fn write(
  univ: &Universe,
  node: Node,
) -> String {
  let (x0, y0, x1, y1) = univ.boundary(node, 0, 0);
  let width = (x1 - x0) as u32;
  let mut output = format!("x = {}, y = {}, rule = B3/S23\n", width, y1 - y0);
  let data = crate::export::save_buffer(univ, node);

  let mut num_consec_next_rows = 0;
  for row in data {
    let mut unit = None;
    let mut num_unit = 0;
    let mut row_left_bits = width;
    for mut x in row {
      let mut left_bits = 32;
      while left_bits != 0 && row_left_bits != 0 {
        let num_new_unit;
        let new_unit = if x & 1 == 0 {
          num_new_unit = x.trailing_zeros().min(left_bits).min(row_left_bits);
          RleUnit::Dead
        } else {
          num_new_unit = x.trailing_ones();
          RleUnit::Alive
        };
        left_bits -= num_new_unit;
        row_left_bits -= num_new_unit;
        if num_new_unit == 32 {
          x = 0;
        } else {
          x >>= num_new_unit;
        }

        if Some(new_unit) != unit {
          if let Some(unit) = unit.take() {
            if num_consec_next_rows > 0 {
              RleUnit::NextRow.write(num_consec_next_rows, &mut output);
              num_consec_next_rows = 0;
            }

            unit.write(num_unit, &mut output);
            num_unit = 0;
          }
          unit = Some(new_unit);
        }
        num_unit += num_new_unit;
      }
    }

    if unit == Some(RleUnit::Dead) && num_unit == width {
      num_consec_next_rows += 1;
    } else {
      if num_consec_next_rows > 0 {
        RleUnit::NextRow.write(num_consec_next_rows, &mut output);
      }

      let unit = unit.unwrap();
      if unit != RleUnit::Dead {
        unit.write(num_unit, &mut output);
      }

      num_consec_next_rows = 1;
    }
  }

  if num_consec_next_rows > 1 {
    RleUnit::NextRow.write(num_consec_next_rows - 1, &mut output);
  }

  output.push('!');
  output.push('\n');
  output
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RleUnit {
  Dead,
  Alive,
  NextRow,
}

impl RleUnit {
  fn write(&self, num: u32, s: &mut String) {
    let c = match self {
      Self::Dead => 'b',
      Self::Alive => 'o',
      Self::NextRow => '$',
    };

    let buf = if num == 1 {
      c.to_string()
    } else {
      format!("{}{}", num, c)
    };

    if s.len() - s.rfind('\n').unwrap() + buf.len() > 71 {
      s.push('\n');
    }

    s.push_str(&buf);
  }
}

#[cfg(test)]
mod tests {
  use crate::universe::*;
  use super::*;

  #[test]
  fn read_glider() {
    let src = r"
x = 3, y = 3
bo$2bo$3o!
".trim();

    let mut univ = Universe::new();
    let node = read(src.to_owned(), &mut univ);
    assert_eq!(&univ.debug(node), r"
        
        
        
        
     #  
      # 
    ### 
        ".trim_start_matches('\n'));
  }
  */
  panic!()
}