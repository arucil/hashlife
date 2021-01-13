use algo::universe::Universe;
use algo::export;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

pub fn save_image(uni: &Universe, path: impl AsRef<Path>) {
  let buffer = export::write_buffer(uni);

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