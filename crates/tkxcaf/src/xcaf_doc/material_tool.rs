// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_MaterialTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_MaterialTool`: the materials of an XDE document (one `XCAFDoc_Material` per
//! child of the tool's label) and the links of shapes to them (tree nodes of
//! `MaterialRefGUID`).
use super::color_tool::entry_text;
use super::generic_empty_tool::generic_empty_attribute;
use super::support::{child_labels, find, find_or_add, live};
use super::tool_support::cached_shape_tool;
use super::{
    XCAFDoc, XCAFDocDocumentTool, XCAFDocMaterial, XCAFDocMaterialFields, XCAFDocShapeTool,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkservice::json_dump::field_string;
use std::io;

generic_empty_attribute!(
    /// `XCAFDoc_MaterialTool`.
    XCAFDocMaterialTool, "XCAFDoc_MaterialTool", "efd212f9-6dfd-11d4-b9c8-0060b0ee281b",
    state: Handle<XCAFDocShapeTool>,
    dump: |tool: &XCAFDocMaterialTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_MaterialTool")?;
        rustycad_tklcaf::json::dump_attribute_json_base(tool, stream, depth)?;
        for label in tool.get_material_labels().map_err(io::Error::other)? {
            field_string(stream, "aMaterialLabel", entry_text(&label).as_bytes())?;
        }
        Ok(())
    }
);

impl XCAFDocMaterialTool {
    /// `Set(L)`: the tool of the label, created (with the document's shape tool) when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocMaterialTool>, Failure> {
        let existing = find::<XCAFDocMaterialTool>(label, &Self::get_id())?;
        if !existing.is_null() {
            return Ok(existing);
        }
        let tool = find_or_add(label, &Self::get_id(), Self::new)?;
        *live(&tool, "XCAFDoc_MaterialTool::Set")?.state.write() =
            XCAFDocDocumentTool::shape_tool(label)?;
        Ok(tool)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `ShapeTool()`: the document's shape tool (looked up once).
    pub fn shape_tool(&self) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        cached_shape_tool(&self.state, &self.label())
    }
    /// `IsMaterial(lab)`: the label holds a material.
    pub fn is_material(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocMaterial::get_id())
    }
    /// `GetMaterialLabels(Labels)`: the material children of the tool's label.
    pub fn get_material_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels(&self.label())? {
            if self.is_material(&label)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
    /// `AddMaterial(aName, aDescription, aDensity, aDensName, aDensValType)`: a new child with
    /// the material, named after it (`Standard_NullObject` for a null name).
    pub fn add_material(&self, fields: &XCAFDocMaterialFields) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        XCAFDocMaterial::set(&label, fields)?;
        let name = live(&fields.name, "XCAFDoc_MaterialTool::AddMaterial")?
            .string()
            .clone();
        TDataStdName::set(&label, &ExtendedString::from_ascii(&name))?;
        Ok(label)
    }
    /// `SetMaterial(L, MatL)`: the label's node, last among the users of the material's node.
    pub fn set_material_label(&self, label: &TdfLabel, material: &TdfLabel) -> Result<(), Failure> {
        let guid = XCAFDoc::material_ref_guid();
        let main = TDataStdTreeNode::set_with_id(material, &guid)?;
        let reference = TDataStdTreeNode::set_with_id(label, &guid)?;
        // abv: fix against bug in TreeNode::Append()
        live(&reference, "XCAFDoc_MaterialTool::SetMaterial")?.remove()?;
        live(&main, "XCAFDoc_MaterialTool::SetMaterial")?.append(&reference)?;
        Ok(())
    }
    /// `SetMaterial(L, aName, aDescription, aDensity, aDensName, aDensValType)`: links the
    /// label to a new material.
    pub fn set_material(
        &self,
        label: &TdfLabel,
        fields: &XCAFDocMaterialFields,
    ) -> Result<(), Failure> {
        let material = self.add_material(fields)?;
        self.set_material_label(label, &material)
    }
    /// `GetMaterial(MatL, aName, aDescription, aDensity, aDensName, aDensValType)`: the values
    /// of a material label.
    pub fn get_material(material: &TdfLabel) -> Result<Option<XCAFDocMaterialFields>, Failure> {
        let attribute = find::<XCAFDocMaterial>(material, &XCAFDocMaterial::get_id())?;
        Ok(attribute.get().map(XCAFDocMaterial::fields))
    }
    /// `GetDensityForShape(ShapeL)`: the density of the shape's material divided by 1000, 0
    /// without one.
    pub fn get_density_for_shape(shape: &TdfLabel) -> Result<f64, Failure> {
        let node = find::<TDataStdTreeNode>(shape, &XCAFDoc::material_ref_guid())?;
        let Some(node) = node.get().filter(|node| node.has_father()) else {
            return Ok(0.0);
        };
        let material = live(&node.father(), "XCAFDoc_MaterialTool::GetDensityForShape")?.label();
        let attribute = find::<XCAFDocMaterial>(&material, &XCAFDocMaterial::get_id())?;
        Ok(attribute.get().map_or(0.0, |m| m.get_density() * 0.001))
    }
}
