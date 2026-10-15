//! Port tests of the XCAFDoc tools: the document tool, colors, layers, materials, clipping
//! planes, visualization materials and AttributeInfo.
mod support;

use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkxcaf::xcaf_doc::{
    XCAFDoc, XCAFDocColorTool, XCAFDocColorType, XCAFDocDocumentTool, XCAFDocLayerTool,
    XCAFDocMaterialFields, XCAFDocMaterialTool, XCAFDocVisMaterial, XCAFDocVisMaterialPbr,
};

fn text(value: &str) -> ExtendedString {
    ExtendedString::from_utf8(value)
}

fn rgb(name: NamedColor) -> Color {
    Color::named(name)
}

fn document() -> Handle<TDocStdDocument> {
    let (_application, document) = support::new_document("BinXCAF");
    document
}

fn main_label(document: &Handle<TDocStdDocument>) -> TdfLabel {
    document.main().unwrap()
}

#[test]
fn document_tool_creates_named_tool_labels() {
    let document = document();
    let main = main_label(&document);
    let tool = XCAFDocDocumentTool::set(&main, true).unwrap();
    assert!(!tool.is_null());
    assert!(XCAFDocDocumentTool::is_xcaf_document(&document).unwrap());
    assert_eq!(
        XCAFDocDocumentTool::doc_label(&main).unwrap(),
        main.root().unwrap().find_child(1, false).unwrap()
    );
    let shapes = XCAFDocDocumentTool::shapes_label(&main).unwrap();
    let mut name: Handle<TDataStdName> = Handle::default();
    assert!(
        shapes
            .find_attribute_as(&TDataStdName::get_id(), &mut name)
            .unwrap()
    );
    assert_eq!(name.get().unwrap().get().to_utf8(), "Shapes");
    assert_eq!(XCAFDocDocumentTool::dgts_label(&main).unwrap().tag(), 4);
    assert_eq!(
        XCAFDocDocumentTool::vis_material_label(&main)
            .unwrap()
            .tag(),
        10
    );
    for check in [
        XCAFDocDocumentTool::check_shape_tool,
        XCAFDocDocumentTool::check_color_tool,
        XCAFDocDocumentTool::check_layer_tool,
        XCAFDocDocumentTool::check_dim_tol_tool,
        XCAFDocDocumentTool::check_material_tool,
        XCAFDocDocumentTool::check_notes_tool,
        XCAFDocDocumentTool::check_view_tool,
        XCAFDocDocumentTool::check_clipping_plane_tool,
    ] {
        assert!(check(&main).unwrap());
    }
    // The visualization material tool is created on request only.
    assert!(!XCAFDocDocumentTool::check_vis_material_tool(&main).unwrap());
    XCAFDocDocumentTool::vis_material_tool(&main).unwrap();
    assert!(XCAFDocDocumentTool::check_vis_material_tool(&main).unwrap());
}

#[test]
fn document_tool_length_unit_round_trips() {
    let document = document();
    let mut value = 0.0;
    assert!(!XCAFDocDocumentTool::get_length_unit(&document, &mut value).unwrap());
    XCAFDocDocumentTool::set_length_unit(&document, 0.001).unwrap();
    assert!(XCAFDocDocumentTool::get_length_unit(&document, &mut value).unwrap());
    assert_eq!(value, 0.001);
}

#[test]
fn color_tool_adds_finds_and_links_colors() {
    let document = document();
    let main = main_label(&document);
    let colors = XCAFDocDocumentTool::color_tool(&main).unwrap();
    let colors = colors.get().unwrap();
    let red = colors.add_color_rgb(&rgb(NamedColor::RED)).unwrap();
    assert_eq!(colors.add_color_rgb(&rgb(NamedColor::RED)).unwrap(), red);
    assert_eq!(
        colors.find_color_rgb_label(&rgb(NamedColor::RED)).unwrap(),
        red
    );
    assert!(
        colors
            .find_color_rgb_label(&rgb(NamedColor::BLUE))
            .unwrap()
            .is_null()
    );
    assert!(colors.is_color(&red).unwrap());
    assert_eq!(colors.get_colors().unwrap(), vec![red.clone()]);
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let solid = support::make_box(1.0, 2.0, 3.0);
    let part = shapes.get().unwrap().add_shape(&solid, true, true).unwrap();
    colors
        .set_color_label(&part, &red, XCAFDocColorType::ColorSurf)
        .unwrap();
    assert!(colors.is_set(&part, XCAFDocColorType::ColorSurf).unwrap());
    assert!(!colors.is_set(&part, XCAFDocColorType::ColorGen).unwrap());
    let found = XCAFDocColorTool::get_label_color_rgb(&part, XCAFDocColorType::ColorSurf).unwrap();
    assert!(found.unwrap().is_equal(rgb(NamedColor::RED)));
    assert!(
        colors
            .is_shape_set(&solid, XCAFDocColorType::ColorSurf)
            .unwrap()
    );
    colors
        .unset_color(&part, XCAFDocColorType::ColorSurf)
        .unwrap();
    assert!(!colors.is_set(&part, XCAFDocColorType::ColorSurf).unwrap());
}

#[test]
fn color_tool_visibility_and_auto_naming() {
    let document = document();
    let main = main_label(&document);
    let colors = XCAFDocDocumentTool::color_tool(&main).unwrap();
    let colors = colors.get().unwrap();
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let part = shapes
        .get()
        .unwrap()
        .add_shape(&support::make_box(1.0, 1.0, 1.0), true, true)
        .unwrap();
    assert!(XCAFDocColorTool::is_visible(&part).unwrap());
    colors.set_visibility(&part, false).unwrap();
    assert!(!XCAFDocColorTool::is_visible(&part).unwrap());
    colors.set_visibility(&part, true).unwrap();
    assert!(XCAFDocColorTool::is_visible(&part).unwrap());
    // Flags go on shape labels only.
    let free = main.find_child(42, true).unwrap();
    colors.set_visibility(&free, false).unwrap();
    assert!(XCAFDocColorTool::is_visible(&free).unwrap());
    colors.set_color_by_layer(&part, true).unwrap();
    assert!(colors.is_color_by_layer(&part).unwrap());
    let blue = colors
        .add_color(&ColorRgba::from_rgb(rgb(NamedColor::BLUE1), 0.5))
        .unwrap();
    let mut name: Handle<TDataStdName> = Handle::default();
    assert!(
        blue.find_attribute_as(&TDataStdName::get_id(), &mut name)
            .unwrap()
    );
    assert!(name.get().unwrap().get().to_utf8().starts_with("BLUE"));
}

#[test]
fn layer_tool_layers_and_links() {
    let document = document();
    let main = main_label(&document);
    let layers = XCAFDocDocumentTool::layer_tool(&main).unwrap();
    let layers = layers.get().unwrap();
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let solid = support::make_box(1.0, 1.0, 1.0);
    let part = shapes.get().unwrap().add_shape(&solid, true, true).unwrap();
    let first = layers.add_layer(&text("first")).unwrap();
    assert_eq!(layers.add_layer(&text("first")).unwrap(), first);
    assert!(layers.is_layer(&first).unwrap());
    layers.set_layer(&part, &text("first"), false).unwrap();
    layers.set_layer(&part, &text("second"), false).unwrap();
    assert!(layers.is_set(&part, &text("second")).unwrap());
    assert_eq!(XCAFDocLayerTool::get_layers(&part).unwrap().len(), 2);
    assert_eq!(
        XCAFDocLayerTool::get_shapes_of_layer(&first).unwrap(),
        vec![part.clone()]
    );
    assert!(layers.unset_one_layer(&part, &text("first")).unwrap());
    assert!(!layers.is_set_label(&part, &first).unwrap());
    layers
        .set_shape_layer(&solid, &text("third"), true)
        .unwrap();
    assert_eq!(
        layers.get_shape_layers(&solid).unwrap(),
        vec![text("third")]
    );
    layers.set_visibility(&first, false).unwrap();
    assert!(!layers.is_visible(&first).unwrap());
    assert!(
        layers
            .find_layer_with(&text("first"), true, true)
            .unwrap()
            .is_null()
    );
    assert_eq!(
        layers.add_layer_visible(&text("first"), false).unwrap(),
        first
    );
    layers.unset_layers(&part).unwrap();
    assert!(XCAFDocLayerTool::get_layers(&part).unwrap().is_empty());
    assert_eq!(layers.get_layer_labels().unwrap().len(), 3);
}

#[test]
fn material_tool_materials_and_density() {
    let document = document();
    let main = main_label(&document);
    let materials = XCAFDocDocumentTool::material_tool(&main).unwrap();
    let materials = materials.get().unwrap();
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let part = shapes
        .get()
        .unwrap()
        .add_shape(&support::make_box(1.0, 1.0, 1.0), true, true)
        .unwrap();
    assert_eq!(
        XCAFDocMaterialTool::get_density_for_shape(&part).unwrap(),
        0.0
    );
    let fields = XCAFDocMaterialFields {
        name: Handle::new(HAsciiString::new("steel")),
        description: Handle::new(HAsciiString::new("alloy")),
        density: 7800.0,
        dens_name: Handle::new(HAsciiString::new("density")),
        dens_val_type: Handle::new(HAsciiString::new("kg/m3")),
    };
    materials.set_material(&part, &fields).unwrap();
    assert_eq!(
        XCAFDocMaterialTool::get_density_for_shape(&part).unwrap(),
        7800.0 * 0.001
    );
    let labels = materials.get_material_labels().unwrap();
    assert_eq!(labels.len(), 1);
    let stored = XCAFDocMaterialTool::get_material(&labels[0])
        .unwrap()
        .unwrap();
    assert_eq!(stored.density, 7800.0);
}

#[test]
fn clipping_plane_tool_planes_and_capping() {
    let document = document();
    let main = main_label(&document);
    let planes = XCAFDocDocumentTool::clipping_plane_tool(&main).unwrap();
    let planes = planes.get().unwrap();
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 1.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let label = planes
        .add_clipping_plane_capping(&plane, &text("cut"), true)
        .unwrap();
    assert_eq!(
        planes.add_clipping_plane(&plane, &text("cut")).unwrap(),
        label
    );
    assert!(planes.is_clipping_plane(&label).unwrap());
    let found = planes.get_clipping_plane(&label).unwrap().unwrap();
    assert_eq!(found.name.to_utf8(), "cut");
    assert!(found.capping);
    assert_eq!(planes.get_capping(&label).unwrap(), Some(true));
    planes.set_capping(&label, false).unwrap();
    assert_eq!(planes.get_capping(&label).unwrap(), Some(false));
    assert_eq!(planes.get_clipping_planes().unwrap(), vec![label.clone()]);
    assert!(planes.remove_clipping_plane(&label).unwrap());
    assert!(planes.get_clipping_planes().unwrap().is_empty());
}

#[test]
fn vis_material_tool_links_materials() {
    let document = document();
    let main = main_label(&document);
    let tool = XCAFDocDocumentTool::vis_material_tool(&main).unwrap();
    let tool = tool.get().unwrap();
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let solid = support::make_box(1.0, 1.0, 1.0);
    shapes.get().unwrap().add_shape(&solid, true, true).unwrap();
    let material = tool.add_material(&AsciiString::from("glass")).unwrap();
    assert!(tool.is_material(&material).unwrap());
    assert!(tool.set_material_of_shape(&solid, &material).unwrap());
    assert!(tool.is_set_material_of_shape(&solid).unwrap());
    let attached = tool.get_material_of_shape(&solid).unwrap();
    assert_eq!(attached.get().unwrap().label(), material);
    assert!(tool.unset_material_of_shape(&solid).unwrap());
    assert!(!tool.is_set_material_of_shape(&solid).unwrap());
    assert_eq!(tool.get_materials().unwrap(), vec![material]);
}

#[test]
fn vis_material_conversions() {
    let material = XCAFDocVisMaterial::new();
    assert!(material.is_empty());
    let white = material.base_color();
    assert!(white.rgb().is_equal(rgb(NamedColor::WHITE)));
    let pbr = XCAFDocVisMaterialPbr {
        metallic: 0.0,
        roughness: 0.5,
        ..Default::default()
    };
    material.set_pbr_material(&pbr).unwrap();
    let common = material.convert_to_common_material().unwrap();
    assert!(common.is_defined);
    assert_eq!(common.shininess, 0.5);
    assert!(material.convert_to_pbr_material().is_equal(&pbr));
    material.unset_pbr_material().unwrap();
    assert!(material.is_empty());
}

#[test]
fn attribute_info_describes_links_and_flags() {
    let document = document();
    let main = main_label(&document);
    let shapes = XCAFDocDocumentTool::shape_tool(&main).unwrap();
    let part = shapes
        .get()
        .unwrap()
        .add_shape(&support::make_box(1.0, 1.0, 1.0), true, true)
        .unwrap();
    let colors = XCAFDocDocumentTool::color_tool(&main).unwrap();
    colors
        .get()
        .unwrap()
        .set_color_rgb(&part, &rgb(NamedColor::RED), XCAFDocColorType::ColorGen)
        .unwrap();
    let mut link: Handle<rustycad_tklcaf::tdata_std::TDataStdTreeNode> = Handle::default();
    assert!(
        part.find_attribute_as(
            &XCAFDoc::color_ref_guid(XCAFDocColorType::ColorGen),
            &mut link
        )
        .unwrap()
    );
    let info = XCAFDoc::attribute_info(&link.cast()).unwrap();
    assert!(String::from_utf8_lossy(info.as_bytes()).starts_with("Generic Color Link ==> 0:1:2:"));
    let mut named: Handle<rustycad_tkcaf::tnaming::TNamingNamedShape> = Handle::default();
    assert!(
        part.find_attribute_as(
            &rustycad_tkcaf::tnaming::TNamingNamedShape::get_id(),
            &mut named
        )
        .unwrap()
    );
    assert_eq!(
        XCAFDoc::attribute_info(&named.cast()).unwrap().as_bytes(),
        b"TopoDS_TSolid"
    );
}

// TDataStd_Real::Set backs up through the attribute's own handle: a derived XCAFDoc_Area
// records a delta of its own type, which restores its value on undo.
#[test]
fn area_backs_up_as_itself() {
    use rustycad_tkxcaf::xcaf_doc::XCAFDocArea;
    let document = document();
    document.set_undo_limit(10).unwrap();
    let label = main_label(&document).find_child(5, true).unwrap();
    document.new_command().unwrap();
    XCAFDocArea::set(&label, 1.0).unwrap();
    document.commit_command().unwrap();
    document.new_command().unwrap();
    XCAFDocArea::set(&label, 2.0).unwrap();
    document.commit_command().unwrap();
    let undos = document.get_undos();
    let deltas = undos.last().unwrap().attribute_deltas();
    let name = deltas
        .last()
        .unwrap()
        .attribute()
        .dynamic_type()
        .unwrap()
        .name();
    assert_eq!(name, "XCAFDoc_Area");
    assert!(document.undo().unwrap());
    let mut area = 0.0;
    assert!(XCAFDocArea::get_of_label(&label, &mut area).unwrap());
    assert_eq!(area, 1.0);
}

// FillAspect: the material, alpha mode and culling of the aspect, and one XCAFPrs_Texture
// per texture of the material (the base color one on unit 0, a color map).
#[test]
fn vis_material_fills_aspects() {
    use rustycad_tkservice::graphic3d::{
        Graphic3dAlphaMode, Graphic3dAspects, Graphic3dAspectsMembers,
        Graphic3dAspectsRenderMembers, Graphic3dTextureRoot, Graphic3dTypeOfBackfacingModel,
    };
    use rustycad_tkservice::image::ImageTexture;
    use rustycad_tkxcaf::xcaf_prs::XCAFPrsTexture;
    let material = XCAFDocVisMaterial::new();
    let texture: Handle<dyn rustycad_tkservice::image::ImageTextureSource> =
        Handle::new(ImageTexture::new(&AsciiString::from("wood.png"))).cast();
    let pbr = XCAFDocVisMaterialPbr {
        base_color_texture: texture.clone(),
        ..Default::default()
    };
    material.set_pbr_material(&pbr).unwrap();
    material
        .set_alpha_mode(Graphic3dAlphaMode::Mask, 0.25)
        .unwrap();
    material.set_double_sided(true).unwrap();
    let aspects = Handle::new(Graphic3dAspects::default());
    material.fill_aspect(&aspects.clone().cast()).unwrap();
    let aspects = aspects.get().unwrap();
    assert_eq!(aspects.alpha_mode(), Graphic3dAlphaMode::Mask);
    assert_eq!(
        aspects.face_culling(),
        Graphic3dTypeOfBackfacingModel::DoubleSided
    );
    assert!(aspects.to_map_texture());
    let set = aspects.texture_set();
    let set = set.get().unwrap();
    assert_eq!(set.size(), 1);
    let first = set.first().unwrap();
    let first = first.cast::<XCAFPrsTexture>();
    let first = first.get().unwrap();
    assert!(first.texture_root_base().fields.read().is_color_map);
    assert_eq!(
        first.get_image_source().object_address(),
        texture.object_address()
    );
}
