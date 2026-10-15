// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_Editor.hxx and .cxx (Expand, Extract and
// CloneShapeLabel) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_Editor`: the editing of the shapes of an XDE document: the expansion of
//! compounds into assemblies and the extraction of shapes with their metadata into another
//! assembly.
use super::editor_metadata::{XCAFDocCloneFlags, XCAFDocVisMaterialMap};
use super::support::find;
use super::{XCAFDoc, XCAFDocDocumentTool, XCAFDocLocation, XCAFDocShapeMapTool, XCAFDocShapeTool};
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel};
use rustycad_tkmath::toploc::Location;

/// The labels cloned from a source document (`NCollection_DataMap<TDF_Label, TDF_Label>`).
pub type XCAFDocLabelMap = DataMap<TdfLabel, TdfLabel>;

/// `XCAFDoc_Editor`: a class of static members only.
pub struct XCAFDocEditor;

/// The shape tool of the document of a label (`Standard_NullObject` without one).
fn shape_tool_of(label: &TdfLabel) -> Result<Handle<XCAFDocShapeTool>, Failure> {
    XCAFDocDocumentTool::shape_tool(label)
}

/// The expanded child of `Expand`: a component moves its metadata to its part (and
/// replaces it when the part's parent has a shape), a part moves its metadata to its users.
fn expand_child(child: &TdfLabel, tool: &XCAFDocShapeTool) -> Result<(), Failure> {
    let mut part = TdfLabel::default();
    if XCAFDocShapeTool::get_referred_shape(child, &mut part)? {
        XCAFDocEditor::clone_meta_data(child, &part, None, XCAFDocCloneFlags::default())?;
        let shape = XCAFDocShapeTool::shape_of(child)?;
        let parent_shape = XCAFDocShapeTool::shape_of(&part.father())?;
        if !parent_shape.is_null() {
            part.forget_attribute_id(&XCAFDoc::shape_ref_guid())?;
            if parent_shape.shape_type()? == ShapeEnum::Compound {
                tool.set_shape(&part, &shape)?;
            }
            part.forget_attribute_id(&XCAFDocShapeMapTool::get_id())?;
            child.forget_all_attributes(false)?;
        }
        child.forget_attribute_id(&TNamingNamedShape::get_id())?;
        child.forget_attribute_id(&XCAFDocShapeMapTool::get_id())?;
        return Ok(());
    }
    let mut users = Vec::new();
    if XCAFDocShapeTool::get_users(child, &mut users, false)? > 0 {
        for user in &users {
            user.forget_attribute_id(&XCAFDoc::shape_ref_guid())?;
            user.forget_attribute_id(&XCAFDocShapeMapTool::get_id())?;
            XCAFDocEditor::clone_meta_data(child, user, None, XCAFDocCloneFlags::default())?;
        }
        child.forget_all_attributes(false)?;
    }
    Ok(())
}

impl XCAFDocEditor {
    /// `Expand(theDoc, theShape, theRecursively)`: expands a compound (or the compound a
    /// reference refers to) into an assembly, moving the metadata of its sub-shapes onto the
    /// new parts, without automatic naming (the native default is recursive).
    pub fn expand_shape(
        document: &TdfLabel,
        shape: &TdfLabel,
        recursively: bool,
    ) -> Result<bool, Failure> {
        if document.is_null() || shape.is_null() {
            return Ok(false);
        }
        let tool = shape_tool_of(document)?;
        let tool = rustycad_tklcaf::live(&tool, "XCAFDoc_Editor::Expand")?;
        let naming = XCAFDocShapeTool::auto_naming();
        XCAFDocShapeTool::set_auto_naming(false);
        let result = Self::expand_compound(document, shape, recursively, tool);
        XCAFDocShapeTool::set_auto_naming(naming);
        result
    }
    /// The body of `Expand` once the auto naming is off.
    fn expand_compound(
        document: &TdfLabel,
        shape: &TdfLabel,
        recursively: bool,
        tool: &XCAFDocShapeTool,
    ) -> Result<bool, Failure> {
        let mut compound = shape.clone();
        if XCAFDocShapeTool::is_reference(shape)? {
            XCAFDocShapeTool::get_referred_shape(&shape.clone(), &mut compound)?;
        }
        if !tool.expand(&compound)? {
            return Ok(false);
        }
        let mut iterator = TdfChildIterator::new(&compound, true)?;
        while iterator.more() {
            expand_child(&iterator.value(), tool)?;
            iterator.next();
        }
        if recursively {
            for mut part in super::support::child_labels(&compound)? {
                if XCAFDocShapeTool::get_referred_shape(&part.clone(), &mut part)? {
                    let part_shape = XCAFDocShapeTool::shape_of(&part)?;
                    if !part_shape.is_null() && part_shape.shape_type()? == ShapeEnum::Compound {
                        Self::expand_shape(document, &part, recursively)?;
                    }
                }
            }
        }
        Ok(true)
    }
    /// `Expand(theDoc, theRecursively)`: expands every free compound that is not an assembly;
    /// true when one was expanded.
    pub fn expand(document: &TdfLabel, recursively: bool) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        let tool = shape_tool_of(document)?;
        let mut result = false;
        for label in rustycad_tklcaf::live(&tool, "XCAFDoc_Editor::Expand")?.get_free_shapes()? {
            let shape = XCAFDocShapeTool::shape_of(&label)?;
            if !shape.is_null()
                && shape.shape_type()? == ShapeEnum::Compound
                && !XCAFDocShapeTool::is_assembly(&label)?
                && Self::expand_shape(document, &label, recursively)?
            {
                result = true;
            }
        }
        Ok(result)
    }
}

/// Extraction.
impl XCAFDocEditor {
    /// `Extract(theSrcLabels, theDstLabel, theIsNoVisMat)`: each source shape cloned under the
    /// destination assembly (or as a new free shape), with the metadata of the cloned labels
    /// (visualization materials shared through one map unless excluded).
    pub fn extract(
        sources: &[TdfLabel],
        destination: &TdfLabel,
        no_vis_material: bool,
    ) -> Result<bool, Failure> {
        if destination.is_null() {
            return Ok(false);
        }
        let target = shape_tool_of(destination)?;
        let mut materials = XCAFDocVisMaterialMap::new();
        for source in sources {
            let map = Self::extract_one(source, destination, &target)?;
            let target_tool = rustycad_tklcaf::live(&target, "XCAFDoc_Editor::Extract")?;
            target_tool.update_assemblies()?;
            let flags = XCAFDocCloneFlags {
                attributes: true,
                vis_material: !no_vis_material,
                ..Default::default()
            };
            for (from, to) in map.items() {
                Self::clone_meta_data(from, to, Some(&mut materials), flags)?;
            }
        }
        Ok(true)
    }
    /// The clone of one source under the destination, with the map of the cloned labels.
    fn extract_one(
        source: &TdfLabel,
        destination: &TdfLabel,
        target: &Handle<XCAFDocShapeTool>,
    ) -> Result<XCAFDocLabelMap, Failure> {
        let location = find::<XCAFDocLocation>(source, &XCAFDocLocation::get_id())?
            .get()
            .map(XCAFDocLocation::get)
            .unwrap_or_default();
        let source_tool = shape_tool_of(source)?;
        let mut component = source.clone();
        XCAFDocShapeTool::get_referred_shape(source, &mut component)?;
        let mut map = XCAFDocLabelMap::default();
        let cloned = Self::clone_shape_label(&component, &source_tool, target, &mut map)?;
        let target_tool = rustycad_tklcaf::live(target, "XCAFDoc_Editor::Extract")?;
        let mut new_component = target_tool.add_component(destination, &cloned, &location)?;
        if new_component.is_null() {
            let mut shape = XCAFDocShapeTool::shape_of(&cloned)?;
            shape.move_by(&location, true)?;
            new_component = target_tool.add_shape(&shape, false, true)?;
        }
        map.bind(source.clone(), new_component);
        map.bind(component, cloned);
        Ok(map)
    }
    /// `Extract(theSrcLabel, theDstLabel, theIsNoVisMat)`.
    pub fn extract_label(
        source: &TdfLabel,
        destination: &TdfLabel,
        no_vis_material: bool,
    ) -> Result<bool, Failure> {
        Self::extract(std::slice::from_ref(source), destination, no_vis_material)
    }
    /// `CloneShapeLabel(theSrcLabel, theSrcShapeTool, theDstShapeTool, theMap)`: the clone of
    /// a shape label (an assembly with clones of its components, else the shape with its
    /// sub-shapes), recorded in the map and reused from it.
    pub fn clone_shape_label(
        source: &TdfLabel,
        source_tool: &Handle<XCAFDocShapeTool>,
        target: &Handle<XCAFDocShapeTool>,
        map: &mut XCAFDocLabelMap,
    ) -> Result<TdfLabel, Failure> {
        if let Some(known) = map.seek(source) {
            return Ok(known.clone());
        }
        rustycad_tklcaf::live(source_tool, "XCAFDoc_Editor::CloneShapeLabel")?;
        if XCAFDocShapeTool::is_assembly(source)? {
            return Self::clone_assembly(source, source_tool, target, map);
        }
        let target_tool = rustycad_tklcaf::live(target, "XCAFDoc_Editor::CloneShapeLabel")?;
        let mut cloned =
            target_tool.add_shape(&XCAFDocShapeTool::shape_of(source)?, false, true)?;
        map.bind(source.clone(), cloned.clone());
        let mut original = source.clone();
        if XCAFDocShapeTool::is_reference(source)? {
            XCAFDocShapeTool::get_referred_shape(source, &mut original)?;
            XCAFDocShapeTool::get_referred_shape(&cloned.clone(), &mut cloned)?;
            map.bind(original.clone(), cloned.clone());
        }
        let mut sub_shapes = Vec::new();
        XCAFDocShapeTool::get_sub_shapes(&original, &mut sub_shapes)?;
        for sub in sub_shapes {
            let added = target_tool.add_sub_shape(&cloned, &XCAFDocShapeTool::shape_of(&sub)?)?;
            map.bind(sub, added);
        }
        Ok(cloned)
    }
    /// The clone of an assembly: an empty compound with clones of the components at their
    /// locations (`Standard_NullObject` for a component without location).
    fn clone_assembly(
        source: &TdfLabel,
        source_tool: &Handle<XCAFDocShapeTool>,
        target: &Handle<XCAFDocShapeTool>,
        map: &mut XCAFDocLabelMap,
    ) -> Result<TdfLabel, Failure> {
        let target_tool = rustycad_tklcaf::live(target, "XCAFDoc_Editor::CloneShapeLabel")?;
        let mut compound = Compound::default();
        Builder.make_compound(&mut compound);
        let cloned = target_tool.add_shape(&compound.into_shape(), true, true)?;
        map.bind(source.clone(), cloned.clone());
        let mut components = Vec::new();
        XCAFDocShapeTool::get_components(source, &mut components, false)?;
        for component in components {
            let mut referred = TdfLabel::default();
            XCAFDocShapeTool::get_referred_shape(&component, &mut referred)?;
            let original = Self::clone_shape_label(&referred, source_tool, target, map)?;
            let location = find::<XCAFDocLocation>(&component, &XCAFDocLocation::get_id())?;
            let location: Location =
                rustycad_tklcaf::live(&location, "XCAFDoc_Editor::CloneShapeLabel")?.get();
            let added = target_tool.add_component(&cloned, &original, &location)?;
            map.bind(component, added);
        }
        Ok(cloned)
    }
}
