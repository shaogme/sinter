pub mod runtime;

pub use runtime::NodeId;

use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::reactivity::runtime::{NodeType, RUNTIME, run_effect};
use crate::{SinterError, SinterResult};

// --- Signal 信号 API ---

/// `ReadSignal` 是一个用于读取响应式数据的句柄。
/// 它实现了 `Copy` 和 `Clone`，因此可以廉价地在闭包之间传递。
/// 当从 `ReadSignal` 读取值时，会自动追踪当前的副作用上下文（Effect）。
pub struct ReadSignal<T> {
    pub(crate) id: NodeId,
    pub(crate) marker: PhantomData<T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ReadSignal<T> {}

/// `WriteSignal` 是一个用于写入/更新响应式数据的句柄。
/// 它也实现了 `Copy` 和 `Clone`。
/// 更新 `WriteSignal` 的值会触发所有依赖于对应 `ReadSignal` 的副作用（Effect）。
pub struct WriteSignal<T> {
    pub(crate) id: NodeId,
    pub(crate) marker: PhantomData<T>,
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for WriteSignal<T> {}

/// 创建一个新的 Signal（信号）。
/// 返回一个包含读取句柄 (`ReadSignal`) 和写入句柄 (`WriteSignal`) 的元组。
///
/// # 参数
/// * `value` - Signal 的初始值。
///
/// # 泛型
/// * `T` - 存储在 Signal 中的数据类型，必须满足 `'static` 生命周期。
pub fn create_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    RUNTIME.with(|rt| {
        let id = rt.register_signal(value);
        (
            ReadSignal {
                id,
                marker: PhantomData,
            },
            WriteSignal {
                id,
                marker: PhantomData,
            },
        )
    })
}

/// 在不通过响应式系统追踪依赖的情况下运行一个闭包。
/// 这意味着在这个闭包内部读取 Signal 不会将当前的副作用注册为依赖。
///
/// # 参数
/// * `f` - 要执行的闭包。
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|rt| {
        // 暂时移除当前的 owner (Effect/Scope)，以避免追踪
        let prev_owner = *rt.current_owner.borrow();
        *rt.current_owner.borrow_mut() = None;
        let t = f();
        // 恢复之前的 owner
        *rt.current_owner.borrow_mut() = prev_owner;
        t
    })
}

/// 创建一个 Memo（派生信号）。
/// Memo 是一个计算属性，它依赖于其他 Signal，并且只有当其依赖发生变化且计算结果改变时，才会通知下游。
///
/// # 参数
/// * `f` - 计算函数，用于生成新的值。
///
/// # 泛型
/// * `T` - 计算结果的类型，需要实现 `Clone` 和 `PartialEq` 以支持变更检测。
pub fn create_memo<T, F>(f: F) -> ReadSignal<T>
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> T + 'static,
{
    // 初始计算，使用 untrack 避免在创建时建立不必要的外部依赖（视情况而定，这里主要是获取初始值）
    // 注意：通常 Memo 内部的首次运行也需要追踪依赖，但这里设计为复用 create_signal + create_effect。
    // 这里的 untrack 是为了避免 Memo 的初始值计算被外层 Effect 意外追踪（如果 create_memo 嵌套在 Effect 中）。
    // 但 Memo 内部的 Effect 必须追踪 f() 中的依赖。
    let initial_value = untrack(|| f());
    let (read, write) = create_signal(initial_value);

    create_effect(move || {
        let new_value = f();
        // 只有当新值与旧值不同时才更新 Signal，这提供了防止不必要更新的优化
        if let Some(old_value) = read.get_untracked()
            && new_value != old_value
        {
            write.set(new_value);
        }
    });
    read
}

impl<T: 'static + Clone> ReadSignal<T> {
    /// 获取 Signal 的当前值，并追踪依赖。
    /// 如果在 Effect 上下文中调用，该 Effect 会被注册为依赖。
    pub fn get(&self) -> Option<T> {
        RUNTIME.with(|rt| {
            rt.track_dependency(self.id);
            self.get_untracked_internal(rt)
        })
    }

    /// 获取 Signal 的当前值，但不追踪依赖。
    pub fn get_untracked(&self) -> Option<T> {
        RUNTIME.with(|rt| self.get_untracked_internal(rt))
    }

    /// 内部使用的获取值方法，不涉及依赖追踪逻辑。
    fn get_untracked_internal(&self, rt: &crate::reactivity::runtime::Runtime) -> Option<T> {
        let nodes = rt.nodes.borrow();
        if let Some(node) = nodes.get(self.id) {
            if let Some(any_val) = &node.value {
                if let Some(val) = any_val.downcast_ref::<T>() {
                    return Some(val.clone());
                } else {
                    crate::error!("ReadSignal Type Mismatch");
                    return None;
                }
            }
        }
        crate::error!("ReadSignal refers to dropped value");
        None
    }
}

/// `RwSignal` 是一个读写信号，同时包含了读取和写入的功能。
/// 它是 `ReadSignal` 和 `WriteSignal` 的组合封装。
pub struct RwSignal<T: 'static> {
    pub read: ReadSignal<T>,
    pub write: WriteSignal<T>,
}

impl<T> Clone for RwSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for RwSignal<T> {}

/// 创建一个 `RwSignal` (读写信号)。
pub fn create_rw_signal<T: 'static>(value: T) -> RwSignal<T> {
    let (read, write) = create_signal(value);
    RwSignal { read, write }
}

impl<T: Clone + 'static> RwSignal<T> {
    /// 创建一个新的 `RwSignal` 实例。
    pub fn new(value: T) -> Self {
        create_rw_signal(value)
    }

    /// 获取值并追踪依赖 (同 `ReadSignal::get`)。
    pub fn get(&self) -> Option<T> {
        self.read.get()
    }

    /// 获取值但不追踪依赖 (同 `ReadSignal::get_untracked`)。
    pub fn get_untracked(&self) -> Option<T> {
        self.read.get_untracked()
    }

    /// 设置新值 (同 `WriteSignal::set`)。
    pub fn set(&self, value: T) -> () {
        self.write.set(value)
    }

    /// 更新值 (同 `WriteSignal::update`)。
    pub fn update(&self, f: impl FnOnce(&mut T)) -> () {
        self.write.update(f)
    }

    /// 获取底层的 `ReadSignal`。
    pub fn read_signal(&self) -> ReadSignal<T> {
        self.read
    }

    /// 获取底层的 `WriteSignal`。
    pub fn write_signal(&self) -> WriteSignal<T> {
        self.write
    }
}

impl<T: 'static> WriteSignal<T> {
    /// 设置 Signal 的新值。
    /// 这将通知所有依赖此 Signal 的副作用进行更新。
    pub fn set(&self, new_value: T) -> () {
        self.update(|v| *v = new_value)
    }

    /// 使用闭包更新 Signal 的值。
    /// 允许就地修改内部值。
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        RUNTIME.with(|rt| {
            // 1. 更新值
            {
                let mut nodes = rt.nodes.borrow_mut();
                if let Some(node) = nodes.get_mut(self.id) {
                    if let Some(any_val) = &mut node.value {
                        if let Some(val) = any_val.downcast_mut::<T>() {
                            f(val);
                        } else {
                            crate::error!("WriteSignal Type Mismatch");
                            return;
                        }
                    } else {
                        // Signal value is missing?
                        crate::error!("WriteSignal refers to dropped value (no value)");
                        return;
                    }
                } else {
                    crate::error!("WriteSignal refers to dropped value (no node)");
                    return;
                }
            }

            // 2. 获取所有依赖此 Signal 的节点 ID
            let effects_to_run = rt.get_dependents(self.id);

            // 3. 运行 Effect
            for effect_id in effects_to_run {
                run_effect(effect_id);
            }
        })
    }
}

/// `Resource` 用于处理异步数据加载。
/// 它包含数据信号 (`data`)、加载状态信号 (`loading`) 和一个重新获取触发器。
pub struct Resource<T: 'static> {
    /// 存储异步获取的数据，初始为 `None`。
    pub data: ReadSignal<Option<T>>,
    /// 指示数据是否正在加载中。
    pub loading: ReadSignal<bool>,
    /// 用于手动触发重新加载的信号。
    trigger: WriteSignal<usize>,
}
impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Resource<T> {}

/// 创建一个资源 (`Resource`)，用于管理异步数据获取。
///
/// # 参数
/// * `source` - 一个闭包，返回用于获取数据的参数（如 ID 或 URL）。它是响应式的，当返回值变化时会自动重新获取数据。
/// * `fetcher` - 一个异步函数，接受 `source` 的返回值并获取数据。
pub fn create_resource<S, T, Fu>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> Fu + 'static,
) -> SinterResult<Resource<T>>
where
    S: PartialEq + Clone + 'static,
    T: Clone + 'static,
    Fu: Future<Output = T> + 'static,
{
    let (data, set_data) = create_signal(None);
    let (loading, set_loading) = create_signal(false);
    let (trigger, set_trigger) = create_signal(0);

    // 追踪资源所有者（通常是组件调用点）的生命周期。
    // 如果组件被卸载，我们不应该再更新状态。
    let alive = Rc::new(Cell::new(true));
    let alive_clone = alive.clone();
    on_cleanup(move || alive_clone.set(false));

    create_effect(move || {
        let source_val = source();
        // 追踪 trigger 以允许手动重新获取
        let _ = trigger.get();

        // 指示加载开始
        let suspense_ctx = use_suspense_context();
        if let Some(ctx) = &suspense_ctx {
            ctx.increment();
        }
        let _ = set_loading.set(true);

        // 启动异步任务
        let fut = fetcher(source_val);
        let suspense_ctx = suspense_ctx.clone();

        let alive = alive.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let res = fut.await;

            if alive.get() {
                // 仅当组件仍然存活时更新状态
                let _ = set_data.set(Some(res));
                let _ = set_loading.set(false);
            } else {
                crate::error!("Resource fetched but owner is dead, discarded");
            }

            // 指示加载完成
            if let Some(ctx) = &suspense_ctx {
                ctx.decrement();
            }
        });
    });

    Ok(Resource {
        data,
        loading,
        trigger: set_trigger,
    })
}

impl<T: Clone + 'static> Resource<T> {
    /// 获取资源数据。如果是 `None` 则表示尚未加载完成或初始状态。
    pub fn get(&self) -> Option<T> {
        self.data.get().flatten()
    }

    /// 检查资源是否正在加载。
    pub fn loading(&self) -> bool {
        self.loading.get().unwrap_or(false)
    }

    /// 手动触发重新获取数据。
    pub fn refetch(&self) {
        let _ = self.trigger.update(|n| *n = n.wrapping_add(1));
    }
}

// --- Context 上下文 API ---

/// 提供一个上下文值给当前组件树及其子孙组件。
/// 上下文基于类型 (`T`) 进行键控。
pub fn provide_context<T: 'static>(value: T) -> SinterResult<()> {
    RUNTIME.with(|rt| {
        if let Some(owner) = *rt.current_owner.borrow() {
            let mut nodes = rt.nodes.borrow_mut();
            if let Some(node) = nodes.get_mut(owner) {
                if node.context.is_none() {
                    node.context = Some(HashMap::new());
                }
                // unwrap exists now because we just checked/created it
                if let Some(ctx) = &mut node.context {
                    ctx.insert(TypeId::of::<T>(), Box::new(value));
                }
                Ok(())
            } else {
                Err(SinterError::Reactivity(
                    "provide_context owner not found".into(),
                ))
            }
        } else {
            Err(SinterError::Reactivity(
                "provide_context 被调用时没有 owner 作用域".into(),
            ))
        }
    })
}

/// 获取上下文值。
/// 会向上遍历组件树，直到找到对应类型的上下文。
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    RUNTIME.with(|rt| {
        let nodes = rt.nodes.borrow();
        let mut current_opt = *rt.current_owner.borrow();

        // 向上遍历树
        while let Some(current) = current_opt {
            if let Some(node) = nodes.get(current) {
                if let Some(ctx) = &node.context {
                    if let Some(val) = ctx.get(&TypeId::of::<T>()) {
                        return val.downcast_ref::<T>().cloned();
                    }
                }
                // 移动到父节点
                current_opt = node.parent;
            } else {
                current_opt = None;
            }
        }
        None
    })
}

// --- Effect 副作用 API ---

/// 创建一个副作用 (Effect)。
/// 副作用是一个并在依赖发生变化时自动重新运行的闭包。
/// `f` 闭包会被立即执行一次以进行依赖收集。
pub fn create_effect<F>(f: F)
where
    F: Fn() + 'static,
{
    let id = RUNTIME.with(|rt| {
        let id = rt.register_node(NodeType::Effect);
        let mut nodes = rt.nodes.borrow_mut();
        if let Some(node) = nodes.get_mut(id) {
            node.computation = Some(Rc::new(f));
        }
        id
    });
    run_effect(id);
}

// --- Scope 作用域 API ---

/// 创建一个新的响应式作用域 (Score)。
/// 作用域用于管理资源的生命周期（如 Effect、Signal 等）。
/// 当作用域被销毁时，其下的所有资源也会被清理。
pub fn create_scope<F>(f: F) -> NodeId
where
    F: FnOnce(),
{
    RUNTIME.with(|rt| {
        let id = rt.register_node(NodeType::Scope);

        let prev_owner = *rt.current_owner.borrow();
        *rt.current_owner.borrow_mut() = Some(id);
        let _ = f();
        *rt.current_owner.borrow_mut() = prev_owner;

        id
    })
}

/// 销毁指定的作用域或节点。
/// 这会清理该节点下的所有资源和子节点。
pub fn dispose(id: NodeId) {
    RUNTIME.with(|rt| {
        rt.dispose_node(id, true);
    });
}

/// 注册一个在当前作用域被清理时执行的回调函数。
/// 这对于释放非内存资源（如定时器、DOM 事件监听器等）非常有用。
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    RUNTIME.with(|rt| {
        if let Some(owner) = *rt.current_owner.borrow() {
            let mut nodes = rt.nodes.borrow_mut();
            if let Some(node) = nodes.get_mut(owner) {
                node.cleanups.push(Box::new(f));
            }
        }
    });
}

// --- Suspense 悬念/异步等待 API ---

/// `SuspenseContext` 用于在异步操作进行时管理挂起状态。
/// 它维护一个计数器，表示当前有多少个异步任务正在进行。
#[derive(Clone, Copy)]
pub struct SuspenseContext {
    pub count: ReadSignal<usize>,
    pub set_count: WriteSignal<usize>,
}

impl SuspenseContext {
    pub fn new() -> Self {
        let (count, set_count) = create_signal(0);
        Self { count, set_count }
    }

    /// 增加挂起的任务计数。
    pub fn increment(&self) {
        // 优化：Signal 更新是同步的。
        let _ = self.set_count.update(|c| *c += 1);
    }

    /// 减少挂起的任务计数。
    pub fn decrement(&self) {
        let _ = self.set_count.update(|c| {
            if *c > 0 {
                *c -= 1
            }
        });
    }
}

/// 获取当前的 `SuspenseContext`。
/// 通常由 `Suspense` 组件提供。
pub fn use_suspense_context() -> Option<SuspenseContext> {
    use_context::<SuspenseContext>()
}
