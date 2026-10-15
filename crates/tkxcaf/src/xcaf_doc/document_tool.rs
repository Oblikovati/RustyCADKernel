// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_DocumentTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_DocumentTool`: the root of an XDE document. Its label (the first child of the
//! root, unless another label is registered through a tree node of the root) holds the
//! labels of the shapes, colors, layers, GD&T, materials, views, clipping planes, notes and
//! visualization materials, each with its tool.
use super::generic_empty_tool::generic_empty_attribute;
use super::support::find;
use super::{
    XCAFDocClippingPlaneTool, XCAFDocColorTool, XCAFDocDimTolTool, XCAFDocLayerTool,
    XCAFDocLengthUnit, XCAFDocMaterialTool, XCAFDocNotesTool, XCAFDocShapeTool, XCAFDocViewTool,
    XCAFDocVisMaterialTool,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tkernel::units_methods::{LengthUnit, UnitsMethods};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use std::io;

/// The file-local `GetDocumentToolRefID()`: the tree nodes that register the document label.
const DOCUMENT_TOOL_REF_ID: Guid = guid_from_text("efd212eb-6dfd-11d4-b9c8-0060b0ee281b");

generic_empty_attribute!(
    /// `XCAFDoc_DocumentTool`.
    XCAFDocDocumentTool, "XCAFDoc_DocumentTool", "efd212ec-6dfd-11d4-b9c8-0060b0ee281b", state: (),
    dump: |tool: &XCAFDocDocumentTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        // No override natively: the `TDataStd_GenericEmpty` dump.
        rustycad_tklcaf::tdata_std::dump_generic_empty(tool, stream, depth)
    },
    after_retrieval: |tool: &XCAFDocDocumentTool, _force_it: bool| -> Result<bool, Failure> {
        tool.init()?;
        Ok(true)
    }
);

impl XCAFDocDocumentTool {
    /// `Set(L, IsAcces)`: the document tool, created (on the document label, or on `L` when
    /// not `IsAcces`) with every other tool when absent.
    pub fn set(label: &TdfLabel, is_access: bool) -> Result<Handle<XCAFDocDocumentTool>, Failure> {
        let mut doc_label = Self::doc_label(label)?;
        let existing = find::<XCAFDocDocumentTool>(&doc_label, &Self::get_id())?;
        if !existing.is_null() {
            return Ok(existing);
        }
        if !is_access {
            doc_label = label.clone();
        }
        let tool = Handle::new(Self::new());
        doc_label.add_attribute(&tool.clone().cast(), false)?;
        rustycad_tklcaf::live(&tool, "XCAFDoc_DocumentTool::Set")?.init()?;
        create_tools(label)?;
        Ok(tool)
    }
    /// `DocLabel(acces)`: the registered document label (the first node of the root's tree
    /// node), else the first child of the root.
    pub fn doc_label(access: &TdfLabel) -> Result<TdfLabel, Failure> {
        let root = access.root()?;
        let node = find::<TDataStdTreeNode>(&root, &DOCUMENT_TOOL_REF_ID)?;
        if let Some(node) = node.get() {
            let first = node.first();
            return Ok(rustycad_tklcaf::live(&first, "XCAFDoc_DocumentTool::DocLabel")?.label());
        }
        root.find_child(1, true)
    }
    /// `Init`: registers the label as the document label of the root (once).
    pub fn init(&self) -> Result<(), Failure> {
        let label = self.label();
        let root = label.root()?;
        if root.is_attribute(&DOCUMENT_TOOL_REF_ID)? {
            return Ok(());
        }
        let root_node = TDataStdTreeNode::set_with_id(&root, &DOCUMENT_TOOL_REF_ID)?;
        let label_node = TDataStdTreeNode::set_with_id(&label, &DOCUMENT_TOOL_REF_ID)?;
        rustycad_tklcaf::live(&label_node, "XCAFDoc_DocumentTool::Init")?.set_father(&root_node)?;
        rustycad_tklcaf::live(&root_node, "XCAFDoc_DocumentTool::Init")?.set_first(&label_node)
    }
    /// `IsXCAFDocument(D)`: the root registers a document label.
    pub fn is_xcaf_document(document: &Handle<TDocStdDocument>) -> Result<bool, Failure> {
        let main =
            rustycad_tklcaf::live(document, "XCAFDoc_DocumentTool::IsXCAFDocument")?.main()?;
        main.root()?.is_attribute(&DOCUMENT_TOOL_REF_ID)
    }
}

/// The tools `Set` creates, in native order.
fn create_tools(label: &TdfLabel) -> Result<(), Failure> {
    use XCAFDocDocumentTool as D;
    XCAFDocShapeTool::set(&D::shapes_label(label)?)?;
    XCAFDocColorTool::set(&D::colors_label(label)?)?;
    XCAFDocLayerTool::set(&D::layers_label(label)?)?;
    XCAFDocDimTolTool::set(&D::dgts_label(label)?)?;
    XCAFDocMaterialTool::set(&D::materials_label(label)?)?;
    XCAFDocNotesTool::set(&D::notes_label(label)?)?;
    XCAFDocViewTool::set(&D::views_label(label)?)?;
    XCAFDocClippingPlaneTool::set(&D::clipping_planes_label(label)?)?;
    Ok(())
}

/// Declares the labels of the document: the child of the document label with that tag,
/// named on each call.
macro_rules! document_labels {
    ($($(#[$doc:meta])* $name:ident => $tag:literal, $text:literal;)*) => {
        impl XCAFDocDocumentTool {
            $(
                $(#[$doc])*
                pub fn $name(access: &TdfLabel) -> Result<TdfLabel, Failure> {
                    let label = Self::doc_label(access)?.find_child($tag, true)?;
                    TDataStdName::set(&label, &ExtendedString::from_utf8($text))?;
                    Ok(label)
                }
            )*
        }
    };
}

document_labels! {
    /// `ShapesLabel`: child 1, `Shapes`.
    shapes_label => 1, "Shapes";
    /// `ColorsLabel`: child 2, `Colors`.
    colors_label => 2, "Colors";
    /// `LayersLabel`: child 3, `Layers`.
    layers_label => 3, "Layers";
    /// `DGTsLabel`: child 4, `D&GTs`.
    dgts_label => 4, "D&GTs";
    /// `MaterialsLabel`: child 5, `Materials`.
    materials_label => 5, "Materials";
    /// `ViewsLabel`: child 7, `Views`.
    views_label => 7, "Views";
    /// `ClippingPlanesLabel`: child 8, `Clipping Planes`.
    clipping_planes_label => 8, "Clipping Planes";
    /// `NotesLabel`: child 9, `Notes`.
    notes_label => 9, "Notes";
    /// `VisMaterialLabel`: child 10, `VisMaterials`.
    vis_material_label => 10, "VisMaterials";
}

/// The `CheckXxxTool` members: the child of that tag exists and holds the tool.
fn check_tool(access: &TdfLabel, tag: i32, id: &Guid) -> Result<bool, Failure> {
    let label = XCAFDocDocumentTool::doc_label(access)?.find_child(tag, false)?;
    if label.is_null() {
        return Ok(false);
    }
    label.is_attribute(id)
}

/// The tools of the document.
impl XCAFDocDocumentTool {
    /// `ShapeTool(acces)`.
    pub fn shape_tool(access: &TdfLabel) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        XCAFDocShapeTool::set(&Self::shapes_label(access)?)
    }
    /// `CheckShapeTool(theAcces)`.
    pub fn check_shape_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 1, &XCAFDocShapeTool::get_id())
    }
    /// `ColorTool(acces)`.
    pub fn color_tool(access: &TdfLabel) -> Result<Handle<XCAFDocColorTool>, Failure> {
        XCAFDocColorTool::set(&Self::colors_label(access)?)
    }
    /// `CheckColorTool(theAcces)`.
    pub fn check_color_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 2, &XCAFDocColorTool::get_id())
    }
    /// `VisMaterialTool(theLabel)`.
    pub fn vis_material_tool(access: &TdfLabel) -> Result<Handle<XCAFDocVisMaterialTool>, Failure> {
        XCAFDocVisMaterialTool::set(&Self::vis_material_label(access)?)
    }
    /// `CheckVisMaterialTool(theAcces)`.
    pub fn check_vis_material_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 10, &XCAFDocVisMaterialTool::get_id())
    }
    /// `LayerTool(acces)`.
    pub fn layer_tool(access: &TdfLabel) -> Result<Handle<XCAFDocLayerTool>, Failure> {
        XCAFDocLayerTool::set(&Self::layers_label(access)?)
    }
    /// `CheckLayerTool(theAcces)`.
    pub fn check_layer_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 3, &XCAFDocLayerTool::get_id())
    }
    /// `DimTolTool(acces)`.
    pub fn dim_tol_tool(access: &TdfLabel) -> Result<Handle<XCAFDocDimTolTool>, Failure> {
        XCAFDocDimTolTool::set(&Self::dgts_label(access)?)
    }
    /// `CheckDimTolTool(theAcces)`.
    pub fn check_dim_tol_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 4, &XCAFDocDimTolTool::get_id())
    }
    /// `MaterialTool(acces)`.
    pub fn material_tool(access: &TdfLabel) -> Result<Handle<XCAFDocMaterialTool>, Failure> {
        XCAFDocMaterialTool::set(&Self::materials_label(access)?)
    }
    /// `CheckMaterialTool(theAcces)`.
    pub fn check_material_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 5, &XCAFDocMaterialTool::get_id())
    }
    /// `ViewTool(acces)`.
    pub fn view_tool(access: &TdfLabel) -> Result<Handle<XCAFDocViewTool>, Failure> {
        XCAFDocViewTool::set(&Self::views_label(access)?)
    }
    /// `CheckViewTool(theAcces)`.
    pub fn check_view_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 7, &XCAFDocViewTool::get_id())
    }
    /// `ClippingPlaneTool(acces)`.
    pub fn clipping_plane_tool(
        access: &TdfLabel,
    ) -> Result<Handle<XCAFDocClippingPlaneTool>, Failure> {
        XCAFDocClippingPlaneTool::set(&Self::clipping_planes_label(access)?)
    }
    /// `CheckClippingPlaneTool(theAcces)`.
    pub fn check_clipping_plane_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 8, &XCAFDocClippingPlaneTool::get_id())
    }
    /// `NotesTool(acces)`.
    pub fn notes_tool(access: &TdfLabel) -> Result<Handle<XCAFDocNotesTool>, Failure> {
        XCAFDocNotesTool::set(&Self::notes_label(access)?)
    }
    /// `CheckNotesTool(theAcces)`.
    pub fn check_notes_tool(access: &TdfLabel) -> Result<bool, Failure> {
        check_tool(access, 9, &XCAFDocNotesTool::get_id())
    }
}

/// The length unit of a document.
impl XCAFDocDocumentTool {
    /// The length unit attribute of the document's root, or null.
    fn length_unit(
        document: &Handle<TDocStdDocument>,
    ) -> Result<Handle<XCAFDocLengthUnit>, Failure> {
        let main =
            rustycad_tklcaf::live(document, "XCAFDoc_DocumentTool::GetLengthUnit")?.main()?;
        find::<XCAFDocLengthUnit>(&main.root()?, &XCAFDocLengthUnit::get_id())
    }
    /// `GetLengthUnit(theDoc, theResult, theBaseUnit)`: the scale in the base unit; false
    /// for a null document or a document without unit.
    pub fn get_length_unit_in(
        document: &Handle<TDocStdDocument>,
        result: &mut f64,
        base: LengthUnit,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        let unit = Self::length_unit(document)?;
        let Some(unit) = unit.get() else {
            return Ok(false);
        };
        *result =
            unit.get_unit_value() * UnitsMethods::get_length_unit_scale(LengthUnit::Meter, base);
        Ok(true)
    }
    /// `GetLengthUnit(theDoc, theResult)`: the scale to the meter.
    pub fn get_length_unit(
        document: &Handle<TDocStdDocument>,
        result: &mut f64,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        let unit = Self::length_unit(document)?;
        let Some(unit) = unit.get() else {
            return Ok(false);
        };
        *result = unit.get_unit_value();
        Ok(true)
    }
    /// `SetLengthUnit(theDoc, theUnitValue, theBaseUnit)`: the unit of a value in the base
    /// unit, stored in meters.
    pub fn set_length_unit_in(
        document: &Handle<TDocStdDocument>,
        value: f64,
        base: LengthUnit,
    ) -> Result<(), Failure> {
        let name = UnitsMethods::dump_length_unit_factor(value, base);
        let scale = value * UnitsMethods::get_length_unit_scale(base, LengthUnit::Meter);
        let main =
            rustycad_tklcaf::live(document, "XCAFDoc_DocumentTool::SetLengthUnit")?.main()?;
        XCAFDocLengthUnit::set(&main.root()?, &AsciiString::from(name), scale)?;
        Ok(())
    }
    /// `SetLengthUnit(theDoc, theUnitValue)`: the unit of a scale to the meter.
    pub fn set_length_unit(document: &Handle<TDocStdDocument>, value: f64) -> Result<(), Failure> {
        let name = UnitsMethods::dump_length_unit_factor(value, LengthUnit::Meter);
        let main =
            rustycad_tklcaf::live(document, "XCAFDoc_DocumentTool::SetLengthUnit")?.main()?;
        XCAFDocLengthUnit::set(&main.root()?, &AsciiString::from(name), value)?;
        Ok(())
    }
}
