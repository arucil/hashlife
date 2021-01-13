use yew::prelude::*;
use wasm_bindgen::prelude::*;

struct Model {
  link: ComponentLink<Self>,
  value: i32,
}

enum Msg {
  Inc,
  Dec,
}

impl Component for Model {
  type Message = Msg;
  type Properties = ();

  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    Self {
      link,
      value: 0,
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::Inc => self.value += 1,
      Msg::Dec => self.value -= 1,
    }
    true
  }

  fn change(&mut self, _: Self::Properties) -> ShouldRender {
    false
  }

  fn view(&self) -> Html {
    html! {
      <div>
        <button onclick=self.link.callback(|_| Msg::Inc)>{ "+1" }</button>
        <button onclick=self.link.callback(|_| Msg::Dec)>{ "-1" }</button>
        <p>{ self.value }</p>
      </div>
    }
  }
}

#[wasm_bindgen(start)]
pub fn run_app() {
  App::<Model>::new().mount_to_body();
}
