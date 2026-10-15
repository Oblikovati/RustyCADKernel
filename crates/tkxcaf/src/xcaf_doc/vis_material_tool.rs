// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_VisMaterialTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_VisMaterialTool`: the visualization materials of an XDE document (one
//! `XCAFDoc_VisMaterial` per child of the tool's label) and the links of shapes to them
//! (tree nodes of `VisMaterialRefGUID`).
use super::support::{child_labels_with, find, find_or_add, live};
use super::tool_support::{cached_shape_tool, search_shape};
use super::{XCAFDoc, XCAFDocDocumentTool, XCAFDocShapeTool, XCAFDocVisMaterial};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, TdfTagSource,
};

/// `XCAFDoc_VisMaterialTool::GetID()`.
const VIS_MATERIAL_TOOL_ID: Guid = guid_from_text("87B511CE-DA15-4A5E-98AF-E3F46AB5B6E8");

/// `XCAFDoc_VisMaterialTool`: a plain attribute (not a `TDataStd_GenericEmpty`).
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocVisMaterialTool;
/// let data = TdfData::new();
/// let tool = XCAFDocVisMaterialTool::set(&data.root().find_child(1, true)?)?;
/// let material = tool.add_material(&"steel".into())?;
/// assert_eq!(tool.get_materials()?, vec![material]);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocVisMaterialTool {
    transient: StandardTransient,
    base: TdfAttributeBase,
    shape_tool: SharedState<Handle<XCAFDocShapeTool>>,
}
rustycad_tkernel::impl_transient!(XCAFDocVisMaterialTool, "XCAFDoc_VisMaterialTool", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl XCAFDocVisMaterialTool {
    /// `XCAFDoc_VisMaterialTool()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        VIS_MATERIAL_TOOL_ID
    }
    /// `Set(theLabel)`: the tool of the label, created (with the document's shape tool) when
    /// absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocVisMaterialTool>, Failure> {
        let existing = find::<XCAFDocVisMaterialTool>(label, &VIS_MATERIAL_TOOL_ID)?;
        if !existing.is_null() {
            return Ok(existing);
        }
        let tool = find_or_add(label, &VIS_MATERIAL_TOOL_ID, Self::new)?;
        *live(&tool, "XCAFDoc_VisMaterialTool::Set")?
            .shape_tool
            .write() = XCAFDocDocumentTool::shape_tool(label)?;
        Ok(tool)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `ShapeTool()`: the document's shape tool (looked up once).
    pub fn shape_tool(&self) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        cached_shape_tool(&self.shape_tool, &self.label())
    }
    /// `IsMaterial(theLabel)`: the label holds a non-null material.
    pub fn is_material(&self, label: &TdfLabel) -> Result<bool, Failure> {
        Ok(!Self::get_material(label)?.is_null())
    }
    /// `GetMaterial(theMatLabel)`: the material of the label, or null.
    pub fn get_material(label: &TdfLabel) -> Result<Handle<XCAFDocVisMaterial>, Failure> {
        find::<XCAFDocVisMaterial>(label, &XCAFDocVisMaterial::get_id())
    }
    /// `AddMaterial(theMat, theName)`: a new child with the material, named when the name is
    /// not empty.
    pub fn add_material_attribute(
        &self,
        material: &Handle<XCAFDocVisMaterial>,
        name: &AsciiString,
    ) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        label.add_attribute(&material.clone().cast(), false)?;
        if !name.is_empty() {
            TDataStdName::set(&label, &ExtendedString::from_ascii(name))?;
        }
        Ok(label)
    }
    /// `AddMaterial(theName)`: a new child with a new empty material.
    pub fn add_material(&self, name: &AsciiString) -> Result<TdfLabel, Failure> {
        self.add_material_attribute(&Handle::new(XCAFDocVisMaterial::new()), name)
    }
    /// `RemoveMaterial(theLabel)`: forgets every attribute of the material label.
    pub fn remove_material(&self, label: &TdfLabel) -> Result<(), Failure> {
        label.forget_all_attributes(true)
    }
    /// `GetMaterials(theLabels)`: the material labels.
    pub fn get_materials(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels_with(&self.label(), &XCAFDocVisMaterial::get_id())? {
            if self.is_material(&label)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
}

/// The material links of labels.
impl XCAFDocVisMaterialTool {
    /// `SetShapeMaterial(theShapeLabel, theMaterialLabel)`: the shape's node first among the
    /// users of the material's node; a null material unsets it.
    pub fn set_shape_material(&self, shape: &TdfLabel, material: &TdfLabel) -> Result<(), Failure> {
        let guid = XCAFDoc::vis_material_ref_guid();
        if material.is_null() {
            shape.forget_attribute_id(&guid)?;
            return Ok(());
        }
        let main = TDataStdTreeNode::set_with_id(material, &guid)?;
        let reference = TDataStdTreeNode::set_with_id(shape, &guid)?;
        // abv: fix against bug in TreeNode::Append()
        live(&reference, "XCAFDoc_VisMaterialTool::SetShapeMaterial")?.remove()?;
        live(&main, "XCAFDoc_VisMaterialTool::SetShapeMaterial")?.prepend(&reference)?;
        Ok(())
    }
    /// `UnSetShapeMaterial(theShapeLabel)`.
    pub fn unset_shape_material(&self, shape: &TdfLabel) -> Result<(), Failure> {
        shape.forget_attribute_id(&XCAFDoc::vis_material_ref_guid())?;
        Ok(())
    }
    /// `IsSetShapeMaterial(theLabel)`: a link with a material.
    pub fn is_set_shape_material(&self, shape: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::get_shape_material_label(shape)?.is_some())
    }
    /// `GetShapeMaterial(theShapeLabel, theMaterialLabel)`: the linked material label.
    pub fn get_shape_material_label(shape: &TdfLabel) -> Result<Option<TdfLabel>, Failure> {
        let node = find::<TDataStdTreeNode>(shape, &XCAFDoc::vis_material_ref_guid())?;
        let Some(node) = node.get().filter(|node| node.has_father()) else {
            return Ok(None);
        };
        Ok(Some(
            live(&node.father(), "XCAFDoc_VisMaterialTool::GetShapeMaterial")?.label(),
        ))
    }
    /// `GetShapeMaterial(theShapeLabel)`: the linked material, or null.
    pub fn get_shape_material(shape: &TdfLabel) -> Result<Handle<XCAFDocVisMaterial>, Failure> {
        match Self::get_shape_material_label(shape)? {
            Some(material) => Self::get_material(&material),
            None => Ok(Handle::default()),
        }
    }
}

/// The material links of shapes (through `ShapeTool()->Search`).
impl XCAFDocVisMaterialTool {
    /// The label of a shape, `None` when not found.
    fn search(&self, shape: &Shape) -> Result<Option<TdfLabel>, Failure> {
        search_shape(&self.shape_tool()?, shape, "XCAFDoc_VisMaterialTool")
    }
    /// `SetShapeMaterial(theShape, theMaterialLabel)`.
    pub fn set_material_of_shape(
        &self,
        shape: &Shape,
        material: &TdfLabel,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.set_shape_material(&label, material)?;
        Ok(true)
    }
    /// `UnSetShapeMaterial(theShape)`.
    pub fn unset_material_of_shape(&self, shape: &Shape) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.unset_shape_material(&label)?;
        Ok(true)
    }
    /// `IsSetShapeMaterial(theShape)`.
    pub fn is_set_material_of_shape(&self, shape: &Shape) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.is_set_shape_material(&label)
    }
    /// `GetShapeMaterial(theShape, theMaterialLabel)`.
    pub fn get_material_label_of_shape(&self, shape: &Shape) -> Result<Option<TdfLabel>, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(None);
        };
        Self::get_shape_material_label(&label)
    }
    /// `GetShapeMaterial(theShape)`: the material, or null.
    pub fn get_material_of_shape(
        &self,
        shape: &Shape,
    ) -> Result<Handle<XCAFDocVisMaterial>, Failure> {
        match self.get_material_label_of_shape(shape)? {
            Some(material) => Self::get_material(&material),
            None => Ok(Handle::default()),
        }
    }
}

impl TdfAttribute for XCAFDocVisMaterialTool {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        VIS_MATERIAL_TOOL_ID
    }
    /// `Restore`: nothing to restore.
    fn restore(&self, _with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: nothing to paste.
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
