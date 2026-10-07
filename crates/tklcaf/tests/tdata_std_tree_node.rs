//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDataStd_TreeNode_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{
    TDataStdChildNodeIterator, TDataStdDirectory, TDataStdGenericExtString, TDataStdName,
    TDataStdTreeNode,
};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfData, TdfLabel, TdfTagSource};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

type Node = Handle<TDataStdTreeNode>;

/// The fixture `TreeData`: a framework, a root node and ten child nodes on labels 2..12.
struct TreeData {
    _data: Handle<TdfData>,
    root: Node,
    children: Vec<Node>,
}

impl TreeData {
    /// `myChild<index>` (1-based, as the fixture names them).
    fn child(&self, index: usize) -> &Node {
        &self.children[index - 1]
    }
}

fn make_label(data: &Handle<TdfData>, tag: i32) -> TdfLabel {
    data.root().find_child(tag, true).unwrap()
}

fn make_flat_tree() -> TreeData {
    let data = TdfData::new();
    let root = TDataStdTreeNode::set(&make_label(&data, 2)).unwrap();
    let children = (3..=12)
        .map(|tag| TDataStdTreeNode::set(&make_label(&data, tag)).unwrap())
        .collect();
    TreeData {
        _data: data,
        root,
        children,
    }
}

fn make_nested_tree() -> TreeData {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.child(1).append(tree.child(3)).unwrap());
    assert!(tree.child(1).append(tree.child(4)).unwrap());
    assert!(tree.child(1).append(tree.child(5)).unwrap());
    assert!(tree.child(2).append(tree.child(6)).unwrap());
    assert!(tree.child(2).append(tree.child(7)).unwrap());
    tree
}

fn expect_tree_order(root: &Node, all_levels: bool, expected: &[&Node]) {
    let mut iterator = TDataStdChildNodeIterator::new(root, all_levels).unwrap();
    let mut index = 0;
    while iterator.more() && index < expected.len() {
        assert_eq!(iterator.value().label(), expected[index].label());
        iterator.next().unwrap();
        index += 1;
    }
    assert_eq!(index, expected.len());
    assert!(!iterator.more());
}

fn name_of(name: &Handle<TDataStdName>) -> ExtendedString {
    TDataStdGenericExtString::get(&**name)
}

// Test BUC60817: TDataStd_TreeNode descendant relationship
#[test]
fn TDataStd_TreeNode_Test__BUC60817_DescendantRelationship() {
    let data = TdfData::new();
    let label1 = data.root().find_child(2, true).unwrap();
    let label2 = data.root().find_child(3, true).unwrap();
    let node1 = TDataStdTreeNode::set(&label1).unwrap();
    let node2 = TDataStdTreeNode::set(&label2).unwrap();
    node1.append(&node2).unwrap();
    assert!(
        node2.is_descendant(&node1),
        "TN2 should be a descendant of TN1 after Append"
    );
    assert!(
        !node1.is_descendant(&node2),
        "TN1 should not be a descendant of TN2"
    );
}

// caf/bugs/bug24645: undoing the removal of intermediate children does not leave
// a stale last-child pointer on the tree node.
#[test]
fn TDataStd_TreeNode_Test__CafBug_24645_UndoDetachedChildren() {
    let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
    document.set_undo_limit(1000).unwrap();
    let root_label = document.main().unwrap().find_child(1, true).unwrap();
    let root = TDataStdTreeNode::set(&root_label).unwrap();
    let mut children = Vec::new();
    document.new_command().unwrap();
    for index in 1..=5 {
        children.push(TDataStdTreeNode::set(&root_label.find_child(index, true).unwrap()).unwrap());
        assert!(root.append(children.last().unwrap()).unwrap());
        document.new_command().unwrap();
    }
    assert!(children[2].remove().unwrap());
    assert!(children[3].remove().unwrap());
    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(!root.has_last());
}

// caf/tree/A1: TDF_TagSource::NewChild keeps the child label addressable.
#[test]
fn TDataStd_TreeNode_Test__CafTree_A1_TagSourceNewChild() {
    let data = TdfData::new();
    let parent = make_label(&data, 2);
    let child = TdfTagSource::new_child_of(&parent).unwrap();
    let name = TDataStdName::set(&child, &ExtendedString::new("Label11")).unwrap();
    assert!(!name.is_null());
    assert_eq!(child.father(), parent);
    assert!(name_of(&name).is_equal(ExtendedString::new("Label11")));
}

// caf/tree/A2: explicit tag allocation and child creation use the same source.
#[test]
fn TDataStd_TreeNode_Test__CafTree_A2_TagSourceNewTag() {
    let data = TdfData::new();
    let parent1 = make_label(&data, 2);
    let parent2 = make_label(&data, 3);
    let _tagger1 = TdfTagSource::set(&parent1).unwrap();
    let tagger2 = TdfTagSource::set(&parent2).unwrap();
    let child1 = TdfTagSource::new_child_of(&parent1).unwrap();
    tagger2.new_tag().unwrap();
    tagger2.new_tag().unwrap();
    tagger2.new_tag().unwrap();
    let child2 = TdfTagSource::new_child_of(&parent2).unwrap();
    assert_eq!(child1.tag(), 1);
    assert_eq!(child2.tag(), 4);
}

// caf/tree/B1: TDataStd_Directory::New creates a directory on the label.
#[test]
fn TDataStd_TreeNode_Test__CafTree_B1_NewDirectory() {
    let data = TdfData::new();
    let label = make_label(&data, 2);
    let directory = TDataStdDirectory::new_on(&label).unwrap();
    let name = TDataStdName::set(&label, &ExtendedString::new("Label1")).unwrap();
    assert!(!directory.is_null());
    assert!(!name.is_null());
    assert_eq!(directory.label(), label);
    assert!(name_of(&name).is_equal(ExtendedString::new("Label1")));
}

// caf/tree/B2: adding a directory allocates a child label and directory.
#[test]
fn TDataStd_TreeNode_Test__CafTree_B2_AddDirectory() {
    let data = TdfData::new();
    let parent = make_label(&data, 2);
    let directory = TDataStdDirectory::new_on(&parent).unwrap();
    let sub_directory = TDataStdDirectory::add_directory(&directory).unwrap();
    let name = TDataStdName::set(
        &sub_directory.label(),
        &ExtendedString::new("SubDirectory1"),
    )
    .unwrap();
    assert!(!sub_directory.is_null());
    assert!(!name.is_null());
    assert_eq!(sub_directory.label().father(), parent);
    assert!(name_of(&name).is_equal(ExtendedString::new("SubDirectory1")));
}

// caf/tree/C1: a node is its own root when it has no father.
#[test]
fn TDataStd_TreeNode_Test__CafTree_C1_RootNode() {
    let tree = make_flat_tree();
    assert_eq!(tree.root.root().unwrap().label(), tree.root.label());
    assert!(tree.root.is_root());
}

// caf/tree/D1-D2: AppendNode preserves insertion order.
#[test]
fn TDataStd_TreeNode_Test__CafTree_D1_AppendNodeCount() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.root.append(tree.child(3)).unwrap());
    assert_eq!(tree.root.nb_children(false), 3);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_D2_AppendNodeOrder() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.root.append(tree.child(3)).unwrap());
    expect_tree_order(
        &tree.root,
        true,
        &[tree.child(1), tree.child(2), tree.child(3)],
    );
}

// caf/tree/E1-E2: PrependNode inserts each new node before the old first child.
#[test]
fn TDataStd_TreeNode_Test__CafTree_E1_PrependNodeCount() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.prepend(tree.child(2)).unwrap());
    assert!(tree.root.prepend(tree.child(3)).unwrap());
    assert_eq!(tree.root.nb_children(false), 3);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_E2_PrependNodeOrder() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.prepend(tree.child(2)).unwrap());
    assert!(tree.root.prepend(tree.child(3)).unwrap());
    expect_tree_order(
        &tree.root,
        true,
        &[tree.child(3), tree.child(2), tree.child(1)],
    );
}

// caf/tree/F1-F2: InsertNodeBefore places a node immediately before its target.
#[test]
fn TDataStd_TreeNode_Test__CafTree_F1_InsertNodeBeforeCount() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.child(2).insert_before(tree.child(3)).unwrap());
    assert_eq!(tree.root.nb_children(false), 3);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_F2_InsertNodeBeforeOrder() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.child(2).insert_before(tree.child(3)).unwrap());
    expect_tree_order(
        &tree.root,
        true,
        &[tree.child(1), tree.child(3), tree.child(2)],
    );
}

// caf/tree/G1-G2: InsertNodeAfter places a node immediately after its target.
#[test]
fn TDataStd_TreeNode_Test__CafTree_G1_InsertNodeAfterCount() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.child(1).insert_after(tree.child(3)).unwrap());
    assert_eq!(tree.root.nb_children(false), 3);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_G2_InsertNodeAfterOrder() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.child(1).insert_after(tree.child(3)).unwrap());
    expect_tree_order(
        &tree.root,
        true,
        &[tree.child(1), tree.child(3), tree.child(2)],
    );
}

// caf/tree/H1-H2: Remove detaches a node without changing its siblings.
#[test]
fn TDataStd_TreeNode_Test__CafTree_H1_DetachNodeCount() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.root.append(tree.child(3)).unwrap());
    assert!(tree.child(2).remove().unwrap());
    assert_eq!(tree.root.nb_children(false), 2);
    assert!(tree.child(2).father().is_null());
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_H2_DetachNodeOrder() {
    let tree = make_flat_tree();
    assert!(tree.root.append(tree.child(1)).unwrap());
    assert!(tree.root.append(tree.child(2)).unwrap());
    assert!(tree.root.append(tree.child(3)).unwrap());
    assert!(tree.child(2).remove().unwrap());
    expect_tree_order(&tree.root, true, &[tree.child(1), tree.child(3)]);
}

// caf/tree/I1-I4: ChildNodeIterator supports both direct and recursive walks.
#[test]
fn TDataStd_TreeNode_Test__CafTree_I1_ChildNodeIterateDirectCount() {
    let tree = make_nested_tree();
    assert_eq!(tree.root.nb_children(false), 2);
    expect_tree_order(&tree.root, false, &[tree.child(1), tree.child(2)]);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_I2_ChildNodeIterateDirectOrder() {
    let tree = make_nested_tree();
    expect_tree_order(&tree.root, false, &[tree.child(1), tree.child(2)]);
}

/// The all-levels order of the nested fixture.
fn nested_all_levels(tree: &TreeData) -> Vec<&Node> {
    [1, 3, 4, 5, 2, 6, 7]
        .into_iter()
        .map(|index| tree.child(index))
        .collect()
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_I3_ChildNodeIterateAllLevelsCount() {
    let tree = make_nested_tree();
    assert_eq!(tree.root.nb_children(true), 7);
    expect_tree_order(&tree.root, true, &nested_all_levels(&tree));
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_I4_ChildNodeIterateAllLevelsOrder() {
    let tree = make_nested_tree();
    expect_tree_order(&tree.root, true, &nested_all_levels(&tree));
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_J1_ChildNodeIteratorMoreAllLevels() {
    let tree = make_nested_tree();
    let iterator = TDataStdChildNodeIterator::new(&tree.root, true).unwrap();
    assert!(iterator.more());
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_J2_ChildNodeIteratorMoreDirect() {
    let tree = make_nested_tree();
    let iterator = TDataStdChildNodeIterator::new(&tree.root, false).unwrap();
    assert!(iterator.more());
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_K1_ChildNodeIteratorNextAllLevels() {
    let tree = make_nested_tree();
    let mut iterator = TDataStdChildNodeIterator::new(&tree.root, true).unwrap();
    let mut child_count = 0;
    while iterator.more() {
        iterator.next().unwrap();
        child_count += 1;
    }
    assert_eq!(child_count, 7);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_K2_ChildNodeIteratorNextDirect() {
    let tree = make_nested_tree();
    let mut iterator = TDataStdChildNodeIterator::new(&tree.root, false).unwrap();
    let mut child_count = 0;
    while iterator.more() {
        iterator.next().unwrap();
        child_count += 1;
    }
    assert_eq!(child_count, 2);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_L1_ChildNodeIteratorValueDirect() {
    let tree = make_nested_tree();
    expect_tree_order(&tree.root, false, &[tree.child(1), tree.child(2)]);
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_L2_ChildNodeIteratorValueAllLevels() {
    let tree = make_nested_tree();
    expect_tree_order(&tree.root, true, &nested_all_levels(&tree));
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_M1_ChildNodeIteratorNextBrotherAllLevels() {
    let tree = make_nested_tree();
    assert!(tree.child(4).append(tree.child(8)).unwrap());
    assert!(tree.child(4).append(tree.child(9)).unwrap());
    let mut iterator = TDataStdChildNodeIterator::new(&tree.root, true).unwrap();
    iterator.next().unwrap();
    iterator.next().unwrap();
    iterator.next().unwrap();
    iterator.next_brother().unwrap();
    assert_eq!(iterator.value().label(), tree.child(9).label());
    iterator.next_brother().unwrap();
    assert_eq!(iterator.value().label(), tree.child(5).label());
    iterator.next_brother().unwrap();
    assert_eq!(iterator.value().label(), tree.child(2).label());
}

#[test]
fn TDataStd_TreeNode_Test__CafTree_M2_ChildNodeIteratorNextBrotherDirect() {
    let tree = make_nested_tree();
    assert!(tree.child(4).append(tree.child(8)).unwrap());
    assert!(tree.child(4).append(tree.child(9)).unwrap());
    assert!(tree.root.append(tree.child(10)).unwrap());
    let mut iterator = TDataStdChildNodeIterator::new(&tree.root, false).unwrap();
    iterator.next_brother().unwrap();
    assert_eq!(iterator.value().label(), tree.child(2).label());
    iterator.next_brother().unwrap();
    assert_eq!(iterator.value().label(), tree.child(10).label());
}

// caf/bugs/buc60844: forgetting a tree node inside a command is reversible.
#[test]
fn TDataStd_TreeNode_Test__Buc_CafBug_60844_ForgetAllUndo() {
    let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
    document.set_undo_limit(100).unwrap();
    document.new_command().unwrap();
    let label1 = document.main().unwrap();
    let label2 = document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(2, true)
        .unwrap();
    let label21 = label2.find_child(1, true).unwrap();
    let label211 = label21.find_child(1, true).unwrap();
    let label212 = label21.find_child(2, true).unwrap();
    let label3 = document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(3, true)
        .unwrap();
    let node1 = TDataStdTreeNode::set(&label1).unwrap();
    let node2 = TDataStdTreeNode::set(&label2).unwrap();
    let mut node21 = TDataStdTreeNode::set(&label21).unwrap();
    let node211 = TDataStdTreeNode::set(&label211).unwrap();
    let node212 = TDataStdTreeNode::set(&label212).unwrap();
    let node3 = TDataStdTreeNode::set(&label3).unwrap();
    assert!(node1.insert_after(&node2).unwrap());
    assert!(node2.insert_after(&node3).unwrap());
    assert!(node2.append(&node21).unwrap());
    assert!(node21.append(&node211).unwrap());
    assert!(node211.insert_after(&node212).unwrap());
    assert_eq!(node2.nb_children(false), 1);
    assert_eq!(node21.nb_children(false), 2);
    assert_eq!(node21.root().unwrap().label(), node2.label());

    document.new_command().unwrap();
    label21.forget_all_attributes(true).unwrap();
    assert!(!TDataStdTreeNode::find(&label21, &mut node21).unwrap());

    document.new_command().unwrap();
    assert!(document.undo().unwrap());

    let mut restored21: Node = Handle::default();
    assert!(TDataStdTreeNode::find(&label21, &mut restored21).unwrap());
    assert_eq!(restored21.father().label(), label2);
    assert_eq!(restored21.nb_children(false), 2);
    assert_eq!(restored21.first().label(), label211);
    assert_eq!(restored21.last().label(), label212);
    assert_eq!(restored21.root().unwrap().label(), label2);
}
