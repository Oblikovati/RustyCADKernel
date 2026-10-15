// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Editor.cxx (CloneMetaData) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Editor::CloneMetaData`: the colors, layers, material, visualization material,
//! plain attributes and name of a shape label copied onto another label.
use super::support::find;
use super::{
    XCAFDoc, XCAFDocColorTool, XCAFDocColorType, XCAFDocDocumentTool, XCAFDocGraphNode,
    XCAFDocLocation, XCAFDocShapeMapTool, XCAFDocShapeTool, XCAFDocVisMaterial,
};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeIterator, TdfAttributeState, TdfLabel, TdfRelocationTable, TdfTool,
};

/// The copies of visualization materials between documents (by source object).
pub type XCAFDocVisMaterialMap = Vec<(Handle<XCAFDocVisMaterial>, Handle<XCAFDocVisMaterial>)>;

/// What `CloneMetaData` copies (the native defaults copy everything).
#[derive(Clone, Copy, Debug)]
pub struct XCAFDocCloneFlags {
    /// `theToCopyColor`.
    pub color: bool,
    /// `theToCopyLayer`.
    pub layer: bool,
    /// `theToCopyMaterial`.
    pub material: bool,
    /// `theToCopyVisMaterial`.
    pub vis_material: bool,
    /// `theToCopyAttributes`.
    pub attributes: bool,
}

impl Default for XCAFDocCloneFlags {
    fn default() -> Self {
        Self {
            color: true,
            layer: true,
            material: true,
            vis_material: true,
            attributes: true,
        }
    }
}

/// The colors and the visibility of the source label onto the destination label.
fn copy_colors(source: &TdfLabel, destination: &TdfLabel) -> Result<(), Failure> {
    let target = XCAFDocDocumentTool::color_tool(destination)?;
    let target = rustycad_tklcaf::live(&target, "XCAFDoc_Editor::CloneMetaData")?;
    for kind in [
        XCAFDocColorType::ColorGen,
        XCAFDocColorType::ColorSurf,
        XCAFDocColorType::ColorCurv,
    ] {
        let mut color_label = TdfLabel::default();
        XCAFDocColorTool::get_color_label(source, kind, &mut color_label)?;
        if !color_label.is_null() {
            let color = XCAFDocColorTool::get_color_rgba(&color_label)?.unwrap_or_default();
            target.set_color(destination, &color, kind)?;
        }
    }
    target.set_visibility(destination, XCAFDocColorTool::is_visible(source)?)
}

/// The layers of the source label onto the destination label (by name).
fn copy_layers(source: &TdfLabel, destination: &TdfLabel) -> Result<(), Failure> {
    let source_tool = XCAFDocDocumentTool::layer_tool(source)?;
    let target = XCAFDocDocumentTool::layer_tool(destination)?;
    let (source_tool, target) = (
        rustycad_tklcaf::live(&source_tool, "CloneMetaData")?,
        rustycad_tklcaf::live(&target, "CloneMetaData")?,
    );
    for layer in super::XCAFDocLayerTool::get_layer_labels_of(source)? {
        let name = source_tool.get_layer(&layer)?.unwrap_or_default();
        target.set_layer(destination, &name, false)?;
    }
    Ok(())
}

/// The material of the source label (when it has a name) onto the destination label.
fn copy_material(source: &TdfLabel, destination: &TdfLabel) -> Result<(), Failure> {
    let node = find::<TDataStdTreeNode>(source, &XCAFDoc::material_ref_guid())?;
    let Some(node) = node.get().filter(|n| n.has_father()) else {
        return Ok(());
    };
    let material = rustycad_tklcaf::live(&node.father(), "CloneMetaData")?.label();
    if material.is_null() {
        return Ok(());
    }
    XCAFDocDocumentTool::material_tool(source)?;
    let target = XCAFDocDocumentTool::material_tool(destination)?;
    let Some(fields) = super::XCAFDocMaterialTool::get_material(&material)? else {
        return Ok(());
    };
    if fields
        .name
        .get()
        .is_some_and(|name| name.string().length() != 0)
    {
        rustycad_tklcaf::live(&target, "CloneMetaData")?.set_material(destination, &fields)?;
    }
    Ok(())
}

/// The copy of a visualization material in the destination document (shared through the
/// map), or the source itself without a map (the same document).
fn vis_material_copy(
    source: &Handle<XCAFDocVisMaterial>,
    destination: &TdfLabel,
    map: Option<&mut XCAFDocVisMaterialMap>,
) -> Result<Handle<XCAFDocVisMaterial>, Failure> {
    let Some(map) = map else {
        return Ok(source.clone()); // consider the same document
    };
    if let Some((_, copy)) = map
        .iter()
        .find(|(known, _)| known.object_address() == source.object_address())
    {
        return Ok(copy.clone());
    }
    let original = rustycad_tklcaf::live(source, "CloneMetaData")?;
    let copy = Handle::new(XCAFDocVisMaterial::new());
    let target = rustycad_tklcaf::live(&copy, "CloneMetaData")?;
    target.set_common_material(&original.common_material())?;
    target.set_pbr_material(&original.pbr_material())?;
    target.set_alpha_mode(original.alpha_mode(), original.alpha_cut_off())?;
    target.set_face_culling(original.face_culling())?;
    let name = find::<TDataStdName>(&original.label(), &TDataStdName::get_id())?;
    let name = name
        .get()
        .map(|n| AsciiString::from_extended(&n.get(), None))
        .unwrap_or_default();
    let tool = XCAFDocDocumentTool::vis_material_tool(destination)?;
    rustycad_tklcaf::live(&tool, "CloneMetaData")?.add_material_attribute(&copy, &name)?;
    map.push((source.clone(), copy.clone()));
    Ok(copy)
}

/// The visualization material of the source label onto the destination label, or its base
/// color when materials are not copied.
fn copy_vis_material(
    source: &TdfLabel,
    destination: &TdfLabel,
    map: Option<&mut XCAFDocVisMaterialMap>,
    copy_material: bool,
) -> Result<(), Failure> {
    XCAFDocDocumentTool::vis_material_tool(source)?;
    let Some(material_label) = super::XCAFDocVisMaterialTool::get_shape_material_label(source)?
    else {
        return Ok(());
    };
    let source_material = super::XCAFDocVisMaterialTool::get_material(&material_label)?;
    if !copy_material {
        let colors = XCAFDocDocumentTool::color_tool(destination)?;
        let base = rustycad_tklcaf::live(&source_material, "CloneMetaData")?.base_color();
        return rustycad_tklcaf::live(&colors, "CloneMetaData")?.set_color(
            destination,
            &base,
            XCAFDocColorType::ColorGen,
        );
    }
    let tool = XCAFDocDocumentTool::vis_material_tool(destination)?;
    let copy = vis_material_copy(&source_material, destination, map)?;
    let copy_label = rustycad_tklcaf::live(&copy, "CloneMetaData")?.label();
    rustycad_tklcaf::live(&tool, "CloneMetaData")?.set_shape_material(destination, &copy_label)
}

/// The attribute is skipped by the plain copy (tree and graph nodes, named shapes, shape
/// maps and locations).
fn is_structural(attribute: &Handle<dyn TdfAttribute>) -> bool {
    !attribute.cast::<TDataStdTreeNode>().is_null()
        || !attribute.cast::<XCAFDocGraphNode>().is_null()
        || !attribute.cast::<TNamingNamedShape>().is_null()
        || !attribute.cast::<XCAFDocShapeMapTool>().is_null()
        || !attribute.cast::<XCAFDocLocation>().is_null()
}

/// The plain attributes of the source label pasted onto (new or existing) attributes of
/// the destination label.
fn copy_attributes(source: &TdfLabel, destination: &TdfLabel) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::new(source, true)?;
    while iterator.more() {
        let attribute = iterator.value();
        iterator.next();
        if is_structural(&attribute) {
            continue;
        }
        let original = rustycad_tklcaf::live(&attribute, "CloneMetaData")?;
        let mut target: Handle<dyn TdfAttribute> = Handle::default();
        if !destination.find_attribute(&original.id(), &mut target)? {
            target = original.new_empty();
            destination.add_attribute(&target, false)?;
        }
        original.paste(&target, &Handle::new(TdfRelocationTable::new(false)))?;
    }
    Ok(())
}

/// The name of the source label onto the destination label; a reference name (`=>...`)
/// becomes the reference to the destination's referred label.
fn copy_name(source: &TdfLabel, destination: &TdfLabel) -> Result<(), Failure> {
    let name = find::<TDataStdName>(source, &TDataStdName::get_id())?;
    XCAFDocDocumentTool::shape_tool(destination)?;
    let Some(name) = name.get() else {
        return Ok(());
    };
    let text = name.get();
    if text.search(ExtendedString::from_utf8("=>").as_utf16()) < 0 {
        TDataStdName::set(destination, &text)?;
        return Ok(());
    }
    let mut referred = TdfLabel::default();
    if XCAFDocShapeTool::get_referred_shape(destination, &mut referred)? {
        let mut entry = AsciiString::default();
        TdfTool::entry(&referred, &mut entry);
        let reference = format!("=>{}", String::from_utf8_lossy(entry.as_bytes()));
        TDataStdName::set(destination, &ExtendedString::from_utf8(&reference))?;
    }
    Ok(())
}

impl super::XCAFDocEditor {
    /// `CloneMetaData(theSrcLabel, theDstLabel, theVisMatMap, theToCopyColor, ...)`: copies
    /// the metadata of a label onto another (nothing for the same or a null label). The
    /// visualization material follows `color` or `vis_material`, as natively.
    pub fn clone_meta_data(
        source: &TdfLabel,
        destination: &TdfLabel,
        map: Option<&mut XCAFDocVisMaterialMap>,
        flags: XCAFDocCloneFlags,
    ) -> Result<(), Failure> {
        if source == destination || source.is_null() || destination.is_null() {
            return Ok(());
        }
        let copy_color = flags.color && XCAFDocDocumentTool::check_color_tool(source)?;
        let copy_layer = flags.layer && XCAFDocDocumentTool::check_layer_tool(source)?;
        let copy_mat = flags.material && XCAFDocDocumentTool::check_material_tool(source)?;
        let copy_vis = XCAFDocDocumentTool::check_vis_material_tool(source)?;
        if copy_color {
            XCAFDocDocumentTool::color_tool(source)?;
            copy_colors(source, destination)?;
        }
        if copy_layer {
            copy_layers(source, destination)?;
        }
        if copy_mat {
            copy_material(source, destination)?;
        }
        if copy_vis && (flags.vis_material || copy_color) {
            copy_vis_material(source, destination, map, flags.vis_material)?;
        }
        if flags.attributes {
            copy_attributes(source, destination)?;
        }
        copy_name(source, destination)
    }
}
