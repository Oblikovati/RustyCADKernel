// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.cxx (adding and removing shapes,
// references and components) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! How `XCAFDoc_ShapeTool` adds shapes: a new top-level label per shape, an assembly per
//! compound (its components are references to the shapes of the children), and references
//! with locations.
use super::shape_tool::{XCAFDocShapeTool, shape_ref_node};
use super::support::live;
use super::{XCAFDoc, XCAFDocLocation, XCAFDocShapeMapTool};
use rustycad_tkbrep::topods::{Builder, Shape, ShapeIterator};
use rustycad_tkcaf::tnaming::TNamingBuilder;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tkg3d::top_abs::{ShapeEnum, shape_type_to_string};
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdTreeNode};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource, TdfTool};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;

/// `SetLabelNameByLink(L)`: the name `=>[entry]` of the referred label; natively an error
/// line on the standard output without a reference.
fn set_label_name_by_link(label: &TdfLabel) -> Result<(), Failure> {
    let node = shape_ref_node(label)?;
    let Some(node) = node.get().filter(|node| node.has_father()) else {
        println!("Error: XCAFDoc_ShapeTool, SetLabelNameByLink(): NO NODE");
        return Ok(());
    };
    let mut entry = AsciiString::default();
    let father = node.father();
    TdfTool::entry(&live(&father, "SetLabelNameByLink")?.label(), &mut entry);
    entry.insert(1, "=>[")?;
    entry.assign_cat("]");
    TDataStdName::set(label, &ExtendedString::from_ascii(&entry))?;
    Ok(())
}

/// The `TopAbs::Print` name of a shape type, as an extended string.
pub(crate) fn shape_type_name(kind: ShapeEnum) -> ExtendedString {
    ExtendedString::from_utf8(shape_type_to_string(kind))
}

/// `SetLabelNameByShape(L)`: the type of the label's shape when it has no name yet.
fn set_label_name_by_shape(label: &TdfLabel) -> Result<(), Failure> {
    let mut shape = Shape::default();
    if XCAFDocShapeTool::get_shape(label, &mut shape)?
        && !label.is_attribute(&TDataStdName::get_id())?
    {
        TDataStdName::set(label, &shape_type_name(shape.shape_type()?))?;
    }
    Ok(())
}

/// The shape without its location (`S0 = S; S0.Location(TopLoc_Location())`).
pub(crate) fn unlocated(shape: &Shape) -> Result<Shape, Failure> {
    shape.located(&Location::default(), true)
}

impl XCAFDocShapeTool {
    /// `MakeReference(L, refL, loc)`: the location and a reference node of `L` appended to
    /// the node of `refL`, then the link name when naming automatically.
    pub(crate) fn make_reference(
        label: &TdfLabel,
        referred: &TdfLabel,
        location: &Location,
    ) -> Result<(), Failure> {
        XCAFDocLocation::set(label, location)?;
        let main = TDataStdTreeNode::set_with_id(referred, &XCAFDoc::shape_ref_guid())?;
        let reference = TDataStdTreeNode::set_with_id(label, &XCAFDoc::shape_ref_guid())?;
        // abv: fix against bug in TreeNode::Append()
        live(&reference, "XCAFDoc_ShapeTool::MakeReference")?.remove()?;
        live(&main, "XCAFDoc_ShapeTool::MakeReference")?.append(&reference)?;
        if Self::auto_naming() {
            set_label_name_by_link(label)?;
        }
        Ok(())
    }
    /// `NewShape`: a new top-level label with an empty compound.
    pub fn new_shape(&self) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        TNamingBuilder::new(&label)?.generated(&super::shape_tool::empty_compound())?;
        Ok(label)
    }
    /// `SetShape(L, S)`: the shape of the label, its sub-shape map, and the cache entry
    /// when the shape is not known yet.
    pub fn set_shape(&self, label: &TdfLabel, shape: &Shape) -> Result<(), Failure> {
        TNamingBuilder::new(label)?.generated(shape)?;
        let map = XCAFDocShapeMapTool::set(label)?;
        live(&map, "XCAFDoc_ShapeTool::SetShape")?.set_shape(shape);
        self.bind_shape_label(shape, label);
        Ok(())
    }
    /// `if (!myShapeLabels.IsBound(S)) myShapeLabels.Bind(S, L)`.
    pub(crate) fn bind_shape_label(&self, shape: &Shape, label: &TdfLabel) {
        let mut caches = self.state.write();
        if !caches.shape_labels.is_bound(shape) {
            caches.shape_labels.bind(shape.clone(), label.clone());
        }
    }
    /// `SetLocation(theShapeLabel, theLoc, theRefLabel)`: the label itself for the identity,
    /// a composed location for a reference, a new reference for a free shape.
    pub fn set_location(
        &self,
        shape_label: &TdfLabel,
        location: &Location,
        reference: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        if location.is_identity() {
            *reference = shape_label.clone();
            return Ok(true);
        }
        if Self::is_reference(shape_label)? {
            let old = Self::get_location(shape_label)?;
            let composed = location.transformation().multiplied(old.transformation());
            XCAFDocLocation::set(shape_label, &Location::new(composed))?;
            *reference = shape_label.clone();
            return Ok(true);
        }
        if Self::is_shape(shape_label)? && Self::is_free(shape_label)? {
            *reference = TdfTagSource::new_child_of(&self.label())?;
            Self::make_reference(reference, shape_label, location)?;
            return Ok(true);
        }
        Ok(false)
    }
    /// `addShape(S, makeAssembly)`: the label of a shape already there (as an instance), or
    /// a new top-level label: a reference for a located shape, else the named shape, its
    /// map, its name, the assembly of a compound, and the sub-shape cache.
    pub(crate) fn add_shape_internal(
        &self,
        shape: &Shape,
        make_assembly: bool,
    ) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        if shape.is_null() || self.find_shape(shape, &mut label, true)? {
            return Ok(label);
        }
        let label = TdfTagSource::new_child_of(&self.label())?;
        if !shape.location().is_identity() {
            let referred = self.add_shape_internal(&unlocated(shape)?, make_assembly)?;
            Self::make_reference(&label, &referred, shape.location())?;
            return Ok(label);
        }
        TNamingBuilder::new(&label)?.generated(shape)?;
        let map = XCAFDocShapeMapTool::set(&label)?;
        live(&map, "XCAFDoc_ShapeTool::addShape")?.set_shape(shape);
        if Self::auto_naming() {
            set_label_name_by_shape(&label)?;
        }
        if make_assembly && shape.shape_type()? == ShapeEnum::Compound {
            self.add_assembly_components(&label, shape)?;
        }
        if !Self::is_assembly(&label)? {
            self.cache_sub_shapes(
                &label,
                &live(&map, "XCAFDoc_ShapeTool::addShape")?.get_map(),
            )?;
        }
        Ok(label)
    }
    /// The assembly part of `addShape`: the flag, the name and one component per child.
    fn add_assembly_components(&self, label: &TdfLabel, shape: &Shape) -> Result<(), Failure> {
        Self::set_assembly_flag(label)?;
        if Self::auto_naming() {
            TDataStdName::set(label, &ExtendedString::from_utf8("ASSEMBLY"))?;
        }
        let mut iterator = ShapeIterator::new(shape, true, true);
        while let Ok(component) = iterator.value() {
            let component = component.clone();
            let referred = self.add_shape_internal(&unlocated(&component)?, true)?;
            let reference = TdfTagSource::new_child_of(label)?;
            Self::make_reference(&reference, &referred, component.location())?;
            iterator.next();
        }
        Ok(())
    }
    /// The sub-shape cache of a simple shape: each mapped sub-shape (also without its
    /// location) to the label.
    fn cache_sub_shapes(
        &self,
        label: &TdfLabel,
        map: &rustycad_tkbrep::toptools::IndexedMapOfShape,
    ) -> Result<(), Failure> {
        for sub in map.iter() {
            let unlocated_sub = if sub.location().is_identity() {
                None
            } else {
                Some(unlocated(sub)?)
            };
            let mut caches = self.state.write();
            caches.sub_shapes.bind(sub.clone(), label.clone());
            if let Some(unlocated_sub) = unlocated_sub {
                caches.sub_shapes.bind(unlocated_sub, label.clone());
            }
        }
        Ok(())
    }
    /// `AddShape(S, makeAssembly, makePrepare)`: prepares a compound to be an assembly
    /// (OCC1669), adds it and caches its label.
    pub fn add_shape(
        &self,
        shape: &Shape,
        make_assembly: bool,
        make_prepare: bool,
    ) -> Result<TdfLabel, Failure> {
        let mut prepared = shape.clone();
        if make_prepare && make_assembly && shape.shape_type()? == ShapeEnum::Compound {
            prepare_assembly(shape, &mut prepared)?;
        }
        let label = self.add_shape_internal(&prepared, make_assembly)?;
        self.bind_shape_label(&prepared, &label);
        Ok(label)
    }
    /// `RemoveShape(L, removeCompletely)`: forgets a free top-level label (and, completely,
    /// the shape it refers to); false for another label.
    pub fn remove_shape(&self, label: &TdfLabel, completely: bool) -> Result<bool, Failure> {
        if !self.is_top_level(label) || !Self::is_free(label)? {
            return Ok(false);
        }
        let mut referred = TdfLabel::default();
        let node = shape_ref_node(label)?;
        if let Some(node) = node.get().filter(|n| completely && n.has_father())
            && label.is_attribute(&XCAFDocLocation::get_id())?
        {
            referred = live(&node.father(), "XCAFDoc_ShapeTool::RemoveShape")?.label();
        }
        label.forget_all_attributes(true)?;
        if completely && !referred.is_null() {
            return self.remove_shape(&referred, true);
        }
        Ok(true)
    }
    /// `AddComponent(assembly, comp, Loc)`: a new reference child of the assembly (a simple
    /// shape becomes an assembly first); a null label for another label.
    pub fn add_component(
        &self,
        assembly: &TdfLabel,
        component: &TdfLabel,
        location: &Location,
    ) -> Result<TdfLabel, Failure> {
        if !Self::is_assembly(assembly)? {
            if !Self::is_simple_shape(assembly)? {
                return Ok(TdfLabel::default());
            }
            Self::set_assembly_flag(assembly)?;
        }
        let label = TdfTagSource::new_child_of(assembly)?;
        Self::make_reference(&label, component, location)?;
        let mut shape = Shape::default();
        if Self::get_shape(&label, &mut shape)? {
            self.bind_shape_label(&shape, &label);
        }
        Ok(label)
    }
    /// `AddComponent(assembly, comp, expand)`: adds the shape (as an assembly when `expand`)
    /// and a component at its location.
    pub fn add_component_shape(
        &self,
        assembly: &TdfLabel,
        component: &Shape,
        expand: bool,
    ) -> Result<TdfLabel, Failure> {
        let label = self.add_shape(&unlocated(component)?, expand, true)?;
        self.add_component(assembly, &label, component.location())
    }
    /// `RemoveComponent(comp)`: forgets a component.
    pub fn remove_component(&self, component: &TdfLabel) -> Result<(), Failure> {
        if Self::is_component(component)? {
            component.forget_all_attributes(true)?;
        }
        Ok(())
    }
}

/// `prepareAssembly(theShape, theOUTShape)`: a compound becomes free and each child is
/// prepared and given a location (an identity scale when it has none), so that every child
/// becomes a component.
pub(crate) fn prepare_assembly(shape: &Shape, prepared: &mut Shape) -> Result<bool, Failure> {
    *prepared = shape.clone();
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(true);
    }
    if !prepared.free()? {
        prepared.set_free(true)?;
    }
    let mut children = Vec::new();
    let mut iterator = ShapeIterator::new(shape, true, true);
    while let Ok(child) = iterator.value() {
        children.push(child.clone());
        iterator.next();
    }
    for child in &children {
        Builder.remove(prepared, child)?;
        let mut new_child = Shape::default();
        prepare_assembly(child, &mut new_child)?;
        if new_child.location().is_identity() {
            let mut scale = Transform3::default();
            scale.set_scale(Point3::new(0.0, 0.0, 0.0), 1.0)?;
            new_child.set_location(&Location::new(scale), false)?;
        }
        Builder.add(prepared, &new_child)?;
    }
    Ok(true)
}
