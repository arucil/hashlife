use crate::universe::*;

pub fn write_buffer(uni: &Universe) -> Vec<Vec<u8>> {
  let (left, top, right, bottom) = uni.boundary();
  if right <= left {
    panic!("empty");
  }

  let level = uni.level();
  let w = right - left;
  let h = (bottom - top) as usize;
  let bw = w + 7 >> 3;
  let mut buffer = vec![vec![0u8; bw as usize]; h];
  let shift = (if level == 3 { left + 4 } else { left }).rem_euclid(8);

  uni.write_cells(|nw, ne, sw, se, x0, y0| {
    let bytes = [
      (nw >> 8 & 0xf0 | ne >> 12 & 0xf) as u8,
      (nw >> 4 & 0xf0 | ne >> 8 & 0xf) as u8,
      (nw & 0xf0 | ne >> 4 & 0xf) as u8,
      (nw << 4 & 0xf0 | ne & 0xf) as u8,
      (sw >> 8 & 0xf0 | se >> 12 & 0xf) as u8,
      (sw >> 4 & 0xf0 | se >> 8 & 0xf) as u8,
      (sw & 0xf0 | se >> 4 & 0xf) as u8,
      (sw << 4 & 0xf0 | se & 0xf) as u8,
    ];
    let x = x0 - left;
    let bx0 = x.div_euclid(8);
    let bx1 = (x + 7).div_euclid(8);
    let y = y0 - top;
    if shift == 0 {
      for i in 0..8 {
        if y + i >= 0 && y + i < h as i64 {
          buffer[(y + i) as usize][bx0 as usize] = bytes[i as usize];
        }
      }
    } else {
      for i in 0..8 {
        if y + i >= 0 && y + i < h as i64 {
          let b = bytes[i as usize];
          if bx1 < bw {
            buffer[(y + i) as usize][bx1 as usize] |= b << shift;
          }
          if bx0 >= 0 {
            buffer[(y + i) as usize][bx0 as usize] |= b >> 8 - shift;
          }
        }
      }
    }
  });

  buffer
}

pub struct CellData {
  pub nw: u16,
  pub ne: u16,
  pub sw: u16,
  pub se: u16,
  pub x: i64,
  pub y: i64,
}

pub fn write_cells(
  univ: &Universe,
  mut f: impl FnMut(CellData),
) {
  univ.write_cells(|nw, ne, sw, se, x, y| {
    f(CellData { nw, ne, sw, se, x, y })
  })
}