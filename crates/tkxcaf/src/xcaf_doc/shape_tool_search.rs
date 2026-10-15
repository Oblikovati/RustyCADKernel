// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.cxx (searching shapes, sub-shapes
// and main shapes, and the simple-shape cache) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! How `XCAFDoc_ShapeTool` finds the label of a shape: through `TNaming_Tool` (the used
//! shapes of the document), through its caches, or as a sub-shape of a top-level shape.
use super::XCAFDocShapeMapTool;
use super::shape_tool::XCAFDocShapeTool;
use super::shape_tool_add::unlocated;
use super::support::{find, live};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingNamedShape, TNamingTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tklcaf::tdf::{
    TdfAttributeState, TdfChildIdIterator, TdfChildIterator, TdfLabel, TdfTagSource,
};
use rustycad_tkmath::toploc::Location;

impl XCAFDocShapeTool {
    /// `FindShape(S, L, findInstance)`: the label of the shape (without its location unless
    /// `findInstance`): the label `TNaming_Tool` gives when it is top-level, else the first
    /// child (at any depth) whose named shape is the same shape.
    pub fn find_shape(
        &self,
        shape: &Shape,
        label: &mut TdfLabel,
        find_instance: bool,
    ) -> Result<bool, Failure> {
        let shape = if find_instance {
            shape.clone()
        } else {
            unlocated(shape)?
        };
        let root = self.label();
        if !TNamingTool::has_label(&root, &shape)? {
            return Ok(false);
        }
        let mut transaction = 0;
        *label = TNamingTool::label(&root, &shape, &mut transaction)?;
        if self.is_top_level(label) {
            return Ok(true);
        }
        let mut iterator = TdfChildIdIterator::new(&root, &TNamingNamedShape::get_id(), false)?;
        while iterator.more() {
            let found = live(&iterator.value(), "XCAFDoc_ShapeTool::FindShape")?.label();
            let named = find::<TNamingNamedShape>(&found, &TNamingNamedShape::get_id())?;
            if !named.is_null() && shape.is_same(&TNamingTool::get_shape(&named)?) {
                *label = found;
                return Ok(true);
            }
            iterator.next()?;
        }
        *label = TdfLabel::default();
        Ok(false)
    }
    /// `FindShape(S, findInstance)`: the label, null when not found.
    pub fn find_shape_label(
        &self,
        shape: &Shape,
        find_instance: bool,
    ) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        if self.find_shape(shape, &mut label, find_instance)? {
            return Ok(label);
        }
        Ok(TdfLabel::default())
    }
    /// `Search(S, L, findInstance, findComponent, findSubshape)`: an instance or a component
    /// of a located shape, then the shape, then a sub-shape of a top-level shape (added as a
    /// sub-shape label).
    pub fn search(
        &self,
        shape: &Shape,
        label: &mut TdfLabel,
        [find_instance, find_component, find_sub_shape]: [bool; 3],
    ) -> Result<bool, Failure> {
        if !shape.location().is_identity() {
            if find_instance && self.find_shape(shape, label, true)? {
                return Ok(true);
            }
            if find_component && self.search_components(shape, label)? {
                return Ok(true);
            }
        }
        if self.find_shape(shape, label, false)? {
            return Ok(true);
        }
        if !find_sub_shape {
            return Ok(false);
        }
        let main = self.find_main_shape(shape)?;
        if main.is_null() {
            return Ok(false);
        }
        *label = self.add_sub_shape(&main, shape)?;
        Ok(!label.is_null())
    }
    /// The component part of `Search`: a component of an assembly whose shape is the same.
    fn search_components(&self, shape: &Shape, label: &mut TdfLabel) -> Result<bool, Failure> {
        for assembly in self.get_shapes()? {
            if !Self::is_assembly(&assembly)? {
                continue;
            }
            let mut components = Vec::new();
            Self::get_components(&assembly, &mut components, false)?;
            for component in components {
                if Self::shape_of(&component)?.is_same(shape) {
                    *label = component;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// `SearchUsingMap(S, L, findWithoutLoc, findSubshape)`: the same search through the
    /// caches (the simple shapes once computed).
    pub fn search_using_map(
        &self,
        shape: &Shape,
        label: &mut TdfLabel,
        find_without_location: bool,
        find_sub_shape: bool,
    ) -> Result<bool, Failure> {
        if let Some(found) = self.cached_label(|c| &c.shape_labels, shape) {
            *label = found;
            return Ok(true);
        }
        let unlocated_shape = unlocated(shape)?;
        if let Some(main) = self.cached_label(|c| &c.shape_labels, &unlocated_shape) {
            let mut users = Vec::new();
            if Self::get_users(&main, &mut users, true)? > 0 {
                for user in users {
                    if Self::shape_of(&user)?.is_same(shape) {
                        *label = user;
                        return Ok(true);
                    }
                }
            }
            if find_without_location {
                *label = main;
                return Ok(true);
            }
        }
        if self.state.read().has_simple_shapes {
            let simple = self
                .cached_label(|c| &c.simple_shapes, shape)
                .or_else(|| self.cached_label(|c| &c.simple_shapes, &unlocated_shape));
            if let Some(simple) = simple {
                *label = simple;
                return Ok(true);
            }
        }
        if !find_sub_shape {
            return Ok(false);
        }
        let main = self.find_main_shape_using_map(shape);
        if main.is_null() {
            return Ok(false);
        }
        *label = self.add_sub_shape(&main, shape)?;
        Ok(!label.is_null())
    }
    /// A label of one of the caches.
    fn cached_label(
        &self,
        map: impl Fn(&super::shape_tool::ShapeToolCaches) -> &super::shape_tool::ShapeLabelMap,
        shape: &Shape,
    ) -> Option<TdfLabel> {
        let caches = self.state.read();
        map(&caches).find(shape).ok().cloned()
    }
}

/// Sub-shapes and main shapes.
impl XCAFDocShapeTool {
    /// `IsSubShape(shapeL, sub)`: the shape is in the label's sub-shape map (built when the
    /// label has none yet).
    pub fn is_sub_shape(&self, shape_label: &TdfLabel, sub: &Shape) -> Result<bool, Failure> {
        let mut map = find::<XCAFDocShapeMapTool>(shape_label, &XCAFDocShapeMapTool::get_id())?;
        if map.is_null() {
            let shape = Self::shape_of(shape_label)?;
            if shape.is_null() {
                return Ok(false);
            }
            map = XCAFDocShapeMapTool::set(shape_label)?;
            live(&map, "XCAFDoc_ShapeTool::IsSubShape")?.set_shape(&shape);
        }
        Ok(live(&map, "XCAFDoc_ShapeTool::IsSubShape")?.is_sub_shape(sub))
    }
    /// `FindSubShape(shapeL, sub, L)`: the child of `shapeL` that holds the sub-shape.
    pub fn find_sub_shape(
        &self,
        shape_label: &TdfLabel,
        sub: &Shape,
        label: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        if sub.is_null() || !TNamingTool::has_label(&self.label(), sub)? {
            return Ok(false);
        }
        let mut transaction = 0;
        *label = TNamingTool::label(&self.label(), sub, &mut transaction)?;
        if label.is_null() {
            return Ok(false);
        }
        if label.father() == *shape_label {
            return Ok(true);
        }
        *label = TdfLabel::default();
        let mut iterator = TdfChildIterator::new(shape_label, false)?;
        while iterator.more() {
            let child = iterator.value();
            let named = find::<TNamingNamedShape>(&child, &TNamingNamedShape::get_id())?;
            if !named.is_null() {
                let shape = TNamingTool::get_shape(&named)?;
                if !shape.is_null() && shape.is_same(sub) {
                    *label = child;
                    return Ok(true);
                }
            }
            iterator.next();
        }
        Ok(false)
    }
    /// `AddSubShape(shapeL, sub)`: the new sub-shape label, null when not added.
    pub fn add_sub_shape(&self, shape_label: &TdfLabel, sub: &Shape) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        self.add_sub_shape_to(shape_label, sub, &mut label)?;
        Ok(label)
    }
    /// `AddSubShape(shapeL, sub, addedSubShapeL)`: a new child of a top-level simple shape
    /// for a sub-shape of it (an unlocated sub-shape matches a located child); false, with
    /// the existing label, when it is there already.
    pub fn add_sub_shape_to(
        &self,
        shape_label: &TdfLabel,
        sub: &Shape,
        added: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        *added = TdfLabel::default();
        if !Self::is_simple_shape(shape_label)? || !self.is_top_level(shape_label) {
            return Ok(false);
        }
        let Some(sub_shape) = self.defined_sub_shape(shape_label, sub)? else {
            return Ok(false);
        };
        if self.find_sub_shape(shape_label, &sub_shape, added)? {
            return Ok(false);
        }
        *added = TdfTagSource::new_child_of(shape_label)?;
        TNamingBuilder::new(added)?.generated(&sub_shape)?;
        Ok(true)
    }
    /// The sub-shape `AddSubShape` adds: the given one when mapped, else (unlocated) a direct
    /// child of the shape that is the same once unlocated.
    fn defined_sub_shape(
        &self,
        shape_label: &TdfLabel,
        sub: &Shape,
    ) -> Result<Option<Shape>, Failure> {
        if self.is_sub_shape(shape_label, sub)? {
            return Ok(Some(sub.clone()));
        }
        if !sub.location().is_identity() {
            return Ok(None);
        }
        let shape = Self::shape_of(shape_label)?;
        let mut iterator = ShapeIterator::new(&shape, true, true);
        while let Ok(child) = iterator.value() {
            if sub.is_same(&child.located(&Location::default(), true)?) {
                return Ok(Some(child.clone()));
            }
            iterator.next();
        }
        Ok(None)
    }
    /// `FindMainShapeUsingMap(sub)`: the cached main shape of a sub-shape, or null.
    pub fn find_main_shape_using_map(&self, sub: &Shape) -> TdfLabel {
        self.cached_label(|c| &c.sub_shapes, sub)
            .unwrap_or_default()
    }
    /// `FindMainShape(sub)`: the first top-level simple shape that has the sub-shape.
    pub fn find_main_shape(&self, sub: &Shape) -> Result<TdfLabel, Failure> {
        let mut iterator = TdfChildIterator::new(&self.label(), false)?;
        while iterator.more() {
            let label = iterator.value();
            if Self::is_simple_shape(&label)? && self.is_sub_shape(&label, sub)? {
                return Ok(label);
            }
            iterator.next();
        }
        Ok(TdfLabel::default())
    }
    /// `ComputeShapes(L)`: caches, at any depth below the label, each shape not in the shape
    /// cache as a simple shape.
    pub fn compute_shapes(&self, label: &TdfLabel) -> Result<(), Failure> {
        let mut iterator = TdfChildIterator::new(label, false)?;
        while iterator.more() {
            let child = iterator.value();
            let mut shape = Shape::default();
            if Self::get_shape(&child, &mut shape)? {
                let mut caches = self.state.write();
                if !caches.shape_labels.is_bound(&shape) {
                    caches.simple_shapes.bind(shape, child.clone());
                }
            }
            self.compute_shapes(&child)?;
            iterator.next();
        }
        Ok(())
    }
    /// `ComputeSimpleShapes`: the simple-shape cache of the whole tool.
    pub fn compute_simple_shapes(&self) -> Result<(), Failure> {
        self.compute_shapes(&self.label())?;
        self.state.write().has_simple_shapes = true;
        Ok(())
    }
}
