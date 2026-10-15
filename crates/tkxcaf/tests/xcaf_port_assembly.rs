//! Port tests of the assemblies of an XDE document: the assembly iterator and graph, the
//! instance colors, the editor, the styles and explorer of XCAFPrs, the AIS object and the
//! XCAFApp application.
mod support;

use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;
use rustycad_tkxcaf::xcaf_app::XCAFAppApplication;
use rustycad_tkxcaf::xcaf_doc::{
    XCAFDocArea, XCAFDocAssemblyGraph, XCAFDocAssemblyGraphNodeType, XCAFDocAssemblyIterator,
    XCAFDocColorType, XCAFDocDocumentTool, XCAFDocEditor, XCAFDocLabelSet, XCAFDocShapeTool,
};
use rustycad_tkxcaf::xcaf_prs::{
    XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_NONE, XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_ONLY_LEAF_NODES,
    XCAFPrs, XCAFPrsAISObject, XCAFPrsDocumentExplorer, XCAFPrsStyle, XCAFPrsStyleMap,
};

/// A compound of a box and a moved box.
fn two_boxes() -> (Shape, Shape) {
    let moved = support::make_box(1.0, 1.0, 1.0)
        .moved(
            &Location::new(Transform3::translation([5.0, 0.0, 0.0])),
            true,
        )
        .unwrap();
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let mut compound = compound.into_shape();
    Builder
        .add(&mut compound, &support::make_box(1.0, 1.0, 1.0))
        .unwrap();
    Builder.add(&mut compound, &moved).unwrap();
    (compound, moved)
}

/// A document with the compound as an assembly: (document, assembly label, moved box).
fn assembly_document() -> (Handle<TDocStdDocument>, TdfLabel, Shape) {
    let (_application, document) = support::new_document("BinXCAF");
    let (compound, moved) = two_boxes();
    let shapes = XCAFDocDocumentTool::shape_tool(&document.main().unwrap()).unwrap();
    let assembly = shapes
        .get()
        .unwrap()
        .add_shape(&compound, true, true)
        .unwrap();
    (document, assembly, moved)
}

fn shape_tool(document: &Handle<TDocStdDocument>) -> Handle<XCAFDocShapeTool> {
    XCAFDocDocumentTool::shape_tool(&document.main().unwrap()).unwrap()
}

#[test]
fn assembly_iterator_and_graph() {
    let (document, assembly, _) = assembly_document();
    assert!(XCAFDocShapeTool::is_assembly(&assembly).unwrap());
    assert_eq!(
        XCAFDocShapeTool::nb_components(&assembly, false).unwrap(),
        2
    );
    let mut items = Vec::new();
    XCAFDocAssemblyIterator::new(&document, i32::MAX)
        .unwrap()
        .traverse(|item| {
            items.push(item.get_path().len());
            true
        })
        .unwrap();
    // The root, then its two components.
    assert_eq!(items, vec![1, 2, 2]);
    let graph = XCAFDocAssemblyGraph::new(&document).unwrap();
    let graph = graph.get().unwrap();
    // Two different boxes: one root, then an occurrence and its part for each.
    assert_eq!(graph.nb_nodes(), 5);
    assert_eq!(graph.nb_links(), 4);
    assert_eq!(
        graph.get_node_type(1),
        XCAFDocAssemblyGraphNodeType::AssemblyRoot
    );
    assert_eq!(
        graph.get_node_type(2),
        XCAFDocAssemblyGraphNodeType::Occurrence
    );
    assert_eq!(graph.get_node_type(3), XCAFDocAssemblyGraphNodeType::Part);
    assert_eq!(graph.nb_occurrences(3), 1);
    assert!(graph.is_direct_link(1, 2) && graph.is_direct_link(2, 3));
    assert!(!graph.is_direct_link(1, 3));
}

#[test]
fn instance_colors_of_components() {
    let (document, _, moved) = assembly_document();
    let colors = XCAFDocDocumentTool::color_tool(&document.main().unwrap()).unwrap();
    let colors = colors.get().unwrap();
    let red = Color::named(NamedColor::RED);
    assert!(
        colors
            .set_instance_color_rgb(&moved, XCAFDocColorType::ColorSurf, &red, true)
            .unwrap()
    );
    let found = colors
        .get_instance_color_rgb(&moved, XCAFDocColorType::ColorSurf)
        .unwrap();
    assert!(found.unwrap().is_equal(red));
    assert!(colors.is_instance_visible(&moved).unwrap());
    assert!(colors.reverse_chains_of_tree_nodes().unwrap());
}

#[test]
fn styles_of_an_assembly() {
    let (document, assembly, moved) = assembly_document();
    let colors = XCAFDocDocumentTool::color_tool(&document.main().unwrap()).unwrap();
    let colors = colors.get().unwrap();
    colors
        .set_instance_color_rgb(
            &moved,
            XCAFDocColorType::ColorGen,
            &Color::named(NamedColor::RED),
            true,
        )
        .unwrap();
    let mut settings = XCAFPrsStyleMap::default();
    let white = ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 1.0);
    XCAFPrs::collect_style_settings(&assembly, &Location::default(), &mut settings, &white)
        .unwrap();
    assert_eq!(settings.extent(), 1);
    let style = settings.find_from_index(1).unwrap();
    assert!(style.is_set_color_surf() && style.is_set_color_curv());
    assert!(
        style
            .get_color_surf()
            .is_equal(Color::named(NamedColor::RED))
    );
    let mut other = XCAFPrsStyle::new();
    assert!(other != *style);
    other.set_color_surf(Color::named(NamedColor::RED));
    other.set_color_curv(Color::named(NamedColor::RED));
    assert!(other == *style && other.hash_code() == style.hash_code());
}

#[test]
fn document_explorer_walks_the_tree() {
    let (document, _, _) = assembly_document();
    let mut explorer = XCAFPrsDocumentExplorer::with_document(
        &document,
        XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_NONE,
        &XCAFPrsStyle::new(),
    )
    .unwrap();
    let mut ids = Vec::new();
    while explorer.more() {
        ids.push(String::from_utf8_lossy(explorer.current().id.as_bytes()).into_owned());
        explorer.next().unwrap();
    }
    assert_eq!(ids.len(), 3);
    assert!(ids[1].starts_with(&ids[0]) && ids[1].contains('/'));
    let leaves = XCAFPrsDocumentExplorer::with_document(
        &document,
        XCAF_PRS_DOCUMENT_EXPLORER_FLAGS_ONLY_LEAF_NODES,
        &XCAFPrsStyle::new(),
    )
    .unwrap();
    assert!(!leaves.current().is_assembly);
    let shape =
        XCAFPrsDocumentExplorer::find_shape_from_path_id(&document, &ids[2].as_str().into())
            .unwrap();
    assert!(!shape.is_null());
}

#[test]
fn editor_expands_a_compound() {
    let (_application, document) = support::new_document("BinXCAF");
    let (compound, _) = two_boxes();
    let shapes = shape_tool(&document);
    let part = shapes
        .get()
        .unwrap()
        .add_shape(&compound, false, true)
        .unwrap();
    assert!(!XCAFDocShapeTool::is_assembly(&part).unwrap());
    let main = document.main().unwrap();
    assert!(XCAFDocEditor::expand(&main, true).unwrap());
    assert!(XCAFDocShapeTool::is_assembly(&part).unwrap());
    assert_eq!(XCAFDocShapeTool::nb_components(&part, false).unwrap(), 2);
}

#[test]
fn editor_extracts_and_filters() {
    let (document, assembly, _) = assembly_document();
    let mut components = Vec::new();
    XCAFDocShapeTool::get_components(&assembly, &mut components, false).unwrap();
    let (_other_application, target) = support::new_document("BinXCAF");
    let target_shapes = shape_tool(&target);
    let target_assembly = target_shapes.get().unwrap().new_shape().unwrap();
    assert!(XCAFDocEditor::extract_label(&components[0], &target_assembly, false).unwrap());
    assert_eq!(
        XCAFDocShapeTool::nb_components(&target_assembly, false).unwrap(),
        1
    );
    let mut keep = XCAFDocLabelSet::default();
    keep.add(components[0].clone());
    assert!(XCAFDocEditor::filter_shape_tree(&shape_tool(&document), &keep).unwrap());
    assert_eq!(
        XCAFDocShapeTool::nb_components(&assembly, false).unwrap(),
        1
    );
}

#[test]
fn editor_rescales_parts_and_locations() {
    let (document, assembly, _) = assembly_document();
    let mut components = Vec::new();
    XCAFDocShapeTool::get_components(&assembly, &mut components, false).unwrap();
    let mut part = TdfLabel::default();
    XCAFDocShapeTool::get_referred_shape(&components[1], &mut part).unwrap();
    XCAFDocArea::set(&part, 6.0).unwrap();
    assert!(XCAFDocEditor::rescale_geometry(&assembly, 2.0, false).unwrap());
    let mut area = 0.0;
    assert!(XCAFDocArea::get_of_label(&part, &mut area).unwrap());
    assert_eq!(area, 24.0);
    let location = XCAFDocShapeTool::get_location(&components[1]).unwrap();
    assert_eq!(location.transformation().translation_part().x(), 10.0);
    // Only roots rescale unless forced.
    assert!(!XCAFDocEditor::rescale_geometry(&part, 2.0, false).unwrap());
    let _ = document;
}

#[test]
fn ais_object_dispatches_styles() {
    let (document, assembly, moved) = assembly_document();
    let colors = XCAFDocDocumentTool::color_tool(&document.main().unwrap()).unwrap();
    colors
        .get()
        .unwrap()
        .set_instance_color_rgb(
            &moved,
            XCAFDocColorType::ColorGen,
            &Color::named(NamedColor::RED),
            true,
        )
        .unwrap();
    let object = XCAFPrsAISObject::new(&assembly).unwrap();
    let object = object.get().unwrap();
    assert_eq!(object.get_label(), assembly);
    object.dispatch_styles(false).unwrap();
    use rustycad_tkv3d::ais::AisColoredShapeMembers;
    assert_eq!(object.custom_aspects_map().len(), 1);
    assert!(object.default_style().is_set_color_surf());
}

#[test]
fn application_initializes_xde_documents() {
    let application = XCAFAppApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::from_utf8("BinXCAF"), &mut document)
        .unwrap();
    assert!(XCAFDocDocumentTool::is_xcaf_document(&document).unwrap());
    assert_eq!(application.resources_name(), "XCAF");
    assert!(
        XCAFAppApplication::get_application().object_address()
            == XCAFAppApplication::get_application().object_address()
    );
}
