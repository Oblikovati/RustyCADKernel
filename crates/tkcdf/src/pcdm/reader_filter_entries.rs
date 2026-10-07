//! The entry filter and the label-tree iteration of `PCDM_ReaderFilter`.
use super::{FilterState, PcdmReaderFilter, TagNode};
use rustycad_tkernel::tcollection::AsciiString;

impl PcdmReaderFilter {
    /// `IsPassed(const TCollection_AsciiString& theEntry)`: true when the entry is one of the
    /// sub-trees or below one (`"0:10"` does not match `"0:1"`), or when no sub-tree is set.
    pub fn is_passed_entry(&self, entry: &AsciiString) -> bool {
        let state = self.state.read();
        if state.sub_trees.is_empty() {
            return true;
        }
        state
            .sub_trees
            .iter()
            .any(|tree| entry_in_tree(entry, tree))
    }
    /// `IsSubPassed(const TCollection_AsciiString& theEntry)`: true when a sub-tree lies below
    /// the entry; the root entry (length 2, `"0:"`) always passes.
    pub fn is_sub_passed_entry(&self, entry: &AsciiString) -> bool {
        let state = self.state.read();
        if state.sub_trees.is_empty() || entry.length() == 2 {
            return true;
        }
        state
            .sub_trees
            .iter()
            .any(|tree| tree_below_entry(entry, tree))
    }
    /// `IsPartTree`: a sub-tree is set and it is not the single root entry.
    pub fn is_part_tree(&self) -> bool {
        let state = self.state.read();
        match state.sub_trees.as_slice() {
            [] => false,
            [only] => only.length() >= 3,
            _ => true,
        }
    }
    /// `StartIteration`: rebuilds the iteration tree from the sub-tree entries and puts the
    /// cursor on its root.
    pub fn start_iteration(&self) {
        let mut state = self.state.write();
        state.tree = vec![TagNode::default()];
        state.current = Some(0);
        state.current_depth = 0;
        let entries: Vec<AsciiString> = state.sub_trees.clone();
        for entry in &entries {
            add_entry_path(&mut state, entry);
        }
    }
    /// `Up`: one level up. Native quirk kept as a defined state: going up from the root
    /// makes the current map null (the root keeps a null father); the Rust cursor then is
    /// `None` and [`PcdmReaderFilter::is_passed`] answers false where native code crashes.
    pub fn up(&self) {
        let mut state = self.state.write();
        if state.current_depth != 0 {
            state.current_depth -= 1;
            return;
        }
        state.current = state.current.and_then(|node| state.tree[node].father);
    }
    /// `Down`: one level down to `tag`; below the defined tree only the depth grows.
    pub fn down(&self, tag: i32) {
        let mut state = self.state.write();
        let child = state
            .current
            .and_then(|node| state.tree[node].children.get(&tag).copied());
        match child.filter(|_| state.current_depth == 0) {
            Some(child) => state.current = Some(child),
            None => state.current_depth += 1,
        }
    }
    /// `IsPassed()`: the current node ends a sub-tree definition.
    pub fn is_passed(&self) -> bool {
        let state = self.state.read();
        state
            .current
            .is_some_and(|node| state.tree[node].in_sub_trees)
    }
    /// `IsSubPassed()`: the cursor is inside the defined tree.
    pub fn is_sub_passed(&self) -> bool {
        self.state.read().current_depth == 0
    }
}

/// One `IsPassed(entry)` test: `entry` starts with `tree` at a tag boundary.
fn entry_in_tree(entry: &AsciiString, tree: &AsciiString) -> bool {
    if !entry.starts_with(tree.as_bytes()) {
        return false;
    }
    entry.length() <= tree.length() || entry.value(tree.length() + 1).ok() == Some(b':')
}

/// One `IsSubPassed(entry)` test: `tree` starts with `entry` at a tag boundary.
fn tree_below_entry(entry: &AsciiString, tree: &AsciiString) -> bool {
    entry.length() < tree.length()
        && tree.value(entry.length() + 1).ok() == Some(b':')
        && tree.starts_with(entry.as_bytes())
}

/// Adds one sub-tree entry to the iteration tree, skipping the root tag (token 1).
fn add_entry_path(state: &mut FilterState, entry: &AsciiString) {
    let mut node: usize = 0;
    let mut index: i32 = 2;
    loop {
        let token = entry.token(":", index);
        if entry.is_empty() || token.is_empty() {
            break;
        }
        // Native `IntegerValue` of a non-numeric tag is 0.
        node = child_node(state, node, token.integer_value().unwrap_or(0));
        index += 1;
    }
    state.tree[node].in_sub_trees = true;
}

/// The child of `node` with `tag`, created with its father link when it is new.
fn child_node(state: &mut FilterState, node: usize, tag: i32) -> usize {
    if let Some(&child) = state.tree[node].children.get(&tag) {
        return child;
    }
    let child = state.tree.len();
    state.tree.push(TagNode {
        father: Some(node),
        ..TagNode::default()
    });
    state.tree[node].children.insert(tag, child);
    child
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter_of(entries: &[&str]) -> PcdmReaderFilter {
        let filter = PcdmReaderFilter::new();
        for entry in entries {
            filter.add_path(&AsciiString::new(entry));
        }
        filter
    }

    #[test]
    fn entries_match_at_tag_boundaries() {
        let filter = filter_of(&["0:1"]);
        assert!(filter.is_passed_entry(&AsciiString::new("0:1")));
        assert!(filter.is_passed_entry(&AsciiString::new("0:1:5")));
        assert!(!filter.is_passed_entry(&AsciiString::new("0:10")));
        assert!(filter.is_sub_passed_entry(&AsciiString::new("0:")));
        assert!(!filter.is_sub_passed_entry(&AsciiString::new("0:1")));
        assert!(filter_of(&["0:1:2"]).is_sub_passed_entry(&AsciiString::new("0:1")));
        assert!(filter.is_part_tree() && !filter_of(&["0:"]).is_part_tree());
    }

    #[test]
    fn iteration_walks_the_defined_tree() {
        let filter = filter_of(&["0:1:2"]);
        filter.start_iteration();
        assert!(!filter.is_passed() && filter.is_sub_passed());
        filter.down(1);
        filter.down(2);
        assert!(filter.is_passed());
        filter.down(7);
        assert!(!filter.is_sub_passed());
        filter.up();
        filter.up();
        filter.up();
        assert!(!filter.is_passed() && filter.is_sub_passed());
        filter.up();
        assert!(!filter.is_passed());
    }

    #[test]
    fn attribute_types_use_read_before_skip() {
        let filter = PcdmReaderFilter::new();
        filter.add_skipped(&AsciiString::new("A"));
        assert!(!filter.is_passed_attr(&AsciiString::new("A")));
        filter.add_read(&AsciiString::new("B"));
        assert!(filter.is_passed_attr(&AsciiString::new("B")));
        assert!(!filter.is_passed_attr(&AsciiString::new("C")));
        filter.clear();
        assert!(filter.is_passed_attr(&AsciiString::new("A")));
        assert!(!filter.is_append_mode());
        filter.set_mode(super::super::PcdmReaderFilterAppendMode::Protect);
        assert!(filter.is_append_mode());
    }
}
