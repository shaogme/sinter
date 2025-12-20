use crate::SinterError;
use crate::reactivity::{ReadSignal, create_effect};
use web_sys::Element as WebElem;

// --- 核心魔法：多态属性特征 ---

/// 任何可以用作 HTML 属性值的类型
pub trait AttributeValue {
    fn apply(self, el: &WebElem, name: &str);
}

// 1. 静态字符串支持
impl AttributeValue for &str {
    fn apply(self, el: &WebElem, name: &str) {
        if let Err(e) = el.set_attribute(name, self).map_err(SinterError::from) {
            crate::error::handle_error(e);
        }
    }
}

impl AttributeValue for String {
    fn apply(self, el: &WebElem, name: &str) {
        if let Err(e) = el.set_attribute(name, &self).map_err(SinterError::from) {
            crate::error::handle_error(e);
        }
    }
}

impl AttributeValue for &String {
    fn apply(self, el: &WebElem, name: &str) {
        if let Err(e) = el.set_attribute(name, &self).map_err(SinterError::from) {
            crate::error::handle_error(e);
        }
    }
}

impl AttributeValue for bool {
    fn apply(self, el: &WebElem, name: &str) {
        let res = if self {
            el.set_attribute(name, "").map_err(SinterError::from)
        } else {
            el.remove_attribute(name).map_err(SinterError::from)
        };
        if let Err(e) = res {
            crate::error::handle_error(e);
        }
    }
}

// 2. 动态闭包支持 (Reactive Closure)
impl<F, S> AttributeValue for F
where
    F: Fn() -> S + 'static,
    S: Into<String>,
{
    fn apply(self, el: &WebElem, name: &str) {
        let el = el.clone();
        let name = name.to_string();
        // 自动创建副作用
        create_effect(move || {
            let value = self().into();
            if let Err(e) = el.set_attribute(&name, &value).map_err(SinterError::from) {
                crate::error::handle_error(e);
            }
        });
    }
}

// 3. 直接 Signal 支持
impl<T> AttributeValue for ReadSignal<T>
where
    T: Into<String> + Clone + 'static,
{
    fn apply(self, el: &WebElem, name: &str) {
        let el = el.clone();
        let name = name.to_string();
        // Signal 是 Copy 的，直接移动进去
        let signal = self;
        create_effect(move || {
            if let Some(v) = signal.get() {
                let value = v.into();
                if let Err(e) = el.set_attribute(&name, &value).map_err(SinterError::from) {
                    crate::error::handle_error(e);
                }
            }
        });
    }
}
