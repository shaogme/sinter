use crate::dom::element::Element;
use crate::reactivity::{ReadSignal, create_effect};
use crate::{SinterError, SinterResult};
use std::fmt::Display;
use web_sys::Node;

/// 视图特征 (View Trait)
/// 核心特征：定义了如何将一个东西挂载到 DOM 上。
pub trait View {
    fn mount(self, parent: &Node);
}

// --- View Trait Implementations ---

// 1. Element 本身就是 View
impl View for Element {
    fn mount(self, parent: &Node) {
        if let Err(e) = parent
            .append_child(&self.dom_element)
            .map_err(SinterError::from)
        {
            crate::error::handle_error(e);
        }
    }
}

// 2. 静态文本 (String, &str)
impl View for String {
    fn mount(self, parent: &Node) {
        let window = match web_sys::window().ok_or_else(|| SinterError::Dom("No window".into())) {
            Ok(w) => w,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let document = match window
            .document()
            .ok_or_else(|| SinterError::Dom("No document".into()))
        {
            Ok(d) => d,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let node = document.create_text_node(&self);
        if let Err(e) = parent.append_child(&node).map_err(SinterError::from) {
            crate::error::handle_error(e);
        }
    }
}

impl View for &str {
    fn mount(self, parent: &Node) {
        let window = match web_sys::window().ok_or_else(|| SinterError::Dom("No window".into())) {
            Ok(w) => w,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let document = match window
            .document()
            .ok_or_else(|| SinterError::Dom("No document".into()))
        {
            Ok(d) => d,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let node = document.create_text_node(self);
        if let Err(e) = parent.append_child(&node).map_err(SinterError::from) {
            crate::error::handle_error(e);
        }
    }
}

// 3. 基础类型支持
macro_rules! impl_view_for_primitive {
    ($($t:ty),*) => {
        $(
            impl View for $t {
                fn mount(self, parent: &Node) {
                    let window = match web_sys::window().ok_or_else(|| SinterError::Dom("No window".into())) {
                        Ok(w) => w,
                        Err(e) => {
                            crate::error::handle_error(e);
                            return;
                        }
                    };
                    let document = match window.document().ok_or_else(|| SinterError::Dom("No document".into())) {
                        Ok(d) => d,
                        Err(e) => {
                            crate::error::handle_error(e);
                            return;
                        }
                    };
                    let node = document.create_text_node(&self.to_string());
                    if let Err(e) = parent.append_child(&node).map_err(SinterError::from) {
                        crate::error::handle_error(e);
                    }
                }
            }
        )*
    };
}

impl_view_for_primitive!(
    i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64, bool, char
);

// 4. 动态闭包支持 (Lazy View / Dynamic Text)
impl<F, S> View for F
where
    F: Fn() -> S + 'static,
    S: Display + 'static,
{
    fn mount(self, parent: &Node) {
        let window = match web_sys::window().ok_or_else(|| SinterError::Dom("No window".into())) {
            Ok(w) => w,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let document = match window
            .document()
            .ok_or_else(|| SinterError::Dom("No document".into()))
        {
            Ok(d) => d,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let node = document.create_text_node("");
        if let Err(e) = parent.append_child(&node).map_err(SinterError::from) {
            crate::error::handle_error(e);
            return;
        }

        create_effect(move || {
            let value = self();
            node.set_node_value(Some(&value.to_string()));
        });
    }
}

// 5. 直接 Signal 支持
impl<T> View for ReadSignal<T>
where
    T: Display + Clone + 'static,
{
    fn mount(self, parent: &Node) {
        let window = match web_sys::window().ok_or_else(|| SinterError::Dom("No window".into())) {
            Ok(w) => w,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        let document = match window
            .document()
            .ok_or_else(|| SinterError::Dom("No document".into()))
        {
            Ok(d) => d,
            Err(e) => {
                crate::error::handle_error(e);
                return;
            }
        };
        // 1. 创建占位符
        let node = document.create_text_node("");
        if let Err(e) = parent.append_child(&node).map_err(SinterError::from) {
            crate::error::handle_error(e);
            return;
        }

        // 2. 创建副作用
        let signal = self;
        create_effect(move || {
            if let Some(value) = signal.get() {
                node.set_node_value(Some(&value.to_string()));
            }
        });
    }
}

// 6. 容器类型支持
impl<V: View> View for Option<V> {
    fn mount(self, parent: &Node) {
        if let Some(v) = self {
            v.mount(parent);
        }
    }
}

impl<V: View> View for Vec<V> {
    fn mount(self, parent: &Node) {
        for v in self {
            v.mount(parent);
        }
    }
}

// 7. 元组支持
macro_rules! impl_view_for_tuple {
    ($($name:ident),*) => {
        impl<$($name: View),*> View for ($($name,)*) {
            #[allow(non_snake_case)]
            fn mount(self, parent: &Node) {
                let ($($name,)*) = self;
                $($name.mount(parent);)*
            }
        }
    }
}
impl_view_for_tuple!(A);
impl_view_for_tuple!(A, B);
impl_view_for_tuple!(A, B, C);
impl_view_for_tuple!(A, B, C, D);
impl_view_for_tuple!(A, B, C, D, E);
impl_view_for_tuple!(A, B, C, D, E, F);
impl_view_for_tuple!(A, B, C, D, E, F, G);
impl_view_for_tuple!(A, B, C, D, E, F, G, H);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

// 8. Result 支持
impl<V: View> View for SinterResult<V> {
    fn mount(self, parent: &Node) {
        match self {
            Ok(v) => v.mount(parent),
            Err(e) => crate::error::handle_error(e),
        }
    }
}

// --- AnyView (Type Erasure) ---

/// 辅助特征，用于支持 Box<dyn View> 的移动语义挂载
pub trait Render {
    fn mount_boxed(self: Box<Self>, parent: &Node);
}

impl<V: View + 'static> Render for V {
    fn mount_boxed(self: Box<Self>, parent: &Node) {
        (*self).mount(parent)
    }
}

/// 类型擦除的 View，可以持有任何 View 的实现。
/// 用于从同一个函数返回不同类型的 View（例如：主题）。
pub struct AnyView(Box<dyn Render>);

impl AnyView {
    pub fn new<V: View + 'static>(view: V) -> Self {
        Self(Box::new(view))
    }
}

impl View for AnyView {
    fn mount(self, parent: &Node) {
        self.0.mount_boxed(parent)
    }
}

pub trait IntoAnyView {
    fn into_any(self) -> AnyView;
}

impl<V: View + 'static> IntoAnyView for V {
    fn into_any(self) -> AnyView {
        AnyView::new(self)
    }
}
