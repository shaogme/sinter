pub mod runtime;

pub use runtime::NodeId;

use slotmap::Key;
use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::reactivity::runtime::{NodeType, RUNTIME, run_effect};
use crate::{SinterError, SinterResult};

// --- Signal API ---

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

pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|rt| {
        let prev_owner = *rt.current_owner.borrow();
        *rt.current_owner.borrow_mut() = None;
        let t = f();
        *rt.current_owner.borrow_mut() = prev_owner;
        t
    })
}

pub fn create_memo<T, F>(f: F) -> ReadSignal<T>
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> T + 'static,
{
    let initial_value = untrack(|| f());
    let (read, write) = create_signal(initial_value);

    create_effect(move || {
        let new_value = f();
        if let Some(old_value) = read.get_untracked()
            && new_value != old_value
        {
            write.set(new_value);
        }
    });
    read
}

impl<T: 'static + Clone> ReadSignal<T> {
    pub fn get(&self) -> Option<T> {
        RUNTIME.with(|rt| {
            rt.track_dependency(self.id);
            self.get_untracked_internal(rt)
        })
    }

    pub fn get_untracked(&self) -> Option<T> {
        RUNTIME.with(|rt| self.get_untracked_internal(rt))
    }

    fn get_untracked_internal(&self, rt: &crate::reactivity::runtime::Runtime) -> Option<T> {
        let values = rt.values.borrow();
        if let Some(any_val) = values.get(self.id) {
            if let Some(val) = any_val.downcast_ref::<T>() {
                Some(val.clone())
            } else {
                crate::error!("ReadSignal Type Mismatch");
                None
            }
        } else {
            crate::error!("ReadSignal refers to dropped value");
            None
        }
    }
}

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

pub fn create_rw_signal<T: 'static>(value: T) -> RwSignal<T> {
    let (read, write) = create_signal(value);
    RwSignal { read, write }
}

impl<T: Clone + 'static> RwSignal<T> {
    pub fn new(value: T) -> Self {
        create_rw_signal(value)
    }

    pub fn get(&self) -> Option<T> {
        self.read.get()
    }

    pub fn get_untracked(&self) -> Option<T> {
        self.read.get_untracked()
    }

    pub fn set(&self, value: T) -> () {
        self.write.set(value)
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) -> () {
        self.write.update(f)
    }

    pub fn read_signal(&self) -> ReadSignal<T> {
        self.read
    }

    pub fn write_signal(&self) -> WriteSignal<T> {
        self.write
    }
}

impl<T: 'static> WriteSignal<T> {
    pub fn set(&self, new_value: T) -> () {
        self.update(|v| *v = new_value)
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        RUNTIME.with(|rt| {
            // 1. Update Value
            {
                let mut values = rt.values.borrow_mut();
                if let Some(any_val) = values.get_mut(self.id) {
                    if let Some(val) = any_val.downcast_mut::<T>() {
                        f(val);
                    } else {
                        crate::error!("WriteSignal Type Mismatch");
                        return;
                    }
                } else {
                    crate::error!("WriteSignal refers to dropped value");
                    return;
                }
            }

            // 2. Trigger Effects
            let effects_to_run = rt.get_dependents(self.id);

            // 3. 运行 Effects
            for effect_id in effects_to_run {
                run_effect(effect_id);
            }
        })
    }
}

pub struct Resource<T: 'static> {
    pub data: ReadSignal<Option<T>>,
    pub loading: ReadSignal<bool>,
    trigger: WriteSignal<usize>,
}
impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Resource<T> {}

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

    // Track liveness of the resource owner (the component call site)
    let alive = Rc::new(Cell::new(true));
    let alive_clone = alive.clone();
    on_cleanup(move || alive_clone.set(false));

    create_effect(move || {
        let source_val = source();
        // Track trigger to allow manual refetch
        let _ = trigger.get();

        // Indicate loading start
        let suspense_ctx = use_suspense_context();
        if let Some(ctx) = &suspense_ctx {
            ctx.increment();
        }
        let _ = set_loading.set(true);

        // Spawn async task
        let fut = fetcher(source_val);
        let suspense_ctx = suspense_ctx.clone();

        let alive = alive.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let res = fut.await;

            if alive.get() {
                // Update state only if alive
                let _ = set_data.set(Some(res));
                let _ = set_loading.set(false);
            } else {
                crate::error!("Resource fetched but owner is dead, discarded");
            }

            // Indicate loading finished
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
    pub fn get(&self) -> Option<T> {
        self.data.get().flatten()
    }

    pub fn loading(&self) -> bool {
        self.loading.get().unwrap_or(false)
    }

    pub fn refetch(&self) {
        let _ = self.trigger.update(|n| *n = n.wrapping_add(1));
    }
}

// --- Context API ---

pub fn provide_context<T: 'static>(value: T) -> SinterResult<()> {
    RUNTIME.with(|rt| {
        if let Some(owner) = *rt.current_owner.borrow() {
            let mut contexts_map = rt.node_contexts.borrow_mut();
            // Ensure we have a map for this owner
            if !contexts_map.contains_key(owner) {
                contexts_map.insert(owner, HashMap::new());
            }
            if let Some(ctx) = contexts_map.get_mut(owner) {
                ctx.insert(TypeId::of::<T>(), Box::new(value));
            }
            Ok(())
        } else {
            Err(SinterError::Reactivity(
                "provide_context called without an owner scope".into(),
            ))
        }
    })
}

pub fn use_context<T: Clone + 'static>() -> Option<T> {
    RUNTIME.with(|rt| {
        let parents = rt.parents.borrow();
        let contexts_map = rt.node_contexts.borrow();

        let mut current_opt = *rt.current_owner.borrow();

        // Traverse up the tree
        while let Some(current) = current_opt {
            if let Some(ctx_map) = contexts_map.get(current) {
                if let Some(val) = ctx_map.get(&TypeId::of::<T>()) {
                    return val.downcast_ref::<T>().cloned();
                }
            }

            // Move to parent
            if !current.is_null() {
                current_opt = parents.get(current).cloned();
            } else {
                current_opt = None;
            }
        }
        None
    })
}

// --- Effect API ---

pub fn create_effect<F>(f: F)
where
    F: Fn() + 'static,
{
    let id = RUNTIME.with(|rt| {
        let id = rt.register_node(NodeType::Effect);
        rt.computations.borrow_mut().insert(id, Rc::new(f));
        id
    });
    run_effect(id);
}

// --- Scope API ---

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

pub fn dispose(id: NodeId) {
    RUNTIME.with(|rt| {
        rt.dispose_node(id, true);
    });
}

pub fn on_cleanup(f: impl FnOnce() + 'static) {
    RUNTIME.with(|rt| {
        if let Some(owner) = *rt.current_owner.borrow() {
            let mut cleanups_map = rt.cleanups.borrow_mut();
            if let Some(vec) = cleanups_map.get_mut(owner) {
                vec.push(Box::new(f));
            }
        }
    });
}

// --- Suspense API ---

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

    pub fn increment(&self) {
        // 优化：批量更新？Signal 更新是同步的。
        let _ = self.set_count.update(|c| *c += 1);
    }

    pub fn decrement(&self) {
        let _ = self.set_count.update(|c| {
            if *c > 0 {
                *c -= 1
            }
        });
    }
}

pub fn use_suspense_context() -> Option<SuspenseContext> {
    use_context::<SuspenseContext>()
}
