//! Port-only tests of TDocStd_XLinkTool with TDataStd attributes (the upstream
//! TDocStd_XLink_Test needs TNaming and BRepPrimAPI, which other toolkits port).
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfReference};
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdXLink, TDocStdXLinkTool};

fn document() -> Handle<TDocStdDocument> {
    let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
    document.set_undo_limit(100).unwrap();
    document
}

#[test]
fn copy_with_link_records_the_link_and_update_link_copies_again() {
    let document = document();
    let source = document.main().unwrap().find_child(2, true).unwrap();
    let target = document.main().unwrap().find_child(3, true).unwrap();
    TDataStdInteger::set(&source, 5).unwrap();
    document.new_command().unwrap();
    let mut tool = TDocStdXLinkTool::new();
    tool.copy_with_link(&target, &source).unwrap();
    assert!(tool.is_done());
    let mut link: Handle<TDocStdXLink> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TDocStdXLink::get_id(), &mut link)
            .unwrap()
    );
    assert_eq!(link.document_entry().as_bytes(), b"0");
    assert!(!link.label_entry().is_empty());
    let mut reference: Handle<TdfReference> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TdfReference::get_id(), &mut reference)
            .unwrap()
    );
    assert_eq!(TdfReference::get(&reference), source);
    TDataStdInteger::set(&source, 9).unwrap();
    tool.update_link(&target).unwrap();
    let mut copied: Handle<TDataStdInteger> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TDataStdInteger::get_id(), &mut copied)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&copied), 9);
    assert!(
        tool.copy_with_link(&target, &source)
            .unwrap_err()
            .is_kind("Standard_DomainError")
    );
}

#[test]
fn copy_reconnects_the_tree_nodes_as_natively() {
    // The source node goes back after its previous brother; the target node, alone under
    // its father, only gets its father link back (TDocStd_XLinkTool::Copy calls
    // SetFather, which does not make it a child again).
    let document = document();
    let main = document.main().unwrap();
    let node_on = |tag| TDataStdTreeNode::set(&main.find_child(tag, true).unwrap()).unwrap();
    let (root, brother, root2) = (node_on(1), node_on(4), node_on(5));
    let source = main.find_child(2, true).unwrap();
    let target = main.find_child(3, true).unwrap();
    let source_node = TDataStdTreeNode::set(&source).unwrap();
    let target_node = TDataStdTreeNode::set(&target).unwrap();
    root.append(&brother).unwrap();
    root.append(&source_node).unwrap();
    root2.append(&target_node).unwrap();
    let mut tool = TDocStdXLinkTool::new();
    tool.copy(&target, &source).unwrap();
    assert!(tool.is_done() && tool.data_set().is_some());
    assert_eq!(root.nb_children(false), 2);
    assert_eq!(brother.next().label(), source);
    let mut node: Handle<TDataStdTreeNode> = Handle::default();
    assert!(TDataStdTreeNode::find(&target, &mut node).unwrap());
    assert_eq!(node.father().label(), root2.label());
    assert_eq!(root2.nb_children(false), 0);
}

#[test]
fn update_link_without_reference_is_a_domain_error() {
    let document = document();
    let label = document.main().unwrap();
    let error = TDocStdXLinkTool::new().update_link(&label).unwrap_err();
    assert!(error.is_kind("Standard_DomainError"));
}
