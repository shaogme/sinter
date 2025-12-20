use crate::SinterError;
use crate::dom::attribute::AttributeValue;
use crate::dom::view::View;
use crate::reactivity::on_cleanup;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Element as WebElem;

/// 基础 DOM 元素包装器
#[derive(Clone)]
pub struct Element {
    pub dom_element: WebElem,
}

impl Element {
    pub fn new(tag: &str) -> Self {
        let window = web_sys::window().expect("No global window");
        let document = window.document().expect("No document");
        let dom_element = document
            .create_element(tag)
            .expect("Failed to create element");
        Self { dom_element }
    }

    pub fn new_svg(tag: &str) -> Self {
        let window = web_sys::window().expect("No global window");
        let document = window.document().expect("No document");
        let dom_element = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), tag)
            .expect("Failed to create SVG element");
        Self { dom_element }
    }

    // --- 统一的属性 API ---

    pub fn attr(self, name: &str, value: impl AttributeValue) -> Self {
        value.apply(&self.dom_element, name);
        self
    }

    pub fn id(self, value: impl AttributeValue) -> Self {
        self.attr("id", value)
    }

    pub fn class(self, value: impl AttributeValue) -> Self {
        self.attr("class", value)
    }

    pub fn style(self, value: impl AttributeValue) -> Self {
        self.attr("style", value)
    }

    // --- 事件 API ---

    pub fn on_click<F>(self, callback: F) -> Self
    where
        F: Fn() + 'static,
    {
        let closure = Closure::wrap(Box::new(move || {
            callback();
        }) as Box<dyn Fn()>);

        let js_value = closure.as_ref().unchecked_ref::<js_sys::Function>();
        if let Err(e) = self
            .dom_element
            .add_event_listener_with_callback("click", js_value)
            .map_err(SinterError::from)
        {
            crate::error::handle_error(e);
            return self;
        }

        let target = self.dom_element.clone();
        let js_fn = js_value.clone();

        // 注册清理回调
        on_cleanup(move || {
            let _ = target.remove_event_listener_with_callback("click", &js_fn);
            drop(closure);
        });

        self
    }

    pub fn on_input<F>(self, mut callback: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target() {
                let input = target.unchecked_into::<web_sys::HtmlInputElement>();
                callback(input.value());
            } else {
                let err = SinterError::Dom("Input event has no target".into());
                crate::error::handle_error(err);
            }
        }) as Box<dyn FnMut(_)>);

        let js_value = closure.as_ref().unchecked_ref::<js_sys::Function>();
        if let Err(e) = self
            .dom_element
            .add_event_listener_with_callback("input", js_value)
            .map_err(SinterError::from)
        {
            crate::error::handle_error(e);
            return self;
        }

        let target = self.dom_element.clone();
        let js_fn = js_value.clone();

        // 注册清理回调
        on_cleanup(move || {
            let _ = target.remove_event_listener_with_callback("input", &js_fn);
            drop(closure);
        });

        self
    }

    // --- 统一的子节点/文本 API ---

    pub fn child<V: View>(self, view: V) -> Self {
        view.mount(&self.dom_element);
        self
    }

    pub fn text<V: View>(self, content: V) -> Self {
        self.child(content)
    }
}

pub mod tag {
    use super::*;

    // --- HTML Tags ---
    pub fn div() -> Element {
        Element::new("div")
    }
    pub fn span() -> Element {
        Element::new("span")
    }
    pub fn h1() -> Element {
        Element::new("h1")
    }
    pub fn h2() -> Element {
        Element::new("h2")
    }
    pub fn h3() -> Element {
        Element::new("h3")
    }
    pub fn h4() -> Element {
        Element::new("h4")
    }
    pub fn h5() -> Element {
        Element::new("h5")
    }
    pub fn h6() -> Element {
        Element::new("h6")
    }
    pub fn p() -> Element {
        Element::new("p")
    }
    pub fn a() -> Element {
        Element::new("a")
    }
    pub fn button() -> Element {
        Element::new("button")
    }
    pub fn img() -> Element {
        Element::new("img")
    }
    pub fn input() -> Element {
        Element::new("input")
    }
    pub fn ul() -> Element {
        Element::new("ul")
    }
    pub fn ol() -> Element {
        Element::new("ol")
    }
    pub fn li() -> Element {
        Element::new("li")
    }
    pub fn nav() -> Element {
        Element::new("nav")
    }
    pub fn main() -> Element {
        Element::new("main")
    }
    pub fn footer() -> Element {
        Element::new("footer")
    }
    pub fn aside() -> Element {
        Element::new("aside")
    }
    pub fn br() -> Element {
        Element::new("br")
    }
    pub fn hr() -> Element {
        Element::new("hr")
    }
    pub fn article() -> Element {
        Element::new("article")
    }
    pub fn header() -> Element {
        Element::new("header")
    }
    pub fn time() -> Element {
        Element::new("time")
    }
    pub fn figure() -> Element {
        Element::new("figure")
    }
    pub fn figcaption() -> Element {
        Element::new("figcaption")
    }
    pub fn blockquote() -> Element {
        Element::new("blockquote")
    }
    pub fn pre() -> Element {
        Element::new("pre")
    }
    pub fn code() -> Element {
        Element::new("code")
    }
    pub fn em() -> Element {
        Element::new("em")
    }
    pub fn strong() -> Element {
        Element::new("strong")
    }
    pub fn s() -> Element {
        Element::new("s")
    }
    pub fn table() -> Element {
        Element::new("table")
    }
    pub fn thead() -> Element {
        Element::new("thead")
    }
    pub fn tbody() -> Element {
        Element::new("tbody")
    }
    pub fn tr() -> Element {
        Element::new("tr")
    }
    pub fn td() -> Element {
        Element::new("td")
    }
    pub fn label() -> Element {
        Element::new("label")
    }
    pub fn section() -> Element {
        Element::new("section")
    }

    // --- SVG Tags ---
    pub fn svg() -> Element {
        Element::new_svg("svg")
    }
    pub fn path() -> Element {
        Element::new_svg("path")
    }
    pub fn defs() -> Element {
        Element::new_svg("defs")
    }
    pub fn filter() -> Element {
        Element::new_svg("filter")
    }
    pub fn fe_turbulence() -> Element {
        Element::new_svg("feTurbulence")
    }
    pub fn fe_component_transfer() -> Element {
        Element::new_svg("feComponentTransfer")
    }
    pub fn fe_func_r() -> Element {
        Element::new_svg("feFuncR")
    }
    pub fn fe_func_g() -> Element {
        Element::new_svg("feFuncG")
    }
    pub fn fe_func_b() -> Element {
        Element::new_svg("feFuncB")
    }
    pub fn fe_gaussian_blur() -> Element {
        Element::new_svg("feGaussianBlur")
    }
    pub fn fe_specular_lighting() -> Element {
        Element::new_svg("feSpecularLighting")
    }
    pub fn fe_point_light() -> Element {
        Element::new_svg("fePointLight")
    }
    pub fn fe_composite() -> Element {
        Element::new_svg("feComposite")
    }
    pub fn fe_displacement_map() -> Element {
        Element::new_svg("feDisplacementMap")
    }
    pub fn g() -> Element {
        Element::new_svg("g")
    }
    pub fn rect() -> Element {
        Element::new_svg("rect")
    }
    pub fn circle() -> Element {
        Element::new_svg("circle")
    }
    pub fn line() -> Element {
        Element::new_svg("line")
    }
    pub fn polyline() -> Element {
        Element::new_svg("polyline")
    }
    pub fn polygon() -> Element {
        Element::new_svg("polygon")
    }
}
