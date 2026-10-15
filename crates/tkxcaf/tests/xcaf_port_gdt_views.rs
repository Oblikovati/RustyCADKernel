//! Port tests of the legacy GD&T entries and the links of XCAFDoc_DimTolTool, the views of
//! XCAFDoc_ViewTool and XCAFDoc_View, the notes and the assembly item references.
mod support;

use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkxcaf::xcaf_dim_tol_objects::XCAFDimTolObjectsTool;
use rustycad_tkxcaf::xcaf_doc::{
    XCAFDocAssemblyItemId, XCAFDocAssemblyItemRef, XCAFDocDimTolTool, XCAFDocDocumentTool,
    XCAFDocNoteMembers, XCAFDocView, XCAFDocViewLinks, xcaf_doc_note_get,
};
use rustycad_tkxcaf::xcaf_note_objects::XCAFNoteObjectsNoteObject;
use rustycad_tkxcaf::xcaf_view::{XCAFViewObject, XCAFViewProjectionType};

fn document() -> Handle<TDocStdDocument> {
    let (_application, document) = support::new_document("BinXCAF");
    document
}

fn two_parts(document: &Handle<TDocStdDocument>) -> (TdfLabel, TdfLabel) {
    let main = document.main().unwrap();
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let shapes = shapes.get().unwrap();
    let first = shapes
        .add_shape(&support::make_box(1.0, 1.0, 1.0), true, true)
        .unwrap();
    let second = shapes
        .add_shape(&support::make_box(2.0, 2.0, 2.0), true, true)
        .unwrap();
    (first, second)
}

fn values(items: &[f64]) -> Handle<HArray1<f64>> {
    let array = HArray1::<f64>::with_bounds(1, items.len() as i32).unwrap();
    array.change_array1().as_mut_slice().copy_from_slice(items);
    Handle::new(array)
}

#[test]
fn dim_tol_tool_legacy_entries() {
    let document = document();
    let (part, _) = two_parts(&document);
    let tool = XCAFDocDocumentTool::dim_tol_tool(&document.main().unwrap()).unwrap();
    let tool = tool.get().unwrap();
    let name = Handle::new(HAsciiString::new("D1"));
    let description = Handle::new(HAsciiString::new("a dimension"));
    let entry = tool
        .set_dim_tol_values(&part, 5, &values(&[1.0, 2.0]), &name, &description)
        .unwrap();
    assert!(tool.is_dim_tol(&entry).unwrap());
    let found = tool
        .find_dim_tol(5, &values(&[1.0, 2.0]), &name, &description)
        .unwrap();
    assert_eq!(found, Some(entry.clone()));
    // Another string object with the same text is not the same name.
    let other = Handle::new(HAsciiString::new("D1"));
    assert_eq!(
        tool.find_dim_tol(5, &values(&[1.0, 2.0]), &other, &description)
            .unwrap(),
        None
    );
    let (mut firsts, mut seconds) = (Vec::new(), Vec::new());
    assert!(XCAFDocDimTolTool::get_ref_shape_label(&entry, &mut firsts, &mut seconds).unwrap());
    assert_eq!(firsts, vec![part]);
    assert!(seconds.is_empty());
    let (kind, stored, _, _) = tool.get_dim_tol(&entry).unwrap().unwrap();
    assert_eq!(kind, 5);
    assert_eq!(stored.get().unwrap().array1().as_slice(), &[1.0, 2.0]);
    assert_eq!(tool.get_dim_tol_labels().unwrap(), vec![entry]);
}

#[test]
fn dim_tol_tool_links_and_locks() {
    let document = document();
    let (first, second) = two_parts(&document);
    let tool = XCAFDocDocumentTool::dim_tol_tool(&document.main().unwrap()).unwrap();
    let tool = tool.get().unwrap();
    let dimension = tool.add_dimension().unwrap();
    tool.set_dimension_pair(&first, &second, &dimension)
        .unwrap();
    let mut dimensions = Vec::new();
    assert!(
        tool.get_ref_dimension_labels(&second, &mut dimensions)
            .unwrap()
    );
    assert_eq!(dimensions, vec![dimension.clone()]);
    let tolerance = tool.add_geom_tolerance().unwrap();
    tool.set_geom_tolerance(&first, &tolerance).unwrap();
    let datum = tool.add_datum().unwrap();
    tool.set_datum_shapes(std::slice::from_ref(&first), &datum)
        .unwrap();
    tool.set_datum_to_geom_tol(&datum, &tolerance).unwrap();
    let mut tolerances = Vec::new();
    assert!(
        tool.get_toler_of_datum_labels(&datum, &mut tolerances)
            .unwrap()
    );
    assert_eq!(tolerances, vec![tolerance.clone()]);
    let mut datums = Vec::new();
    assert!(tool.get_ref_datum_label(&first, &mut datums).unwrap());
    assert_eq!(datums, vec![datum]);
    assert!(!tool.is_locked(&dimension).unwrap());
    tool.lock(&dimension).unwrap();
    assert!(tool.is_locked(&dimension).unwrap());
    tool.unlock(&dimension).unwrap();
    assert!(!tool.is_locked(&dimension).unwrap());
    // Relinking a dimension drops its previous shapes.
    tool.set_dimension(&second, &dimension).unwrap();
    let mut on_first = Vec::new();
    assert!(
        !tool
            .get_ref_dimension_labels(&first, &mut on_first)
            .unwrap()
    );
}

#[test]
fn dim_tol_objects_tool_reads_the_objects() {
    let document = document();
    let (first, _) = two_parts(&document);
    let tool = XCAFDocDocumentTool::dim_tol_tool(&document.main().unwrap()).unwrap();
    let tool = tool.get().unwrap();
    let dimension = tool.add_dimension().unwrap();
    tool.set_dimension(&first, &dimension).unwrap();
    let tolerance = tool.add_geom_tolerance().unwrap();
    tool.set_geom_tolerance(&first, &tolerance).unwrap();
    let datum = tool.add_datum().unwrap();
    tool.set_datum_to_geom_tol(&datum, &tolerance).unwrap();
    let objects = XCAFDimTolObjectsTool::new(&document).unwrap();
    assert_eq!(objects.get_dimensions().unwrap().len(), 1);
    let (mut datum_objects, mut map) = (Vec::new(), Vec::new());
    let tolerances = objects
        .get_geom_tolerances(&mut datum_objects, &mut map)
        .unwrap();
    assert_eq!(
        (tolerances.len(), datum_objects.len(), map.len()),
        (1, 1, 1)
    );
    let shape = support::make_box(1.0, 1.0, 1.0);
    // The shape is not in the document (a new box), so it has no GD&T.
    assert!(objects.get_ref_dimensions(&shape).unwrap().is_none());
}

#[test]
fn view_round_trips_its_object() {
    let document = document();
    let main = document.main().unwrap();
    let tool = XCAFDocDocumentTool::view_tool(&main).unwrap();
    let tool = tool.get().unwrap();
    let view = tool.add_view().unwrap();
    assert!(tool.is_view(&view).unwrap());
    let object = XCAFViewObject::new();
    object.set_name(&Handle::new(HAsciiString::new("front")));
    object.set_type(XCAFViewProjectionType::Central);
    object.set_projection_point(&Point3::new(1.0, 2.0, 3.0));
    object.set_view_direction(&Direction3::new(0.0, 1.0, 0.0).unwrap());
    object.set_zoom_factor(2.0);
    object.set_front_plane_distance(5.0);
    object.create_gdt_points(2);
    object.set_gdt_point(1, &Point3::new(1.0, 0.0, 0.0));
    object.set_gdt_point(2, &Point3::new(0.0, 1.0, 0.0));
    let attribute = XCAFDocView::set(&view).unwrap();
    attribute
        .get()
        .unwrap()
        .set_object(&Handle::new(object))
        .unwrap();
    let back = attribute.get().unwrap().get_object().unwrap();
    let back = back.get().unwrap();
    assert_eq!(back.name().get().unwrap().string().as_bytes(), b"front");
    assert_eq!(back.kind(), XCAFViewProjectionType::Central);
    assert!(
        back.projection_point()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), 0.0)
    );
    assert_eq!(back.zoom_factor(), 2.0);
    assert!(back.has_front_plane_clipping());
    assert!(!back.has_back_plane_clipping());
    assert_eq!(back.nb_gdt_points(), 2);
}

#[test]
fn view_tool_links() {
    let document = document();
    let (first, second) = two_parts(&document);
    let main = document.main().unwrap();
    let tool = XCAFDocDocumentTool::view_tool(&main).unwrap();
    let tool = tool.get().unwrap();
    let view = tool.add_view().unwrap();
    tool.set_view_shapes(&[first.clone(), second.clone()], &[], &view)
        .unwrap();
    assert_eq!(
        tool.get_ref_shape_label(&view).unwrap(),
        Some(vec![first.clone(), second])
    );
    let mut views = Vec::new();
    assert!(tool.get_view_labels_for_shape(&first, &mut views).unwrap());
    assert_eq!(views, vec![view.clone()]);
    tool.set_view(
        &XCAFDocViewLinks {
            shapes: vec![first.clone()],
            ..Default::default()
        },
        &view,
    )
    .unwrap();
    assert_eq!(tool.get_ref_shape_label(&view).unwrap(), Some(vec![first]));
    assert_eq!(tool.get_view_labels().unwrap(), vec![view.clone()]);
    tool.remove_view(&view).unwrap();
    assert!(tool.get_view_labels().unwrap().is_empty());
}

#[test]
fn note_object_and_orphans() {
    let document = document();
    let main = document.main().unwrap();
    let notes = XCAFDocDocumentTool::notes_tool(&main).unwrap();
    let notes = notes.get().unwrap();
    let text = ExtendedString::from_utf8;
    let note = notes
        .create_comment(&text("me"), &text("today"), &text("a note"))
        .unwrap();
    let label = note.get().unwrap().label();
    let object = XCAFNoteObjectsNoteObject::new();
    object.set_point(&Point3::new(1.0, 1.0, 1.0));
    object.set_point_text(&Point3::new(2.0, 2.0, 2.0));
    note.get()
        .unwrap()
        .set_object(&Handle::new(object))
        .unwrap();
    let back = note.get().unwrap().get_object().unwrap();
    assert!(back.get().unwrap().has_point() && back.get().unwrap().has_point_text());
    assert!(!back.get().unwrap().has_plane());
    assert!(note.get().unwrap().is_orphan().unwrap());
    assert!(!xcaf_doc_note_get(&label).unwrap().is_null());
    assert_eq!(notes.nb_orphan_notes().unwrap(), 1);
}

#[test]
fn assembly_item_id_and_reference() {
    let id = XCAFDocAssemblyItemId::from_string(&AsciiString::from("0:1:1:1/0:1:1:2"));
    assert_eq!(id.get_path().len(), 2);
    assert!(
        id.is_child(&XCAFDocAssemblyItemId::from_string(&AsciiString::from(
            "0:1:1:1"
        )))
    );
    assert!(XCAFDocAssemblyItemId::new().to_string_path().is_err());
    let document = document();
    let (part, _) = two_parts(&document);
    let mut entry = AsciiString::default();
    rustycad_tklcaf::tdf::TdfTool::entry(&part, &mut entry);
    let label = document.main().unwrap().find_child(77, true).unwrap();
    let reference =
        XCAFDocAssemblyItemRef::set(&label, &XCAFDocAssemblyItemId::from_string(&entry)).unwrap();
    let reference = reference.get().unwrap();
    assert!(!reference.is_orphan().unwrap());
    let guid = Guid::from_ascii(b"27927843-72A0-41ef-901D-621FAB01C27A").unwrap();
    reference.set_guid(&guid).unwrap();
    assert!(reference.is_guid() && reference.get_guid() == guid);
    // The part has no attribute of that GUID.
    assert!(reference.is_orphan().unwrap());
    reference.set_subshape_index(1).unwrap();
    assert_eq!(reference.get_subshape_index(), 1);
    assert!(!reference.is_orphan().unwrap());
    reference.clear_extra_ref().unwrap();
    assert!(!reference.has_extra_ref());
}
