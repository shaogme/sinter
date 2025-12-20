use crate::dom::View;
use crate::dom::tag::div;
use crate::reactivity::create_effect;
use web_sys::Node;

/// Dynamic 组件：用于渲染动态内容，类似于 SolidJS 的 <Dynamic>
///
/// 它接受一个返回 `View` 的闭包，并在该闭包的依赖发生变化时自动重新渲染。
/// 通常用于根据状态动态切换组件。
///
/// # 示例
///
/// ```rust,no_run
/// let (component_name, set_component_name) = create_signal("A");
///
/// Dynamic::new(move || {
///     let name = component_name.get();
///     if name == "A" {
///         ComponentA().into_any()
///     } else {
///         ComponentB().into_any()
///     }
/// })
/// ```
pub struct Dynamic<V, F>
where
    V: View,
    F: Fn() -> V + 'static,
{
    view_fn: F,
}

impl<V, F> Dynamic<V, F>
where
    V: View,
    F: Fn() -> V + 'static,
{
    pub fn new(f: F) -> Self {
        Self { view_fn: f }
    }
}

impl<V, F> View for Dynamic<V, F>
where
    V: View,
    F: Fn() -> V + 'static,
{
    fn mount(self, parent: &Node) {
        // 使用 display: contents 的 div 作为挂载锚点
        // 这允许我们在内部完全替换内容而不破坏父节点的结构
        let container = div().style("display: contents");
        container.clone().mount(parent);
        let root = container.dom_element;

        let view_fn = self.view_fn;

        create_effect(move || {
            // view_fn 应该包含它需要的响应式依赖
            let new_view = view_fn();

            // 清空旧内容
            root.set_inner_html("");

            // 挂载新内容
            new_view.mount(&root);
        });
    }
}
