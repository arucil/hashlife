use crate::rule::*;
use crate::universe::*;

/// Read a Life pattern from a RLE string.
///
/// RLE format: <https://www.conwaylife.com/wiki/Run_Length_Encoded>.
pub fn read(
  src: impl AsRef<str>,
) -> Universe {
  let mut src = src.as_ref();
  let header = src.lines().next().unwrap()
    .split(",")
    .map(|s| s.trim());

  let mut width = None::<u32>;
  let mut height = None::<u32>;
  let mut rule = None::<Rule>;
  for w in header {
    let kv = w.split("=").map(|s| s.trim()).collect::<Vec<_>>();
    if kv.len() != 2 {
      panic!("invalid header line");
    }
    match &kv[0][..] {
      "x" => {
        width = Some(kv[1].parse().expect("invalid x"));
      }
      "y" => {
        height = Some(kv[1].parse().expect("invalid y"));
      }
      "rule" => {
        rule = Some(parse_rule(&kv[1]).expect("invalid header line"));
      }
      _ => {}
    }
  }

  let _width = width.expect("missing x in header line");
  let _height = height.expect("missing y in header line");
  let rule = rule.unwrap_or(GAME_OF_LIFE);

  let mut uni = Universe::new(rule);
  src = &src[src.find('\n').unwrap_or(src.len())..];

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
      b'$' => {
        x = 0;
        y += num;
      }
      c => {
        if c.is_ascii_alphabetic() {
          for i in 0..num {
            uni.set(x + i, y, true);
          }
        x += num;
        } else {
          panic!("invalid character {:?}", src.chars().next().unwrap());
        }
      }
    }

    src = &src[1..];
  }

  uni
}

fn parse_rule(s: &str) -> Option<Rule> {
  let r = s.split("/").collect::<Vec<_>>();
  if r.len() != 2 {
    return None;
  }
  if r[0].as_bytes()[0].to_ascii_lowercase() != b'b' ||
    r[1].as_bytes()[0].to_ascii_lowercase() != b's'
  {
    return None;
  }

  let mut rule = Rule::new();
  for c in r[0].chars().skip(1) {
    let b = c.to_digit(10)?;
    if b > 8 {
      return None;
    }
    rule.set_birth(b as u8);
  }
  for c in r[1].chars().skip(1) {
    let b = c.to_digit(10)?;
    if b > 8 {
      return None;
    }
    rule.set_survival(b as u8);
  }

  Some(rule)
}

/// Write a Life pattern to a RLE string.
///
/// RLE format: <https://www.conwaylife.com/wiki/Run_Length_Encoded>.
pub fn write(
  univ: &Universe,
) -> String {
  let (left, top, right, bottom) = univ.boundary();
  let width = (right - left) as u32;
  let mut output = format!("x = {}, y = {}, rule = {}\n",
    width, bottom - top, univ.rule);
  let data = crate::export::write_buffer(univ);

  let mut num_consec_next_rows = 0;
  for row in data {
    let mut unit = None;
    let mut num_unit = 0;
    let mut row_left_bits = width;
    for mut x in row {
      let mut left_bits = 8;
      while left_bits != 0 && row_left_bits != 0 {
        let num_new_unit;
        let new_unit = if x & 128 == 0 {
          num_new_unit = x.leading_zeros().min(left_bits).min(row_left_bits);
          RleUnit::Dead
        } else {
          num_new_unit = x.leading_ones();
          RleUnit::Alive
        };
        left_bits -= num_new_unit;
        row_left_bits -= num_new_unit;
        if num_new_unit == 8 {
          x = 0;
        } else {
          x <<= num_new_unit;
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
  use super::*;

  #[test]
  fn read_glider() {
    let src = r"
x = 3, y = 3
bo$2bo$3o!
".trim();

    let uni = read(src.to_owned());
    assert_eq!(uni.debug(uni.root), vec![
      0b_0000_0000,
      0b_0000_0000,
      0b_0000_0000,
      0b_0000_0000,
      0b_0000_0100,
      0b_0000_0010,
      0b_0000_1110,
      0b_0000_0000,
    ]);
  }
}