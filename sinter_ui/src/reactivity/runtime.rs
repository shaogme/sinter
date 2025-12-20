use slotmap::{SecondaryMap, SlotMap, new_key_type};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// --- 基础类型定义 ---

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum NodeType {
    Signal,
    Effect,
    Scope,
}

// --- 响应式系统运行时 ---

pub(crate) struct Runtime {
    // 主 Map：负责分配 ID，管理生命周期 (Existence) 和元数据
    pub(crate) nodes: RefCell<SlotMap<NodeId, NodeType>>,

    // 数据层：Signal 的值
    pub(crate) values: RefCell<SecondaryMap<NodeId, Box<dyn Any>>>,

    // 计算层：Effect/Scope 的执行逻辑
    pub(crate) computations: RefCell<SecondaryMap<NodeId, Rc<dyn Fn() -> ()>>>,

    // 图结构层：分离 Subscribers 和 Dependencies 以允许细粒度借用
    pub(crate) subscribers: RefCell<SecondaryMap<NodeId, HashSet<NodeId>>>, // Signal -> Effects
    pub(crate) dependencies: RefCell<SecondaryMap<NodeId, HashSet<NodeId>>>, // Effect -> Signals

    // 层级关系层
    pub(crate) children: RefCell<SecondaryMap<NodeId, HashSet<NodeId>>>,
    pub(crate) parents: RefCell<SecondaryMap<NodeId, NodeId>>,
    pub(crate) cleanups: RefCell<SecondaryMap<NodeId, Vec<Box<dyn FnOnce()>>>>,

    // 全局状态
    pub(crate) current_owner: RefCell<Option<NodeId>>,
    // Contexts per node
    pub(crate) node_contexts: RefCell<SecondaryMap<NodeId, HashMap<TypeId, Box<dyn Any>>>>,
}

thread_local! {
    pub(crate) static RUNTIME: Runtime = Runtime::new();
}

impl Runtime {
    fn new() -> Self {
        Self {
            nodes: RefCell::new(SlotMap::with_key()),
            values: RefCell::new(SecondaryMap::new()),
            computations: RefCell::new(SecondaryMap::new()),
            subscribers: RefCell::new(SecondaryMap::new()),
            dependencies: RefCell::new(SecondaryMap::new()),
            children: RefCell::new(SecondaryMap::new()),
            parents: RefCell::new(SecondaryMap::new()),
            cleanups: RefCell::new(SecondaryMap::new()),
            current_owner: RefCell::new(None),
            node_contexts: RefCell::new(SecondaryMap::new()),
        }
    }

    // --- Core Operations ---

    pub(crate) fn register_node(&self, kind: NodeType) -> NodeId {
        // 1. 分配 ID
        let id = self.nodes.borrow_mut().insert(kind);

        // 2. 初始化必要的辅助结构
        match kind {
            NodeType::Signal => {
                self.subscribers.borrow_mut().insert(id, HashSet::new());
            }
            NodeType::Effect => {
                self.dependencies.borrow_mut().insert(id, HashSet::new());
                self.children.borrow_mut().insert(id, HashSet::new());
                self.cleanups.borrow_mut().insert(id, Vec::new());
            }
            NodeType::Scope => {
                self.children.borrow_mut().insert(id, HashSet::new());
                self.cleanups.borrow_mut().insert(id, Vec::new());
            }
        }

        // 3. 建立父子关系
        if let Some(parent) = *self.current_owner.borrow() {
            self.parents.borrow_mut().insert(id, parent);

            let mut children_map = self.children.borrow_mut();
            if let Some(siblings) = children_map.get_mut(parent) {
                siblings.insert(id);
            }
        }

        id
    }

    pub(crate) fn register_signal<T: 'static>(&self, value: T) -> NodeId {
        let id = self.register_node(NodeType::Signal);
        self.values.borrow_mut().insert(id, Box::new(value));
        id
    }

    pub(crate) fn track_dependency(&self, signal_id: NodeId) {
        if let Some(owner) = *self.current_owner.borrow() {
            let is_effect = {
                let nodes = self.nodes.borrow();
                matches!(nodes.get(owner), Some(NodeType::Effect))
            };

            if is_effect {
                {
                    let mut deps_map = self.dependencies.borrow_mut();
                    if let Some(deps) = deps_map.get_mut(owner) {
                        deps.insert(signal_id);
                    }
                }

                {
                    let mut subs_map = self.subscribers.borrow_mut();
                    if let Some(subs) = subs_map.get_mut(signal_id) {
                        subs.insert(owner);
                    }
                }
            }
        }
    }

    pub(crate) fn get_dependents(&self, signal_id: NodeId) -> Vec<NodeId> {
        let subs_map = self.subscribers.borrow();
        if let Some(subs) = subs_map.get(signal_id) {
            subs.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn clean_node(&self, id: NodeId) {
        let children = {
            let mut children_map = self.children.borrow_mut();
            if let Some(children) = children_map.get_mut(id) {
                children.drain().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for child in children {
            self.dispose_node(child, false);
        }

        let cleanups = {
            let mut cleanups_map = self.cleanups.borrow_mut();
            if let Some(vec) = cleanups_map.get_mut(id) {
                std::mem::take(vec)
            } else {
                Vec::new()
            }
        };

        for cleanup in cleanups {
            cleanup();
        }

        let dependencies = {
            let mut deps_map = self.dependencies.borrow_mut();
            if let Some(deps) = deps_map.get_mut(id) {
                deps.drain().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        if !dependencies.is_empty() {
            let mut subs_map = self.subscribers.borrow_mut();
            for signal_id in dependencies {
                if let Some(subs) = subs_map.get_mut(signal_id) {
                    subs.remove(&id);
                }
            }
        }
    }

    pub(crate) fn dispose_node(&self, id: NodeId, remove_from_parent: bool) {
        self.clean_node(id);

        if remove_from_parent {
            let parent = self.parents.borrow_mut().remove(id);
            if let Some(parent_id) = parent {
                let mut children_map = self.children.borrow_mut();
                if let Some(siblings) = children_map.get_mut(parent_id) {
                    siblings.remove(&id);
                }
            }
        } else {
            self.parents.borrow_mut().remove(id);
        }

        self.values.borrow_mut().remove(id);
        self.computations.borrow_mut().remove(id);
        self.subscribers.borrow_mut().remove(id);
        self.dependencies.borrow_mut().remove(id);
        self.children.borrow_mut().remove(id);
        self.cleanups.borrow_mut().remove(id);
        self.node_contexts.borrow_mut().remove(id);

        self.nodes.borrow_mut().remove(id);
    }
}

pub(crate) fn run_effect(effect_id: NodeId) -> () {
    RUNTIME.with(|rt| {
        rt.clean_node(effect_id);

        let computation = {
            let comps = rt.computations.borrow();
            comps.get(effect_id).cloned()
        };

        if let Some(f) = computation {
            let prev_owner = *rt.current_owner.borrow();
            *rt.current_owner.borrow_mut() = Some(effect_id);

            f();

            *rt.current_owner.borrow_mut() = prev_owner;
        }
    })
}
