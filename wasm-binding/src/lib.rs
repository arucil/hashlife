use wasm_bindgen::prelude::*;
use algo::*;
use algo::universe::Boundary;

#[wasm_bindgen]
pub struct Universe(universe::Universe);

#[wasm_bindgen]
impl Universe {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self(universe::Universe::new(rule::GAME_OF_LIFE))
  }

  pub fn read(rle: &str) -> Result<Universe, JsValue> {
    Ok(Self(rle::read(rle)?))
  }

  pub fn set(&mut self, x: i32, y: i32, alive: bool) {
    self.0.set(x as i64, y as i64, alive)
  }

  pub fn simulate(&mut self, num_gen: usize) {
    self.0.simulate(num_gen)
  }

  pub fn write_cells(&self, viewport: &Viewport, f: &js_sys::Function) {
    let null = JsValue::null();
    let viewport = Boundary {
      left: viewport.left as i64,
      top: viewport.top as i64,
      right: viewport.right as i64,
      bottom: viewport.bottom as i64,
    };
    export::write_cells(&self.0, &viewport, move |cell| {
      let b = (cell.nw as u64) << 48
        | (cell.ne as u64) << 32
        | (cell.sw as u64) << 16
        | (cell.se as u64);
      let b = unsafe { std::mem::transmute::<_, f64>(b) };
      let x = cell.x as i32;
      let y = cell.y as i32;
      f.call3(&null, &JsValue::from(x), &JsValue::from(y), &JsValue::from(b))
        .unwrap();
    })
  }
}

#[wasm_bindgen]
pub struct Viewport {
  pub left: i32,
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
}

#[wasm_bindgen]
impl Viewport {
  #[wasm_bindgen(constructor)]
  pub fn new(left: i32, top: i32, width: u32, height: u32) -> Self {
    Self { left, top, right: left + width as i32, bottom: top + height as i32 }
  }
}