// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_TList.gxx (the edits: Unlink, LinkBefore, LinkAfter, FrontAdd, BackAdd, Permute and Loop) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edits of `MAT_TList`: they relink the nodes around the current one.
use super::{MatTList, MatTListItem, MatTListNode, node};
use rustycad_tkernel::standard::{Failure, Handle};

type Node<I> = Handle<MatTListNode<I>>;

impl<I: MatTListItem> MatTList<I> {
    /// `Unlink()`: removes the current node from the chain (it stays current, so
    /// `Next` reaches the node after it) and decrements the count and the index;
    /// `list.unlink()?`.
    pub fn unlink(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        let current: Node<I> = state.current.clone();
        let current_node = node(&current, "Unlink")?;
        let (previous, next) = (current_node.previous(), current_node.next());
        if state.index != 0 {
            if let Some(next_node) = next.get() {
                next_node.set_previous(&previous);
            }
            if let Some(previous_node) = previous.get() {
                previous_node.set_next(&next);
            }
            if state.index == 1 {
                state.first = next;
            } else if state.index == state.count {
                state.last = previous;
            }
        }
        state.count -= 1;
        state.index -= 1;
        Ok(())
    }
    /// `LinkBefore(anitem)`: inserts `anitem` before the current node;
    /// `list.link_before(&edge)?`.
    pub fn link_before(&self, anitem: &Handle<I>) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.count += 1;
        if state.index != 0 {
            state.index += 1;
        }
        let new_node: Node<I> = Handle::new(MatTListNode::with_item(anitem));
        let current: Node<I> = state.current.clone();
        let current_node = node(&current, "LinkBefore")?;
        let previous: Node<I> = current_node.previous();
        if let Some(previous_node) = previous.get() {
            previous_node.set_next(&new_node);
            new_node.set_previous(&previous);
        }
        if state.index == 2 {
            state.first = new_node.clone();
        }
        current_node.set_previous(&new_node);
        new_node.set_next(&current);
        Ok(())
    }
    /// `LinkAfter(anitem)`: inserts `anitem` after the current node;
    /// `list.link_after(&edge)?`.
    pub fn link_after(&self, anitem: &Handle<I>) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.count += 1;
        let new_node: Node<I> = Handle::new(MatTListNode::with_item(anitem));
        let current: Node<I> = state.current.clone();
        let current_node = node(&current, "LinkAfter")?;
        let next: Node<I> = current_node.next();
        if let Some(next_node) = next.get() {
            next_node.set_previous(&new_node);
            new_node.set_next(&next);
        }
        if state.index + 1 == state.count {
            state.last = new_node.clone();
        }
        current_node.set_next(&new_node);
        new_node.set_previous(&current);
        Ok(())
    }
    /// `FrontAdd(anitem)`: inserts `anitem` before the first node;
    /// `list.front_add(&edge)`.
    pub fn front_add(&self, anitem: &Handle<I>) {
        let mut state = self.state.write();
        state.count += 1;
        if state.index != 0 {
            state.index += 1;
        }
        let new_node: Node<I> = Handle::new(MatTListNode::with_item(anitem));
        match state.first.get() {
            Some(first) => {
                first.set_previous(&new_node);
                new_node.set_next(&state.first);
            }
            None => state.last = new_node.clone(),
        }
        state.first = new_node;
    }
    /// `BackAdd(anitem)`: appends `anitem` after the last node; `list.back_add(&edge)`.
    pub fn back_add(&self, anitem: &Handle<I>) {
        let mut state = self.state.write();
        state.count += 1;
        let new_node: Node<I> = Handle::new(MatTListNode::with_item(anitem));
        match state.last.get() {
            Some(last) => {
                last.set_next(&new_node);
                new_node.set_previous(&state.last);
            }
            None => state.first = new_node.clone(),
        }
        state.last = new_node;
    }
    /// `Permute()`: exchanges the current node and the next one (the current node
    /// moves one place on, and so does the index); `list.permute()?`.
    pub fn permute(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        let current: Node<I> = state.current.clone();
        let current_node = node(&current, "Permute")?;
        let previous: Node<I> = current_node.previous();
        let next: Node<I> = current_node.next();
        let next_node = node(&next, "Permute")?;
        let nextnext: Node<I> = next_node.next();
        match previous.get() {
            Some(previous_node) => {
                previous_node.set_next(&next);
                next_node.set_previous(&previous);
            }
            None => next_node.set_previous(&Handle::default()),
        }
        next_node.set_next(&current);
        current_node.set_previous(&next);
        match nextnext.get() {
            Some(nextnext_node) => {
                current_node.set_next(&nextnext);
                nextnext_node.set_previous(&current);
            }
            None => current_node.set_next(&Handle::default()),
        }
        if state.first == current {
            state.first = next.clone();
        }
        if state.last == next {
            state.last = current.clone();
        }
        state.index += 1;
        Ok(())
    }
    /// `Loop()`: links the last node to the first one, so the list becomes a ring;
    /// `list.close_loop()?`.
    pub fn close_loop(&self) -> Result<(), Failure> {
        let state = self.state.read();
        node(&state.last, "Loop")?.set_next(&state.first);
        node(&state.first, "Loop")?.set_previous(&state.last);
        Ok(())
    }
}
