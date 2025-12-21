use slotmap::{SlotMap, new_key_type};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// --- 基础类型定义 ---

new_key_type! {
    /// 响应式节点的唯一标识符。
    pub struct NodeId;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum NodeType {
    Signal,
    Effect,
    Scope,
}

/// 响应式节点结构体。
/// 为了提高性能和内存紧凑性，我们将所有节点相关的数据聚合到这一个结构体中 (SoA -> AoS 转换)。
pub(crate) struct Node {
    pub(crate) kind: NodeType,
    /// 仅 Signal 节点使用：存储 Signal 的值。
    pub(crate) value: Option<Box<dyn Any>>,
    /// 仅 Effect 节点使用：存储副作用的计算闭包。
    pub(crate) computation: Option<Rc<dyn Fn() -> ()>>,
    /// 信号的订阅者列表 (Signal -> Effects)。使用 Vec 替代 HashSet 以减少内存开销。
    pub(crate) subscribers: Vec<NodeId>,
    /// 副作用的依赖列表 (Effect -> Signals)。使用 Vec 替代 HashSet 以减少内存开销。
    pub(crate) dependencies: Vec<NodeId>,
    /// 子节点列表 (Scope/Effect -> Recursive)。用于生命周期管理。
    pub(crate) children: Vec<NodeId>,
    /// 父节点 ID。
    pub(crate) parent: Option<NodeId>,
    /// 清理回调函数列表。
    pub(crate) cleanups: Vec<Box<dyn FnOnce()>>,
    /// 上下文存储 (Context)。
    pub(crate) context: Option<HashMap<TypeId, Box<dyn Any>>>,
}

impl Node {
    fn new(kind: NodeType) -> Self {
        Self {
            kind,
            value: None,
            computation: None,
            subscribers: Vec::new(),
            dependencies: Vec::new(),
            children: Vec::new(),
            parent: None,
            cleanups: Vec::new(),
            context: None,
        }
    }
}

// --- 响应式系统运行时 ---

pub(crate) struct Runtime {
    /// 存储所有活动节点的 SlotMap。
    /// 使用 RefCell 提供内部可变性。
    pub(crate) nodes: RefCell<SlotMap<NodeId, Node>>,
    /// 当前正在运行的 Effect 或 Scope 的 ID (Owner)。
    pub(crate) current_owner: RefCell<Option<NodeId>>,
}

thread_local! {
    /// 线程局部的 Runtime 实例。
    pub(crate) static RUNTIME: Runtime = Runtime::new();
}

impl Runtime {
    fn new() -> Self {
        Self {
            nodes: RefCell::new(SlotMap::with_key()),
            current_owner: RefCell::new(None),
        }
    }

    // --- 核心操作 ---

    /// 注册一个新的节点到运行时系统中。
    pub(crate) fn register_node(&self, kind: NodeType) -> NodeId {
        let mut nodes = self.nodes.borrow_mut();
        self.register_node_internal(&mut nodes, kind)
    }

    /// 内部辅助函数：在已持有锁的情况下注册节点。
    fn register_node_internal(&self, nodes: &mut SlotMap<NodeId, Node>, kind: NodeType) -> NodeId {
        let parent = *self.current_owner.borrow();
        let mut node = Node::new(kind);
        node.parent = parent;

        let id = nodes.insert(node);

        if let Some(parent_id) = parent {
            if let Some(parent_node) = nodes.get_mut(parent_id) {
                parent_node.children.push(id);
            }
        }
        id
    }

    /// 注册一个新的 Signal。
    pub(crate) fn register_signal<T: 'static>(&self, value: T) -> NodeId {
        let mut nodes = self.nodes.borrow_mut();
        // 使用内部方法避免二次 borrow，直接在一个锁范围内完成创建和赋值
        let id = self.register_node_internal(&mut nodes, NodeType::Signal);

        if let Some(node) = nodes.get_mut(id) {
            node.value = Some(Box::new(value));
        }
        id
    }

    /// 追踪依赖关系。
    /// 当一个 Signal 被读取时调用，将其添加到当前运行的 Effect 的依赖列表中。
    pub(crate) fn track_dependency(&self, signal_id: NodeId) {
        if let Some(owner) = *self.current_owner.borrow() {
            let mut nodes = self.nodes.borrow_mut();

            // 显式处理自依赖情况
            if owner == signal_id {
                return;
            }

            // 使用 get_disjoint_mut 安全地同时借用两个可变引用
            if let Some([owner_node, signal_node]) = nodes.get_disjoint_mut([owner, signal_id]) {
                if owner_node.kind == NodeType::Effect {
                    // 优化：使用 Vec 代替 HashSet。
                    // 插入前检查是否存在以保持 Set 语义。
                    if !owner_node.dependencies.contains(&signal_id) {
                        owner_node.dependencies.push(signal_id);
                    }
                    if !signal_node.subscribers.contains(&owner) {
                        signal_node.subscribers.push(owner);
                    }
                }
            }
        }
    }

    /// 获取 Signal 的所有依赖者（订阅者）。
    pub(crate) fn get_dependents(&self, signal_id: NodeId) -> Vec<NodeId> {
        let nodes = self.nodes.borrow();
        if let Some(node) = nodes.get(signal_id) {
            // 返回订阅者列表的克隆。
            node.subscribers.clone()
        } else {
            Vec::new()
        }
    }

    /// 清理节点。
    /// 这包括递归清理子节点、运行清理回调以及解除依赖关系。
    pub(crate) fn clean_node(&self, id: NodeId) {
        let (children, cleanups, dependencies) = {
            let mut nodes = self.nodes.borrow_mut();
            if let Some(node) = nodes.get_mut(id) {
                (
                    std::mem::take(&mut node.children),
                    std::mem::take(&mut node.cleanups),
                    std::mem::take(&mut node.dependencies),
                )
            } else {
                return;
            }
        };

        self.run_cleanups(id, children, cleanups, dependencies);
    }

    /// 执行清理逻辑（独立出来的部分，以便复用）
    fn run_cleanups(
        &self,
        self_id: NodeId,
        children: Vec<NodeId>,
        cleanups: Vec<Box<dyn FnOnce()>>,
        dependencies: Vec<NodeId>,
    ) {
        // 1. 递归销毁子节点
        for child in children {
            self.dispose_node(child, false);
        }

        // 2. 运行清理回调
        for cleanup in cleanups {
            cleanup();
        }

        // 3. 从依赖项的订阅者列表中移除自身
        if !dependencies.is_empty() {
            let mut nodes = self.nodes.borrow_mut();
            for signal_id in dependencies {
                if let Some(signal_node) = nodes.get_mut(signal_id) {
                    if let Some(idx) = signal_node.subscribers.iter().position(|&x| x == self_id) {
                        signal_node.subscribers.swap_remove(idx);
                    }
                }
            }
        }
    }

    /// 销毁节点。
    /// 除了清理节点内容外，还会将其从 Runtime 中完全移除。
    pub(crate) fn dispose_node(&self, id: NodeId, remove_from_parent: bool) {
        self.clean_node(id);

        let mut nodes = self.nodes.borrow_mut();

        if remove_from_parent {
            // 从父节点的子节点列表中移除自己
            let parent_id = nodes.get(id).and_then(|n| n.parent);
            if let Some(parent) = parent_id {
                if let Some(parent_node) = nodes.get_mut(parent) {
                    if let Some(idx) = parent_node.children.iter().position(|&x| x == id) {
                        parent_node.children.swap_remove(idx);
                    }
                }
            }
        }

        nodes.remove(id);
    }
}

/// 运行一个 Effect。
/// 这会清理 Effect 之前的依赖，然后重新执行计算闭包并收集新的依赖。
pub(crate) fn run_effect(effect_id: NodeId) -> () {
    RUNTIME.with(|rt| {
        // 优化：在一次 borrow_mut 中同时获取计算闭包和需要清理的数据
        // 这减少了 RefCell 的借用开销，并利用了状态聚合的优势。
        let (computation, children, cleanups, dependencies) = {
            let mut nodes = rt.nodes.borrow_mut();
            if let Some(node) = nodes.get_mut(effect_id) {
                (
                    node.computation.clone(),
                    std::mem::take(&mut node.children),
                    std::mem::take(&mut node.cleanups),
                    std::mem::take(&mut node.dependencies),
                )
            } else {
                return;
            }
        };

        // 执行清理逻辑（不持有锁）
        rt.run_cleanups(effect_id, children, cleanups, dependencies);

        if let Some(f) = computation {
            let prev_owner = *rt.current_owner.borrow();
            *rt.current_owner.borrow_mut() = Some(effect_id);

            f();

            *rt.current_owner.borrow_mut() = prev_owner;
        }
    })
}
