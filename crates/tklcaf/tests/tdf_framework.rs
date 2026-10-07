//! Port-only behaviour tests of the TDF package, written against the native C++ bodies.
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tklcaf::tdf::{
    Tdf, TdfAttribute, TdfAttributeIterator, TdfAttributeState, TdfChildIdIterator,
    TdfChildIterator, TdfClosureTool, TdfCopyLabel, TdfData, TdfDataSet, TdfIdFilter, TdfLabel,
    TdfReference, TdfRelocationTable, TdfTagSource, TdfTool, TdfTransaction,
};

fn entry_of(label: &TdfLabel) -> String {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    String::from_utf8_lossy(entry.as_bytes()).into_owned()
}

fn tag_source(label: &TdfLabel) -> Handle<TdfTagSource> {
    let mut found: Handle<TdfTagSource> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TdfTagSource::get_id(), &mut found)
            .unwrap()
    );
    found
}

#[test]
fn labels_are_kept_in_tag_order() {
    let data = TdfData::new();
    let root = data.root();
    for tag in [5, 1, 3] {
        root.find_child(tag, true).unwrap();
    }
    let mut children = TdfChildIterator::new(&root, false).unwrap();
    let mut tags = Vec::new();
    while children.more() {
        tags.push(children.value().tag());
        children.next();
    }
    assert_eq!(tags, vec![1, 3, 5]);
    assert!(root.find_child(4, false).unwrap().is_null());
    assert_eq!(root.nb_children().unwrap(), 3);
    let deep = root
        .find_child(3, true)
        .unwrap()
        .find_child(7, true)
        .unwrap();
    assert_eq!(entry_of(&deep), "0:3:7");
    assert!(deep.is_descendant(&root) && !root.is_descendant(&deep));
    assert_eq!(TdfTool::nb_labels(&root).unwrap(), 5);
    let mut all = TdfChildIterator::new(&root, true).unwrap();
    let mut entries = Vec::new();
    while all.more() {
        entries.push(entry_of(&all.value()));
        all.next();
    }
    assert_eq!(entries, ["0:1", "0:3", "0:3:7", "0:5"]);
}

#[test]
fn null_labels_follow_the_native_exceptions() {
    let null = TdfLabel::new();
    assert_eq!(
        null.depth().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert!(null.find_child(1, true).is_err() && null.nb_children().is_err());
    assert_eq!(entry_of(&null), "");
    assert!(!null.is_root() && null.father().is_null() && null.data().is_null());
}

#[test]
fn tool_finds_labels_from_entries() {
    let data = TdfData::new();
    let mut label = TdfLabel::default();
    TdfTool::label_from_entry(&data, &AsciiString::new("0:2:4"), &mut label, true).unwrap();
    assert_eq!(entry_of(&label), "0:2:4");
    let mut tags = Vec::new();
    TdfTool::tag_list_from_entry(b"0:2:x", &mut tags);
    assert!(tags.is_empty());
    data.set_access_by_entries(true).unwrap();
    let mut found = TdfLabel::default();
    assert!(data.get_label(&AsciiString::new("0:2:4"), &mut found) && found == label);
    let created = data.root().find_child(9, true).unwrap();
    assert!(data.get_label(&AsciiString::new("0:9"), &mut found) && found == created);
    let mut moved = TdfLabel::default();
    let target_root = data.root().find_child(8, true).unwrap();
    TdfTool::relocate_label(&label, &data.root(), &target_root, &mut moved, true).unwrap();
    assert_eq!(entry_of(&moved), "0:8:2:4");
}

#[test]
fn abort_removes_attributes_added_in_the_transaction() {
    let data = TdfData::new();
    let mut transaction = TdfTransaction::with_data(&data, &AsciiString::new("add"));
    transaction.open().unwrap();
    let source = TdfTagSource::set(&data.root()).unwrap();
    assert_eq!(source.transaction(), 1);
    assert!(data.root().attributes_modified());
    transaction.abort().unwrap();
    assert!(!data.root().is_attribute(&TdfTagSource::get_id()).unwrap());
    assert!(source.label().is_null() && source.is_forgotten());
}

#[test]
fn commit_with_delta_supports_undo_and_redo() {
    let data = TdfData::new();
    let mut transaction = TdfTransaction::with_data(&data, &AsciiString::new("t"));
    transaction.open().unwrap();
    TdfTagSource::set(&data.root()).unwrap();
    let delta = transaction.commit(true).unwrap();
    assert_eq!(data.time(), 1);
    assert!(data.is_applicable(&delta));
    let delta_ref = delta.get().unwrap();
    assert_eq!(
        (
            delta_ref.begin_time(),
            delta_ref.end_time(),
            delta_ref.attribute_deltas().len()
        ),
        (0, 1, 1)
    );
    let redo = data.undo(&delta, true).unwrap();
    assert!(!data.root().is_attribute(&TdfTagSource::get_id()).unwrap());
    assert_eq!(data.time(), 0);
    assert!(data.is_applicable(&redo));
    data.undo(&redo, false).unwrap();
    assert!(data.root().is_attribute(&TdfTagSource::get_id()).unwrap());
}

#[test]
fn modifications_back_up_and_undo_restores_the_value() {
    let data = TdfData::new();
    TdfTagSource::set(&data.root()).unwrap();
    let source = tag_source(&data.root());
    let mut transaction = TdfTransaction::with_data(&data, &AsciiString::new("m"));
    transaction.open().unwrap();
    source.set_value(7).unwrap();
    assert!(!source.is_new() && source.transaction() == 1);
    let delta = transaction.commit(true).unwrap();
    assert_eq!(TdfTagSource::get(&source), 7);
    assert!(
        source.is_new(),
        "the backup of the outermost transaction is removed"
    );
    data.undo(&delta, false).unwrap();
    assert_eq!(TdfTagSource::get(&tag_source(&data.root())), 0);
}

#[test]
fn forget_and_resume_through_transactions() {
    let data = TdfData::new();
    let label = data.root().find_child(1, true).unwrap();
    TdfTagSource::set(&label).unwrap();
    let mut transaction = TdfTransaction::with_data(&data, &AsciiString::new("f"));
    transaction.open().unwrap();
    assert!(label.forget_attribute_id(&TdfTagSource::get_id()).unwrap());
    assert!(!label.is_attribute(&TdfTagSource::get_id()).unwrap());
    let mut forgotten = TdfAttributeIterator::new(&label, false).unwrap();
    assert!(forgotten.more() && forgotten.value().get().unwrap().is_forgotten());
    forgotten.next();
    assert!(!forgotten.more());
    let delta = transaction.commit(true).unwrap();
    assert!(
        !label.has_attribute().unwrap(),
        "removed at commit of transaction 1"
    );
    data.undo(&delta, false).unwrap();
    assert!(label.is_attribute(&TdfTagSource::get_id()).unwrap());
}

#[test]
fn nested_transactions_commit_into_the_enclosing_one() {
    let data = TdfData::new();
    let mut outer = TdfTransaction::with_data(&data, &AsciiString::new("outer"));
    outer.open().unwrap();
    let mut inner = TdfTransaction::with_data(&data, &AsciiString::new("inner"));
    assert_eq!(inner.open().unwrap(), 2);
    let child = TdfTagSource::new_child_of(&data.root()).unwrap();
    assert_eq!(child.tag(), 1);
    inner.commit(false).unwrap();
    assert_eq!(tag_source(&data.root()).transaction(), 1);
    outer.abort().unwrap();
    assert!(!data.root().is_attribute(&TdfTagSource::get_id()).unwrap());
    assert!(!outer.is_open() && data.transaction() == 0);
}

#[test]
fn modification_outside_transactions_can_be_forbidden() {
    let data = TdfData::new();
    data.allow_modification(false);
    let error = TdfTagSource::set(&data.root()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ImmutableObject");
    data.allow_modification(true);
    let source = TdfTagSource::set(&data.root()).unwrap();
    let duplicate: Handle<dyn TdfAttribute> = Handle::new(TdfTagSource::new()).cast();
    assert_eq!(
        data.root()
            .add_attribute(&duplicate, true)
            .unwrap_err()
            .exception_type(),
        "Standard_DomainError"
    );
    let attached: Handle<dyn TdfAttribute> = source.cast();
    let other = data.root().find_child(1, true).unwrap();
    assert_eq!(
        other
            .add_attribute(&attached, true)
            .unwrap_err()
            .exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn copy_label_copies_attributes_and_relocates_references() {
    let data = TdfData::new();
    let source = data.root().find_child(1, true).unwrap();
    let inner = source.find_child(1, true).unwrap();
    TdfReference::set(&source, &inner).unwrap();
    TdfTagSource::set(&inner).unwrap().set_value(4).unwrap();
    let target = data.root().find_child(2, true).unwrap();
    let mut copy = TdfCopyLabel::with_labels(&source, &target);
    copy.perform().unwrap();
    assert!(copy.is_done());
    let mut reference: Handle<TdfReference> = Handle::default();
    assert!(
        target
            .find_attribute_as(&TdfReference::get_id(), &mut reference)
            .unwrap()
    );
    assert_eq!(entry_of(&TdfReference::get(&reference)), "0:2:1");
    assert_eq!(
        TdfTagSource::get(&tag_source(&target.find_child(1, false).unwrap())),
        4
    );
}

#[test]
fn closure_and_self_containment_follow_references() {
    let data = TdfData::new();
    let first = data.root().find_child(1, true).unwrap();
    let outside = data.root().find_child(2, true).unwrap();
    TdfTagSource::set(&outside).unwrap();
    TdfReference::set(&first, &outside).unwrap();
    assert!(!TdfTool::is_self_contained(&first).unwrap());
    assert!(TdfTool::is_self_contained(&outside).unwrap());
    let set = Handle::new(TdfDataSet::new());
    set.add_label(&first);
    TdfClosureTool::closure(&set).unwrap();
    assert!(set.contains_label(&outside) && set.roots().len() == 1);
    let mut referers = Default::default();
    TdfTool::out_referers(&first, &mut referers).unwrap();
    assert_eq!(referers.extent(), 1);
    let mut references = Default::default();
    TdfTool::out_references(&first, &mut references).unwrap();
    assert_eq!(references.extent(), 1);
}

#[test]
fn relocation_table_self_relocation() {
    let data = TdfData::new();
    let label = data.root().find_child(1, true).unwrap();
    let table = TdfRelocationTable::new(true);
    let mut target = TdfLabel::default();
    assert!(table.has_label_relocation(&label, &mut target) && target == label);
    table.set_after_relocate(true);
    assert!(!table.has_label_relocation(&label, &mut target) && target == label);
}

#[test]
fn id_filter_modes() {
    let mut filter = TdfIdFilter::new(false);
    assert!(filter.is_kept(&TdfTagSource::get_id()));
    filter.ignore(&TdfTagSource::get_id());
    assert!(filter.is_ignored(&TdfTagSource::get_id()));
    let mut copy = TdfIdFilter::default();
    copy.copy(&filter);
    assert!(copy.is_ignored(&TdfTagSource::get_id()) && copy.is_kept(&TdfReference::get_id()));
    let mut text = Vec::new();
    filter.dump(&mut text).unwrap();
    assert!(
        String::from_utf8(text)
            .unwrap()
            .starts_with("INCLUSIVE filter: keeps   all IDs BUT:")
    );
}

#[test]
fn child_id_iterator_finds_attributes_by_id() {
    let data = TdfData::new();
    for tag in 1..=3 {
        let child = data.root().find_child(tag, true).unwrap();
        if tag != 2 {
            TdfTagSource::set(&child).unwrap();
        }
    }
    let mut iterator =
        TdfChildIdIterator::new(&data.root(), &TdfTagSource::get_id(), false).unwrap();
    let mut found = Vec::new();
    while iterator.more() {
        found.push(iterator.value().get().unwrap().label().tag());
        iterator.next().unwrap();
    }
    assert_eq!(found, vec![1, 3]);
}

#[test]
fn package_guid_bounds_and_dumps() {
    assert_eq!(
        Tdf::lowest_id().to_string(),
        "00000000-0000-0000-0000-000000000000"
    );
    let data = TdfData::new();
    TdfTagSource::set(&data.root()).unwrap();
    let mut text = Vec::new();
    data.root().dump(&mut text).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(
        text.starts_with("0:\tNOT imported; NOT modified; NO attribute modified; has 1 attribute."),
        "{text}"
    );
    assert!(
        text.contains(
            "\tTDF_TagSource\tTrans. 0; Valid;\tID = 2a96b611-ec8b-11d0-bee7-080009dc3333"
        )
    );
}
