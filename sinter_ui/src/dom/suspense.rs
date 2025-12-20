use crate::dom::element::tag::div;
use crate::dom::view::View;
use crate::reactivity::SuspenseContext;
use crate::reactivity::{create_effect, create_scope, provide_context};
use web_sys::Node;

pub struct Suspense<V, F> {
    children: V,
    fallback: F,
}

pub fn suspense() -> Suspense<(), ()> {
    Suspense {
        children: (),
        fallback: (),
    }
}

impl<V, F> Suspense<V, F> {
    pub fn children<NewV>(self, children: NewV) -> Suspense<NewV, F> {
        Suspense {
            children,
            fallback: self.fallback,
        }
    }

    pub fn fallback<NewF>(self, fallback: NewF) -> Suspense<V, NewF> {
        Suspense {
            children: self.children,
            fallback,
        }
    }
}

// 支持 children/fallback 作为返回 View 的闭包
impl<V, F, VRes, FRes> View for Suspense<V, F>
where
    V: Fn() -> VRes + 'static,
    VRes: View + 'static,
    F: Fn() -> FRes + 'static,
    FRes: View + 'static,
{
    fn mount(self, parent: &Node) {
        let children_fn = self.children;
        let fallback_fn = self.fallback;

        let parent_clone = parent.clone();

        // 包裹在作用域中以管理上下文和生命周期
        create_scope(move || {
            let ctx = SuspenseContext::new();
            if let Err(e) = provide_context(ctx) {
                crate::error::handle_error(e);
                return;
            }

            let count = ctx.count;

            // 1. 内容包装器（加载时隐藏）
            let content_wrapper = div().class("suspense-content");
            let _ = content_wrapper.clone().style(move || {
                if count.get().unwrap_or(0) > 0 {
                    "display: none"
                } else {
                    "display: block"
                }
            });
            content_wrapper.clone().mount(&parent_clone);
            let content_root = content_wrapper.dom_element;

            create_effect(move || {
                let view = children_fn();
                content_root.set_inner_html("");
                view.mount(&content_root);
            });

            // 2. 后备包装器（加载时可见）
            let fallback_wrapper = div().class("suspense-fallback");
            let _ = fallback_wrapper.clone().style(move || {
                if count.get().unwrap_or(0) > 0 {
                    "display: block"
                } else {
                    "display: none"
                }
            });
            fallback_wrapper.clone().mount(&parent_clone);
            let fallback_root = fallback_wrapper.dom_element;

            create_effect(move || {
                let view = fallback_fn();
                fallback_root.set_inner_html("");
                view.mount(&fallback_root);
            });
        });
    }
}
