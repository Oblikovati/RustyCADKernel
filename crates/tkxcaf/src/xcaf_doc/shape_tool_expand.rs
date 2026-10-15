// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.cxx (the SHUO instances,
// Expand, UpdateAssemblies and the named properties) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members of `XCAFDoc_ShapeTool` that rebuild shapes: the located instances of a SHUO,
//! the expansion of a compound into an assembly and the update of the assembly compounds.
use super::XCAFDocGraphNode;
use super::shape_tool::XCAFDocShapeTool;
use super::shape_tool_add::shape_type_name;
use super::support::find;
use rustycad_tkbrep::topods::{Builder, Shape, ShapeIterator};
use rustycad_tkcaf::tnaming::TNamingBuilder;
use rustycad_tkernel::ncollection::{IndexedMap, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName, TDataStdNamedData};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkmath::toploc::Location;

/// The locations of the upper usages of a SHUO, in order.
type LocationChain = IndexedMap<Location>;

/// `getShapesOfSHUO(theaPrevLocMap, theSTool, theSHUOlab, theShape)`: follows the next
/// usages down to the last one, whose component shape is placed by the composed locations.
fn shapes_of_shuo(
    chain: &LocationChain,
    shuo_label: &TdfLabel,
    shape: &mut Shape,
) -> Result<bool, Failure> {
    let mut next = Vec::new();
    XCAFDocShapeTool::get_shuo_next_usage(shuo_label, &mut next)?;
    if !next.is_empty() {
        for sub_component in next {
            let component_location = XCAFDocShapeTool::get_location(&sub_component.father())?;
            let mut longer = chain.clone();
            longer.add(component_location);
            shapes_of_shuo(&longer, &sub_component, shape)?;
        }
        return Ok(!shape.is_null());
    }
    let mut usage = XCAFDocShapeTool::shape_of(&shuo_label.father())?;
    if usage.is_null() {
        return Ok(false);
    }
    usage.set_location(&Location::default(), true)?;
    let length = chain.extent();
    if length < 1 {
        return Ok(false); // should not be, but to avoid exception...?
    }
    let mut composed = chain.find_key(length)?.clone();
    for index in (1..length).rev() {
        composed = chain.find_key(index)?.multiplied(&composed)?;
    }
    usage.set_location(&composed, false)?;
    *shape = usage;
    Ok(!shape.is_null())
}

/// `getUsersShapesOfSHUO(aPrevLocMap, STool, aSHUOlab, theUserL, theSHUOShapeSeq)`: the
/// instances of the SHUO for each user of the assemblies above.
fn users_shapes_of_shuo(
    chain: &LocationChain,
    shuo_label: &TdfLabel,
    user: &TdfLabel,
    shapes: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let mut longer = LocationChain::default();
    longer.add(XCAFDocShapeTool::get_location(&user.father())?);
    longer.add(XCAFDocShapeTool::get_location(user)?);
    for index in 1..=chain.extent() {
        longer.add(chain.find_key(index)?.clone());
    }
    let mut users = Vec::new();
    XCAFDocShapeTool::get_users(&user.father(), &mut users, false)?;
    if users.is_empty() {
        let mut shape = Shape::default();
        shapes_of_shuo(&longer, shuo_label, &mut shape)?;
        if !shape.is_null() {
            shapes.push(shape);
            return Ok(true);
        }
    }
    for next_user in users {
        users_shapes_of_shuo(&longer, shuo_label, &next_user, shapes)?;
    }
    Ok(shapes.len() > 1)
}

/// The chain a SHUO instance starts from: the location of the upper assembly (when not the
/// identity) and of the component.
fn shuo_start(shuo_label: &TdfLabel) -> Result<LocationChain, Failure> {
    let mut chain = LocationChain::default();
    let location = XCAFDocShapeTool::get_location(&shuo_label.father().father())?;
    if !location.is_identity() {
        chain.add(location);
    }
    chain.add(XCAFDocShapeTool::get_location(&shuo_label.father())?);
    Ok(chain)
}

impl XCAFDocShapeTool {
    /// `GetSHUOInstance(theSHUO)`: the located shape of the last usage of the SHUO.
    pub fn get_shuo_instance(&self, shuo: &Handle<XCAFDocGraphNode>) -> Result<Shape, Failure> {
        let mut shape = Shape::default();
        let Some(shuo) = shuo.get() else {
            return Ok(shape);
        };
        let label = shuo.label();
        shapes_of_shuo(&shuo_start(&label)?, &label, &mut shape)?;
        Ok(shape)
    }
    /// `GetAllSHUOInstances(theSHUO, theSHUOShapeSeq)`: the instances for every user of the
    /// assembly; true for more than one instance, or for the single instance of an unused
    /// assembly.
    pub fn get_all_shuo_instances(
        &self,
        shuo: &Handle<XCAFDocGraphNode>,
        shapes: &mut Vec<Shape>,
    ) -> Result<bool, Failure> {
        let Some(shuo) = shuo.get() else {
            return Ok(false);
        };
        let label = shuo.label();
        let chain = shuo_start(&label)?;
        let mut users = Vec::new();
        Self::get_users(&label.father().father(), &mut users, false)?;
        if users.is_empty() {
            let mut shape = Shape::default();
            shapes_of_shuo(&chain, &label, &mut shape)?;
            if !shape.is_null() {
                shapes.push(shape);
                return Ok(true);
            }
        }
        for user in users {
            users_shapes_of_shuo(&chain, &label, &user, shapes)?;
        }
        Ok(shapes.len() > 1)
    }
}

/// The children of a shape (`TopoDS_Iterator`).
fn children_of(shape: &Shape) -> Vec<Shape> {
    let mut children = Vec::new();
    let mut iterator = ShapeIterator::new(shape, true, true);
    while let Ok(child) = iterator.value() {
        children.push(child.clone());
        iterator.next();
    }
    children
}

/// The name of a new part or sub-shape: the given name attribute, else the shape type.
fn name_part(label: &TdfLabel, name: &Handle<TDataStdName>, shape: &Shape) -> Result<(), Failure> {
    let text = match name.get() {
        Some(name) => name.get(),
        None => shape_type_name(shape.shape_type()?),
    };
    TDataStdName::set(label, &text)?;
    Ok(())
}

impl XCAFDocShapeTool {
    /// `Expand(Shape)`: a compound, compsolid, shell or wire becomes an assembly: each child
    /// becomes a sub-shape label referring to a part (a top-level shape), and the sub-shapes
    /// below move to the parts.
    pub fn expand(&self, shape_label: &TdfLabel) -> Result<bool, Failure> {
        if shape_label.is_null() || Self::is_assembly(shape_label)? {
            return Ok(false);
        }
        let shape = Self::shape_of(shape_label)?;
        if shape.is_null() {
            return Ok(false);
        }
        let expanded = [
            ShapeEnum::Compound,
            ShapeEnum::CompSolid,
            ShapeEnum::Shell,
            ShapeEnum::Wire,
        ];
        if !expanded.contains(&shape.shape_type()?) {
            return Ok(false);
        }
        for child in children_of(&shape) {
            self.expand_child(shape_label, &child)?;
        }
        Self::set_assembly_flag(shape_label)?;
        Ok(true)
    }
    /// One child of `Expand`: its sub-shape label, its part, the reference and the moved
    /// sub-shapes.
    fn expand_child(&self, shape_label: &TdfLabel, child: &Shape) -> Result<(), Failure> {
        let mut child_label = TdfLabel::default();
        self.find_sub_shape(shape_label, child, &mut child_label)?;
        let mut name: Handle<TDataStdName> = Handle::default();
        if child_label.is_null() {
            child_label = self.add_sub_shape(shape_label, child)?;
        } else {
            child_label.find_attribute_as(&TDataStdName::get_id(), &mut name)?;
        }
        let part = self.expand_part(child)?;
        name_part(&part, &name, child)?;
        Self::make_reference(&child_label, &part, child.location())?;
        self.make_sub_shape(shape_label, &part, child, child.location())
    }
    /// The part of an expanded child: the top-level shape of the unlocated child (added when
    /// absent; a non-top-level label becomes its referred shape, or a new top-level label).
    fn expand_part(&self, child: &Shape) -> Result<TdfLabel, Failure> {
        let unlocated_child = child.located(&Location::default(), true)?;
        let mut part = self.find_shape_label(&unlocated_child, false)?;
        if part.is_null() {
            part = self.add_shape(&unlocated_child, false, false)?;
        }
        if !part.is_null() && !self.is_top_level(&part) {
            let mut referred = TdfLabel::default();
            if Self::get_referred_shape(&part, &mut referred)? {
                part = referred;
            } else {
                part = TdfTagSource::new_child_of(&self.label())?;
                self.set_shape(&part, &unlocated_child)?;
            }
        }
        Ok(part)
    }
    /// `makeSubShape(theMainShapeL, thePart, theShape, theLoc)`: moves the sub-shape labels
    /// of the main shape below the shape to the part (references to the old labels), at any
    /// depth.
    fn make_sub_shape(
        &self,
        main: &TdfLabel,
        part: &TdfLabel,
        shape: &Shape,
        location: &Location,
    ) -> Result<(), Failure> {
        let is_compound_part = Self::shape_of(part)?.shape_type()? == ShapeEnum::Compound;
        let is_assembly = Self::is_assembly(part)?;
        for child in children_of(shape) {
            let mut child_label = TdfLabel::default();
            self.find_sub_shape(main, &child, &mut child_label)?;
            if !child_label.is_null() {
                if is_assembly {
                    child_label.forget_all_attributes(true)?;
                    self.make_sub_shape(main, part, &child, location)?;
                    continue;
                }
                self.move_sub_shape(part, &child_label, &child, location, is_compound_part)?;
            }
            self.make_sub_shape(main, part, &child, location)?;
        }
        Ok(())
    }
    /// One found sub-shape of `makeSubShape`: a new sub-shape of the part (relocated for a
    /// compound part) named as the old one and referring to it, else the old label forgets
    /// its attributes.
    fn move_sub_shape(
        &self,
        part: &TdfLabel,
        child_label: &TdfLabel,
        child: &Shape,
        location: &Location,
        is_compound_part: bool,
    ) -> Result<(), Failure> {
        let name = find::<TDataStdName>(child_label, &TDataStdName::get_id())?;
        let mut sub_location = child.location().clone();
        if is_compound_part {
            sub_location = location.inverted()?.multiplied(&sub_location)?;
        }
        let mut sub_label = TdfLabel::default();
        let mut is_new =
            self.add_sub_shape_to(part, &child.located(&sub_location, false)?, &mut sub_label)?;
        if sub_label.is_null() {
            let unlocated_child = child.located(&Location::default(), true)?;
            is_new = self.add_sub_shape_to(part, &unlocated_child, &mut sub_label)?;
        }
        if !is_new {
            return child_label.forget_all_attributes(true);
        }
        name_part(&sub_label, &name, child)?;
        Self::make_reference(&sub_label, child_label, child.location())
    }
}

impl XCAFDocShapeTool {
    /// `UpdateAssemblies`: rebuilds the compounds of the assemblies whose components changed,
    /// from each free shape (or the shape it refers to).
    pub fn update_assemblies(&self) -> Result<(), Failure> {
        let mut updated = Map::<TdfLabel>::default();
        for root in self.get_free_shapes()? {
            let mut root = root;
            if Self::is_reference(&root)? {
                let mut referred = TdfLabel::default();
                Self::get_referred_shape(&root, &mut referred)?;
                root = referred;
            }
            let mut shape = Shape::default();
            self.update_component(&root, &mut shape, &mut updated)?;
        }
        Ok(())
    }
    /// `updateComponent(theAssmLabel, theUpdatedShape, theUpdated)`: the compound of the
    /// component shapes (the sub-assemblies updated first, placed again) when a component is
    /// new, missing or changed; true when the assembly was updated.
    fn update_component(
        &self,
        item: &TdfLabel,
        updated_shape: &mut Shape,
        updated: &mut Map<TdfLabel>,
    ) -> Result<bool, Failure> {
        if !Self::is_assembly(item)? {
            return Ok(false); // Do nothing for non-assemblies
        }
        let current = Self::shape_of(item)?;
        if updated.contains(item) {
            *updated_shape = current;
            return Ok(true);
        }
        let mut components = Vec::new();
        Self::get_components(item, &mut components, false)?;
        let mut modified = current.nb_children() != components.len() as i32;
        let mut current_children = Map::<Shape>::default();
        let mut shapes = Vec::new();
        for component in &components {
            let shape = self.updated_component_shape(
                component,
                &current,
                &mut current_children,
                &mut modified,
                updated,
            )?;
            shapes.push(shape);
        }
        if modified {
            let mut compound = super::shape_tool::empty_compound();
            for shape in &shapes {
                Builder.add(&mut compound, shape)?;
            }
            *updated_shape = compound;
            TNamingBuilder::new(item)?.generated(updated_shape)?;
            updated.add(item.clone());
        }
        Ok(modified)
    }
    /// The shape of one component in `updateComponent`, updating `modified`.
    fn updated_component_shape(
        &self,
        component: &TdfLabel,
        current: &Shape,
        current_children: &mut Map<Shape>,
        modified: &mut bool,
        updated: &mut Map<TdfLabel>,
    ) -> Result<Shape, Failure> {
        let mut referred = TdfLabel::default();
        Self::get_referred_shape(component, &mut referred)?;
        let mut shape = Self::shape_of(component)?;
        let location = shape.location().clone();
        if Self::is_assembly(&referred)? {
            if self.update_component(&referred, &mut shape, updated)? {
                *modified = true;
                shape.set_location(&location, false)?; // Apply placement
            }
            return Ok(shape);
        }
        if !*modified {
            if current_children.is_empty() {
                for child in children_of(current) {
                    current_children.add(child);
                }
            }
            if !current_children.contains(&shape) {
                *modified = true;
            }
        }
        Ok(shape)
    }
    /// `GetNamedProperties(theLabel, theToCreate)`: the label's `TDataStd_NamedData`, created
    /// on request.
    pub fn get_named_properties(
        &self,
        label: &TdfLabel,
        create: bool,
    ) -> Result<Handle<TDataStdNamedData>, Failure> {
        let found = find::<TDataStdNamedData>(label, &TDataStdNamedData::get_id())?;
        if found.is_null() && create {
            return TDataStdNamedData::set(label);
        }
        Ok(found)
    }
    /// `GetNamedProperties(theShape, theToCreate)`: those of the label the search finds.
    pub fn get_named_properties_of_shape(
        &self,
        shape: &Shape,
        create: bool,
    ) -> Result<Handle<TDataStdNamedData>, Failure> {
        let mut label = TdfLabel::default();
        if !self.search(shape, &mut label, [true, true, true])? {
            return Ok(Handle::default());
        }
        self.get_named_properties(&label, create)
    }
}
