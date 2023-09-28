use ego_tree::{NodeId, NodeMut, NodeRef, Tree};

pub trait NodeMutAddons<T> {
	/// Appends a new child node by cloning the values of source and all its descendants. 
	fn append_cloned_tree(&mut self, source: &NodeRef<T>);
	/// Inserts a new sibling before this node by cloning the values of source and all its descendants. 
	fn insert_cloned_tree_before(&mut self, source: &NodeRef<T>);
	/// Inserts clones of source's children and all their descendants as siblings before this node
	fn insert_cloned_descendants_before(&mut self, source: &NodeRef<T>);
}

impl<'a, T: Clone + 'a> NodeMutAddons<T> for NodeMut<'a, T> {
	// There's probably a more performant way to merge these tree branches together, but it works for now.
    fn append_cloned_tree(&mut self, source: &NodeRef<T>) {
		let mut new_child = self.append(source.value().clone());
		for source_child in source.children() {
			new_child.append_cloned_tree(&source_child);
		}
    }
	fn insert_cloned_tree_before(&mut self, source: &NodeRef<T>) {
		let mut new_child = self.insert_before(source.value().clone());
		for source_child in source.children() {
			new_child.append_cloned_tree(&source_child);
		}
	}
	fn insert_cloned_descendants_before(&mut self, source: &NodeRef<T>) {
		for source_child in source.children() {
			let mut new_sibling = self.insert_before(source_child.value().clone());
			for source_grandchild in source_child.children() {
				new_sibling.append_cloned_tree(&source_grandchild);
			}
		}
	}
}
