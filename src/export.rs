use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use crate::universe::*;

pub fn save_image(uni: &Universe, path: impl AsRef<Path>) {
  let buffer = write_buffer(uni);
  println!("{:?}", buffer);

  let mut f = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)
    .unwrap();

  let (left, top, right, bottom) = uni.boundary();
  let w = right - left;
  let h = bottom - top;

  // BMP header
  f.write_all(&[
    0x42, 0x4D, 0x7E, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x3E, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00,
    w as u8, (w >> 8) as u8, (w >> 16) as u8, (w >> 24) as u8,
    h as u8, (h >> 8) as u8, (h >> 16) as u8, (h >> 24) as u8,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x06,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF,
    0xFF, 0x00, 0x00, 0x00, 0x00, 0x00
  ]).unwrap();

  let align = vec![0u8; (-(buffer[0].len() as isize)).rem_euclid(4) as usize];
  for row in buffer.into_iter().rev() {
    f.write_all(&row).unwrap();
    f.write_all(&align).unwrap();
  }
}

pub(crate) fn write_buffer(uni: &Universe) -> Vec<Vec<u8>> {
  let (left, top, right, bottom) = uni.boundary();
  if right <= left {
    panic!("empty");
  }


  let w = right - left;
  let h = (bottom - top) as usize;
  let bw = w + 7 >> 3;
  let mut buffer = vec![vec![0u8; bw as usize]; h];
  let shift = left.rem_euclid(8);

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
        buffer[y as usize + i][bx0 as usize] = bytes[i];
      }
    } else {
      for i in 0..8 {
        if y + i >= 0 && y + i < h as i64 {
          let b = bytes[i as usize];
          println!("-------------{}  {} {:08b} {:08b}", shift, bx1, b, b>>shift);
          buffer[(y + i) as usize][bx1 as usize] |= b >> shift;
          if bx0 >= 0 {
            buffer[(y + i) as usize][bx0 as usize] |= b << 8 - shift;
          }
        }
      }
    }
  });

  buffer
}