use js_sys::JsString;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::EventTarget;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <h1>{"Gib file!"}</h1>
        <input type="file" name="inputfile"
            onchange={Callback::from(move |e: Event| {
                let target: Option<EventTarget> = e.target();

                let filevec: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec![]));

                let filevec_cloned = filevec.clone();
                let onload = Closure::wrap(Box::new(move |event: Event| {
                    // let element = event.target().unwrap().dyn_into::<web_sys::FileReader>().unwrap();
                    // let data = element.result().unwrap();
                    // let file_string: JsString = data.dyn_into::<JsString>().unwrap();
                    // let file_vec: Vec<u8> = file_string.iter().map(|x| x as u8).collect();
                    // *filevec_cloned.borrow_mut() = file_vec;
                    // web_sys::console::log_1(&"file loaded".to_string().into());
                }) as Box<dyn FnMut(_)>);

                let stuff = target.map(|target| {
                    let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    let files = target.files().unwrap();
                    web_sys::console::log_1(&format!("{:?}", files).into());
                    let file = files.get(0).unwrap();
                    let reader = web_sys::FileReader::new().unwrap();
                    // reader.set_onloadend(Some(onload.as_ref().unchecked_ref()));
                    // onload.forget();
                    reader.read_as_text(&file)
                });
                // web_sys::console::log_1(&format!("{:?}", filevec.borrow()).into());
            })}
            id="inputfile"/>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
