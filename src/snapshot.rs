use std::{
    any::{Any, TypeId},
    fmt,
};

use crate::registry::Registry;

pub struct Element {
    pub type_id: TypeId,
    pub props: Box<dyn Any>,
    pub children: Vec<ElementId>,
}

impl Element {
    pub fn new<T: Any, P: Any>(props: P) -> Element {
        Element {
            type_id: TypeId::of::<T>(),
            props: Box::new(props),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(u32);

pub struct Snapshot {
    pub tree: Vec<Element>,
    pub roots: Vec<ElementId>,
    pub stack: Vec<ElementId>,
    registry: Registry,
}

impl Snapshot {
    pub fn new(registry: Registry) -> Self {
        Self {
            tree: Vec::new(),
            roots: Vec::new(),
            stack: Vec::new(),
            registry,
        }
    }

    pub fn clear(&mut self) {
        self.tree.clear();
        self.roots.clear();
        self.stack.clear();
    }

    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.tree.get(id.0 as usize)
    }

    pub(crate) fn insert(&mut self, element: Element) -> ElementId {
        let id = ElementId(self.tree.len() as u32);

        if let Some(top) = self.stack.last() {
            let top_element = &mut self.tree[top.0 as usize];
            top_element.children.push(id);
        } else {
            self.roots.push(id);
        }

        self.tree.push(element);

        id
    }

    pub(crate) fn push(&mut self, element: Element) -> ElementId {
        let id = self.insert(element);
        self.stack.push(id);
        id
    }

    pub(crate) fn pop(&mut self, id: ElementId) {
        match self.stack.pop() {
            Some(old_top) => {
                assert!(id == old_top, "Snapshot::pop popped the wrong element!");
            }
            None => {
                panic!("Cannot pop when there are no elements on the stack.");
            }
        }
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("roots", &self.roots)
            .field("tree", &ViewTree(self))
            .finish()
    }
}

struct ViewTree<'a>(&'a Snapshot);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dom = &self.0;
        let iter = dom.tree.iter().enumerate().map(|(index, element)| {
            let id = element.type_id;

            let debug = match dom.registry.get_by_id(id) {
                Some(component_impl) => (component_impl.debug_props)(element.props.as_ref()),
                None => &"(could not find debug impl)",
            };

            let children: Vec<_> = element.children.iter().collect();

            format!("{index:?}: {debug:?}, children: {children:?}")
        });

        f.debug_list().entries(iter).finish()
    }
}
