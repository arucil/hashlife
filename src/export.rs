use std::path::Path;
use image::{ImageBuffer, Luma};
use crate::universe::*;

pub fn save_image(uni: &Universe, node: Node, path: impl AsRef<Path>) {
  let b@(x0, y0, x1, y1) = uni.boundary(node, 0, 0);
  if x1 <= x0 {
    panic!("empty");
  }

  let mut buffer = ImageBuffer::new((x1 - x0) as u32, (y1 - y0) as u32);
  uni.write_cells(node, 0, 0, b, &mut |x, y| {
    assert!(x >= x0 && x < x1 && y >= y0 && y < y1);
    buffer.put_pixel((x - x0) as u32, (y - y0) as u32, Luma([255u8]));
  });

  buffer.save(path).unwrap();
}

pub fn save_buffer(uni: &Universe, node: Node) -> Vec<Vec<u32>> {
  let b@(x0, y0, x1, y1) = uni.boundary(node, 0, 0);
  if x1 <= x0 {
    panic!("empty");
  }

  let mut buffer = vec![vec![0u32; (x1 - x0 + 31) as usize / 32]; (y1 - y0) as usize];
  uni.write_cells(node, 0, 0, b, &mut |x, y| {
    assert!(x >= x0 && x < x1 && y >= y0 && y < y1);
    let x = (x - x0) as usize;
    let y = (y - y0) as usize;
    buffer[y][x >> 5] |= 1 << (x & 31);
  });

  buffer
}