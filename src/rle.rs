use regex::Regex;
use crate::universe::*;

/// Read a Life pattern from a RLE string.
///
/// RLE format: <https://www.conwaylife.com/wiki/Run_Length_Encoded>.
pub fn read(
  src: impl AsRef<str>,
  univ: &mut Universe,
) -> Node {
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

  let level = 33 - width.max(height).leading_zeros();
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
}