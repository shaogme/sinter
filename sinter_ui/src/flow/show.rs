use crate::dom::View;
use crate::dom::tag::div;
use crate::reactivity::{ReadSignal, create_effect};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::Node;

/// Show 组件：根据条件渲染不同的视图
pub struct Show<Cond, ViewFn, FalsyViewFn, V1, V2> {
    condition: Cond,
    view: ViewFn,
    fallback: Option<FalsyViewFn>,
    _marker: std::marker::PhantomData<(V1, V2)>,
}

impl<Cond, ViewFn, FalsyViewFn, V1, V2> Show<Cond, ViewFn, FalsyViewFn, V1, V2>
where
    Cond: Fn() -> Option<bool> + 'static,
    ViewFn: Fn() -> V1 + 'static,
    FalsyViewFn: Fn() -> V2 + 'static,
    V1: View,
    V2: View,
{
    pub fn new(condition: Cond, view: ViewFn, fallback: Option<FalsyViewFn>) -> Self {
        Self {
            condition,
            view,
            fallback,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Cond, ViewFn, FalsyViewFn, V1, V2> View for Show<Cond, ViewFn, FalsyViewFn, V1, V2>
where
    Cond: Fn() -> Option<bool> + 'static,
    ViewFn: Fn() -> V1 + 'static,
    FalsyViewFn: Fn() -> V2 + 'static,
    V1: View,
    V2: View,
{
    fn mount(self, parent: &Node) {
        let container = div().style("display: contents");

        container.clone().mount(parent);
        let root = container.dom_element;

        let cond = self.condition;
        let view_fn = self.view;
        let fallback_fn = self.fallback;

        let prev_state = Rc::new(RefCell::new(None::<bool>));

        create_effect(move || {
            // Check condition, might fail if signal dropped
            let val = (cond)().unwrap_or(false);
            let mut state = prev_state.borrow_mut();

            if *state == Some(val) {
                return;
            }

            root.set_inner_html("");

            if val {
                (view_fn)().mount(&root);
            } else if let Some(fb) = fallback_fn.as_ref() {
                (fb)().mount(&root);
            }

            *state = Some(val);
        });
    }
}

// --- Fluent API for Show (NEW) ---

/// 用于构建 Show 组件的构建器
pub struct ShowBuilder<Cond, ViewFn, V1> {
    condition: Cond,
    view: ViewFn,
    _marker: std::marker::PhantomData<V1>,
}

impl<Cond, ViewFn, V1> ShowBuilder<Cond, ViewFn, V1>
where
    Cond: Fn() -> Option<bool> + 'static,
    ViewFn: Fn() -> V1 + 'static,
    V1: View,
{
    /// 定义 "Else" 分支，返回完整的 Show 组件
    pub fn otherwise<FalsyViewFn, V2>(
        self,
        fallback: FalsyViewFn,
    ) -> Show<Cond, ViewFn, FalsyViewFn, V1, V2>
    where
        FalsyViewFn: Fn() -> V2 + 'static,
        V2: View,
    {
        Show::new(self.condition, self.view, Some(fallback))
    }
}

// 让 ShowBuilder 本身也是 View (默认没有 fallback)
impl<Cond, ViewFn, V1> View for ShowBuilder<Cond, ViewFn, V1>
where
    Cond: Fn() -> Option<bool> + 'static,
    ViewFn: Fn() -> V1 + 'static,
    V1: View,
{
    fn mount(self, parent: &Node) {
        // 使用一个空的 dummy fallback
        // String 实现了 View，所以闭包返回 String 是合法的
        let dummy_fallback = || "";
        Show::new(self.condition, self.view, Some(dummy_fallback)).mount(parent)
    }
}

/// Signal 扩展特质，提供 .when() 语法糖
pub trait SignalShowExt {
    // 使用 Box<dyn> 简化返回类型签名，避免复杂的泛型嵌套
    type Cond: Fn() -> Option<bool> + 'static;

    fn when<V, F>(self, view: F) -> ShowBuilder<Self::Cond, F, V>
    where
        V: View,
        F: Fn() -> V + 'static;
}

// 为 ReadSignal<bool> 实现扩展
impl SignalShowExt for ReadSignal<bool> {
    type Cond = Box<dyn Fn() -> Option<bool>>;

    fn when<V, F>(self, view: F) -> ShowBuilder<Self::Cond, F, V>
    where
        V: View,
        F: Fn() -> V + 'static,
    {
        // 捕获 Signal，创建一个返回 bool 的闭包
        let signal = self;
        let condition = Box::new(move || signal.get());

        ShowBuilder {
            condition,
            view,
            _marker: std::marker::PhantomData,
        }
    }
}
