use crate::dom::tag::div;
use crate::dom::{Element, View};
use crate::reactivity::{NodeId, create_effect, create_scope, dispose};
use crate::{SinterError, SinterResult};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use web_sys::Node;

// For 组件保持不变
pub struct For<ItemsFn, Item, Items, KeyFn, Key, MapFn, V> {
    items: Rc<ItemsFn>,
    key: Rc<KeyFn>,
    map: Rc<MapFn>,
    _marker: std::marker::PhantomData<(Item, Items, Key, V)>,
}

impl<ItemsFn, Item, Items, KeyFn, Key, MapFn, V> For<ItemsFn, Item, Items, KeyFn, Key, MapFn, V>
where
    ItemsFn: Fn() -> SinterResult<Items> + 'static,
    Items: IntoIterator<Item = Item>,
    KeyFn: Fn(&Item) -> Key + 'static,
    MapFn: Fn(Item) -> V + 'static,
    V: View,
    Item: 'static,
{
    pub fn new(items: ItemsFn, key: KeyFn, map: MapFn) -> Self {
        Self {
            items: Rc::new(items),
            key: Rc::new(key),
            map: Rc::new(map),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<ItemsFn, Item, Items, KeyFn, Key, MapFn, V> View
    for For<ItemsFn, Item, Items, KeyFn, Key, MapFn, V>
where
    ItemsFn: Fn() -> SinterResult<Items> + 'static,
    Items: IntoIterator<Item = Item>,
    KeyFn: Fn(&Item) -> Key + 'static,
    Key: std::hash::Hash + Eq + Clone + 'static,
    MapFn: Fn(Item) -> V + 'static,
    V: View,
    Item: 'static,
{
    fn mount(self, parent: &Node) {
        let container = div().style("display: contents");

        container.clone().mount(parent);
        let root = container.dom_element;

        let items_fn = self.items;
        let key_fn = self.key;
        let map_fn = self.map;

        // 修改：存储 Tuple (Element, ScopeId)
        let active_rows = Rc::new(RefCell::new(HashMap::<Key, (Element, NodeId)>::new()));

        create_effect(move || {
            let mut rows_map = active_rows.borrow_mut();

            let items = match (items_fn)() {
                Ok(items) => items,
                Err(e) => {
                    crate::error::handle_error(e);
                    return;
                }
            };

            let mut new_keys = HashSet::new();
            let mut new_rows_order = Vec::new();

            for item in items {
                let key = (key_fn)(&item);
                new_keys.insert(key.clone());

                let (wrapper, id) = if let Some(existing) = rows_map.get(&key) {
                    existing.clone()
                } else {
                    let wrapper = div().style("display: contents");

                    let parent = wrapper.dom_element.clone();
                    // 这里克隆 map_fn 引用，因为需要在闭包中使用
                    let map_fn = map_fn.clone();

                    // 创建独立 Scope，防止 create_effect 重运行时清理掉该行的事件监听器
                    let scope_id = create_scope(move || {
                        let view = (map_fn)(item);
                        view.mount(&parent);
                    });

                    (wrapper, scope_id)
                };

                new_rows_order.push((key, wrapper, id));
            }

            rows_map.retain(|k, v| {
                if !new_keys.contains(k) {
                    v.0.dom_element.remove();
                    // 销毁 Scope，释放相关闭包内存
                    dispose(v.1);
                    false
                } else {
                    true
                }
            });

            let mut cursor = root.first_child();
            for (key, wrapper, id) in new_rows_order {
                let node = &wrapper.dom_element;
                let is_in_place = if let Some(ref current) = cursor {
                    current.is_same_node(Some(node))
                } else {
                    false
                };

                if is_in_place {
                    cursor = cursor.and_then(|c| c.next_sibling());
                } else {
                    if let Err(e) = root
                        .insert_before(node, cursor.as_ref())
                        .map_err(SinterError::from)
                    {
                        crate::error::handle_error(e);
                    }
                }
                rows_map.insert(key, (wrapper, id));
            }
        });
    }
}
