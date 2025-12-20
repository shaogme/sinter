use sinter_ui::dom::suspense::suspense;
use sinter_ui::dom::tag::*;
use sinter_ui::prelude::*;
use std::rc::Rc;
use web_sys::Node;

// --- 组件重构：Props Builder Pattern ---

// 1. 定义组件结构体
// 不再是简单的函数，而是一个持有配置状态的结构体
pub struct Card<V> {
    title: String,
    child: V,
    elevation: u8,                  // 可选属性示例
    on_hover: Option<Rc<dyn Fn()>>, // 事件属性示例
}

// 2. 实现构建器方法
impl Card<()> {
    // 初始构造器，设置默认值
    pub fn new() -> Self {
        Self {
            title: "Default Title".to_string(),
            child: (),
            elevation: 1,
            on_hover: None,
        }
    }
}

impl<V> Card<V> {
    // 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    // 设置阴影深度 (链式调用)
    pub fn elevation(mut self, elevation: u8) -> Self {
        self.elevation = elevation;
        self
    }

    // 设置子视图 (这会改变 Card 的泛型类型 V)
    pub fn child<NewV: View>(self, child: NewV) -> Card<NewV> {
        Card {
            title: self.title,
            child, // 这里的 child 是具体的 View
            elevation: self.elevation,
            on_hover: self.on_hover,
        }
    }

    // 设置悬停事件
    pub fn on_hover<F: Fn() + 'static>(mut self, f: F) -> Self {
        self.on_hover = Some(Rc::new(f));
        self
    }
}

// 3. 实现 View 特征 (渲染逻辑)
impl<V: View> View for Card<V> {
    fn mount(self, parent: &Node) {
        // 根据属性计算样式
        let style = format!(
            "border: 1px solid #e0e0e0; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 4px {}px rgba(0,0,0,0.1); transition: transform 0.2s;",
            self.elevation * 4
        );

        let mut root = div().class("card").style(&style);

        // 如果绑定了事件
        if let Some(cb) = self.on_hover {
            // 这里为了演示简单，只是绑定了 click，实际可以使用 mouseover
            root = root.on_click(move || cb());
        }

        root.child((
            h1().style("margin-top: 0; font-size: 1.2rem; color: #333;")
                .text(self.title),
            self.child,
        ))
        .mount(parent)
    }
}

// --- 子组件：演示 Context API ---

// 这是一个深层嵌套的组件，它不需要通过参数接收 count
fn counter_display() -> SinterResult<impl View> {
    // ✨ 魔法：直接从上下文获取 Signal
    // unwrap here is okay for demo or should be handled User said NO unwrap.
    // context missing -> error
    let count = use_context::<ReadSignal<i32>>()
        .ok_or_else(|| sinter_ui::SinterError::Reactivity("Context 'count' not found!".into()))?;

    Ok(div()
        .style("margin-top: 10px; color: #888; font-size: 0.9rem;")
        .child((
            span().text("Global Context Status: "),
            span()
                .style("font-weight: bold; color: #6200ea;")
                .text(count), // 直接绑定 Signal
        )))
}

fn counter_controls() -> SinterResult<impl View> {
    // ✨ 魔法：获取 setter，分离读写关注点
    let set_count = use_context::<WriteSignal<i32>>().ok_or_else(|| {
        sinter_ui::SinterError::Reactivity("Context 'set_count' not found!".into())
    })?;
    let count = use_context::<ReadSignal<i32>>()
        .ok_or_else(|| sinter_ui::SinterError::Reactivity("Context count not found".into()))?;

    Ok(div().style("display: flex; align-items: center; gap: 15px;").child((
        button()
            .style("padding: 8px 16px; border-radius: 4px; border: 1px solid #ccc; cursor: pointer;")
            .text("-")
            .on_click(move || { let _ = set_count.update(|n| *n -= 1); }),
        span()
            .style("font-size: 1.5rem; font-weight: bold; min-width: 30px; text-align: center;")
            .text(count),
        button()
            .style("padding: 8px 16px; border-radius: 4px; border: 1px solid #ccc; cursor: pointer;")
            .text("+")
            .on_click(move || { let _ = set_count.update(|n| *n += 1); }),
    )))
}

// --- Main ---

fn main() -> () {
    console_error_panic_hook::set_once();
    let window = web_sys::window().expect("No Window");
    let document = window.document().expect("No Document");
    let app_container = document.get_element_by_id("app").expect("No App Element");

    // 1. 创建 Root Scope (防止闭包过早 Drop)
    create_scope(move || {
        // 2. 状态定义
        let (count, set_count) = create_signal(0);
        let (name, set_name) = create_signal("Rustacean".to_string());

        let is_high = create_memo(move || match count.get() {
            Some(c) => c > 5,
            None => false,
        });

        // Async Resource for Suspense Demo
        let async_data = create_resource(
            || (),
            |_| async {
                gloo_timers::future::TimeoutFuture::new(2_000).await;
                "Loaded Data from Server!".to_string()
            }
        ).expect("Failed to create resource");

        // 3. ✨ 提供上下文 (Dependency Injection)
        provide_context(count).expect("应该在create_scope内调用");
        provide_context(set_count).expect("应该在create_scope内调用");

        // 4. 构建 UI
        let app = div()
                .class("app-container")
                .style("font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;")
                .child((
                    // Header
                    div().style("text-align: center; margin-bottom: 30px;").child((
                        h1().text("Sinter UI: Next Gen"),
                        p().style("color: #666").text("Builder Pattern + Context API + Suspense"),
                    )),

                    // Card 1: 使用 Builder 模式 + Context
                    Card::new()
                        .title("Context-Aware Counter")
                        .elevation(3) // 设置阴影
                        .on_hover(|| { let _ = web_sys::console::log_1(&"Card Hovered!".into()); })
                        .child((
                            // 注意：这里没有传递 props！
                            counter_controls(),
                            counter_display(),
                        )),

                    // Card 2: 传统的直接绑定 (演示混合使用)
                    Card::new()
                        .title("Input Binding")
                        .child(div().child((
                            div().style("margin-bottom: 10px").child((
                                span().text("Hello, "),
                                span().style("color: #007bff; font-weight: bold;").text(name),
                                span().text("!"),
                            )),
                            input()
                                .attr("type", "text")
                                .attr("placeholder", "Enter your name")
                                .style("padding: 8px; border: 1px solid #ccc; border-radius: 4px; width: 100%; box-sizing: border-box;")
                                .attr("value", name)
                                .on_input(move |val| { let _ = set_name.set(val); })
                        ))),

                    // Card 3: 流式控制流
                    Card::new()
                        .title("Control Flow")
                        .child(
                            is_high
                                .when(|| div()
                                    .style("background: #ffebee; color: #c62828; padding: 10px; border-radius: 4px;")
                                    .text("⚠️ Warning: Count is getting high!"))
                                .otherwise(|| div()
                                    .style("background: #e8f5e9; color: #2e7d32; padding: 10px; border-radius: 4px;")
                                    .text("✓ System works normally."))
                        ),
                    
                    // Card 4: Suspense (Async)
                    Card::new()
                        .title("Suspense (Async Loading)")
                        .child(
                             suspense()
                                .fallback(|| div().style("color: orange; font-style: italic;").text("Loading data (approx 2s)..."))
                                .children(move || {
                                    div()
                                        .style("color: #2e7d32; font-weight: bold; background: #e8f5e9; padding: 10px; border-radius: 4px;")
                                        .text(move || async_data.get().unwrap_or("Waiting...".to_string()))
                                })
                        )
                ));

        app.mount(&app_container);
    });
}
