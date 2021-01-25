use wasm_bindgen::prelude::*;
use algo::*;

#[wasm_bindgen]
pub struct Universe(universe::Universe);

#[wasm_bindgen]
impl Universe {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self(universe::Universe::new(rule::GAME_OF_LIFE))
  }

  pub fn read(rle: &str) -> Self {
    Self(rle::read(rle))
  }

  pub fn set(&mut self, x: i32, y: i32, alive: bool) {
    self.0.set(x as i64, y as i64, alive)
  }

  pub fn simulate(&mut self, num_gen: usize) {
    self.0.simulate(num_gen)
  }

  pub fn write_cells(&self, f: &js_sys::Function) {
    let null = JsValue::null();
    export::write_cells(&self.0, move |cell| {
      let b = (cell.nw as u64) << 48 | (cell.ne as u64) << 32
        | (cell.sw as u64) << 16
        | (cell.sw as u64);
      let b = unsafe { std::mem::transmute::<_, f64>(b) };
      let x = cell.x as i32;
      let y = cell.y as f32;
      f.call3(&null, &JsValue::from(x), &JsValue::from(y), &JsValue::from(b))
        .unwrap();
    })
  }
}