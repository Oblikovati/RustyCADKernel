//! Port-only regression tests whose expectations come from the native OCCT run of the same
//! scenarios (3d097a0328e71b826377d4814ab05ec3c3d23871, linux-gcc, C locale): the quirks the
//! port keeps (a new name of SetReal is not undone, XLinkTool reconnects a lone target node
//! by its father link only, the BooleanArray bit packing) and the DumpJson text.
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{
    TDataStdBooleanArray, TDataStdByteArray, TDataStdExtStringArray, TDataStdIntPackedMap,
    TDataStdIntegerList, TDataStdNamedData, TDataStdRealArray, TDataStdTreeNode,
};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeState, TdfData};
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdXLinkTool};

/// `DumpJson(stream, 0)` as text.
fn json<A: TdfAttribute + ?Sized>(attribute: &A) -> Vec<u8> {
    let mut stream = DumpStream::default();
    attribute.dump_json(&mut stream, 0).unwrap();
    stream.bytes().to_vec()
}

fn name(text: &str) -> ExtendedString {
    ExtendedString::new(text)
}

#[test]
fn set_real_of_a_new_name_survives_undo_as_natively() {
    let document = TDocStdDocument::new(&name("BinOcaf")).unwrap();
    document.set_undo_limit(10).unwrap();
    let named = TDataStdNamedData::set(&document.main().unwrap()).unwrap();
    document.new_command().unwrap();
    named.set_real(&name("x"), 1.5).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(named.has_real(&name("x")), "native: 1 hasReal=1");
}

#[test]
fn xlink_tool_reconnects_as_natively() {
    let document = TDocStdDocument::new(&name("BinOcaf")).unwrap();
    let main = document.main().unwrap();
    let node_on = |tag| TDataStdTreeNode::set(&main.find_child(tag, true).unwrap()).unwrap();
    let (root, brother, root2) = (node_on(1), node_on(4), node_on(5));
    let (source, target) = (
        main.find_child(2, true).unwrap(),
        main.find_child(3, true).unwrap(),
    );
    let (source_node, target_node) = (
        TDataStdTreeNode::set(&source).unwrap(),
        TDataStdTreeNode::set(&target).unwrap(),
    );
    root.append(&brother).unwrap();
    root.append(&source_node).unwrap();
    root2.append(&target_node).unwrap();
    TDocStdXLinkTool::new().copy(&target, &source).unwrap();
    let mut node: Handle<TDataStdTreeNode> = Handle::default();
    TDataStdTreeNode::find(&target, &mut node).unwrap();
    // native: 2 rootChildren=2 brotherNextIsSource=1 targetFatherIsRoot2=1 root2Children=0
    assert_eq!(root.nb_children(false), 2);
    assert_eq!(brother.next().label(), source);
    assert_eq!(node.father(), root2);
    assert_eq!(root2.nb_children(false), 0);
}

#[test]
fn boolean_array_packing_and_json_match_native() {
    let data = TdfData::new();
    let array = TDataStdBooleanArray::set(&data.root(), 3, 12).unwrap();
    array.set_value(3, true).unwrap();
    array.set_value(12, true).unwrap();
    array.set_value(12, false).unwrap();
    array.set_value(11, true).unwrap();
    let bytes: Vec<u8> = array
        .internal_array()
        .get()
        .unwrap()
        .array1()
        .iter()
        .copied()
        .collect();
    assert_eq!(bytes, vec![1, 1]);
    assert!(!array.value(2).unwrap() && !array.value(13).unwrap());
    let expected = b"\"className\": \"TDataStd_BooleanArray\", \"Values->Lower\": 0, \"Values->Upper\": 1, \"Value\": \x01, \"Value\": \x01, \"Lower\": 3, \"Upper\": 12";
    assert_eq!(json(&*array), expected.to_vec());
}

#[test]
fn array_list_map_and_named_data_json_match_native() {
    let data = TdfData::new();
    let label = data.root();
    let reals = TDataStdRealArray::set(&label, 1, 2, false).unwrap();
    reals.set_value(1, 0.1).unwrap();
    reals.set_value(2, 1e20).unwrap();
    assert_eq!(json(&*reals), b"\"className\": \"TDataStd_RealArray\", \"Value->Lower\": 1, \"Value->Upper\": 2, \"Value\": 0.1, \"Value\": 1e+20, \"IsDelta\": 0".to_vec());
    let bytes = TDataStdByteArray::set(&label, 1, 2, false).unwrap();
    bytes.set_value(1, 65).unwrap();
    bytes.set_value(2, 66).unwrap();
    assert_eq!(json(&*bytes), b"\"className\": \"TDataStd_ByteArray\", \"Value->Lower\": 1, \"Value->Upper\": 2, \"Value\": A, \"Value\": B, \"IsDelta\": 0".to_vec());
    let strings = TDataStdExtStringArray::set(&label, 1, 1, false).unwrap();
    strings.set_value(1, name("abc")).unwrap();
    assert_eq!(json(&*strings), b"\"className\": \"TDataStd_ExtStringArray\", \"Value->Lower\": 1, \"Value->Upper\": 1, \"Value\": \"abc\", \"IsDelta\": 0".to_vec());
    let map = TDataStdIntPackedMap::set(&label, false).unwrap();
    for key in [70, 3, 1000] {
        map.add(key).unwrap();
    }
    assert_eq!(json(&*map), b"\"className\": \"TDataStd_IntPackedMap\", \"Key\": 3, \"Key\": 70, \"Key\": 1000, \"IsDelta\": 0".to_vec());
    let mut dump = Vec::new();
    map.dump(&mut dump).unwrap();
    let dump = String::from_utf8(dump).unwrap();
    assert!(dump.ends_with("IntPackedMap\tTrans. 0; Valid;\tID = 7031faff-161e-44df-8239-7c264a81f5a1\nIntPackedMap size = 3 Delta is OFF\n"), "{dump:?}");
    let named = TDataStdNamedData::set(&label).unwrap();
    named.set_integer(&name("i"), 1).unwrap();
    assert!(
        named
            .get_real(&name("missing"))
            .unwrap_err()
            .is_kind("Standard_NoSuchObject")
    );
    assert_eq!(json(&*named), b"\"className\": \"TDataStd_NamedData\", \"Integers->Map().Size\": 1, \"Reals->Map().Size\": 0".to_vec());
    let list = TDataStdIntegerList::set(&label).unwrap();
    list.append(5).unwrap();
    list.append(6).unwrap();
    assert_eq!(
        json(&*list),
        b"\"className\": \"TDataStd_IntegerList\", \"Value\": 5, \"Value\": 6".to_vec()
    );
}
