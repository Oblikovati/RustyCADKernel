// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Localizer.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Localizer`: finds where a shape comes from (its ancestors in a context, its
//! generators, the named shapes it goes back to), with caches of sub-shapes and ancestors.
use super::{
    TNamingEvolution, TNamingIterator, TNamingNamedShape, TNamingNamedShapeMap,
    TNamingOldShapeIterator, TNamingTool, TNamingUsedShapes,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// `TNaming_Localizer`. The native default constructor leaves the transaction
/// uninitialized; the port starts at 0.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::TNamingLocalizer;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let mut localizer = TNamingLocalizer::new();
/// assert!(localizer.sub_shapes(&Shape::default(), ShapeEnum::Edge).unwrap().is_empty());
/// ```
#[derive(Default)]
pub struct TNamingLocalizer {
    cur_trans: i32,
    used_shapes: Handle<TNamingUsedShapes>,
    shape_with_sub_shapes: Vec<Shape>,
    sub_shapes: Vec<MapOfShape>,
    shape_with_ancestors: Vec<Shape>,
    ancestors: Vec<IndexedDataMapOfShapeListOfShape>,
}

impl TNamingLocalizer {
    /// `TNaming_Localizer()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Init(US, CurTrans)`.
    pub fn init(&mut self, used_shapes: &Handle<TNamingUsedShapes>, cur_trans: i32) {
        self.used_shapes = used_shapes.clone();
        self.cur_trans = cur_trans;
    }
    /// `SubShapes(In, TS)`: the cached sub-shapes of `In` of a type (the exploration stops at
    /// the first sub-shape already cached, as natively).
    pub fn sub_shapes(&mut self, shape: &Shape, kind: ShapeEnum) -> Result<&MapOfShape, Failure> {
        let index = match self
            .shape_with_sub_shapes
            .iter()
            .position(|s| shape.is_same(s))
        {
            Some(index) => index,
            None => {
                self.shape_with_sub_shapes.insert(0, shape.clone());
                self.sub_shapes.insert(0, MapOfShape::default());
                0
            }
        };
        fill_sub_shapes(shape, kind, &mut self.sub_shapes[index])?;
        Ok(&self.sub_shapes[index])
    }
    /// `Ancestors(In, TS)`: the cached map of the sub-shapes of a type of `In` to their
    /// ancestors (edges for vertices, faces for edges, solids for faces), completed when the
    /// first sub-shape of the type is missing.
    pub fn ancestors(
        &mut self,
        shape: &Shape,
        kind: ShapeEnum,
    ) -> Result<&IndexedDataMapOfShapeListOfShape, Failure> {
        if let Some(index) = self
            .shape_with_ancestors
            .iter()
            .position(|s| shape.is_same(s))
        {
            complete_ancestors(shape, kind, &mut self.ancestors[index])?;
            return Ok(&self.ancestors[index]);
        }
        self.shape_with_ancestors.insert(0, shape.clone());
        self.ancestors
            .insert(0, IndexedDataMapOfShapeListOfShape::default());
        let ancestor_kind = match kind {
            ShapeEnum::Vertex => ShapeEnum::Edge,
            ShapeEnum::Edge => ShapeEnum::Face,
            ShapeEnum::Face => ShapeEnum::Solid,
            _ => ShapeEnum::Compound,
        };
        let buildable = matches!(kind, ShapeEnum::Edge | ShapeEnum::Vertex | ShapeEnum::Face);
        if buildable && ancestor_kind as i32 >= shape.shape_type()? as i32 {
            topexp::map_shapes_and_ancestors(shape, kind, ancestor_kind, &mut self.ancestors[0])?;
        }
        Ok(&self.ancestors[0])
    }
    /// `FindFeaturesInAncestors(S, In, AncInFeatures)`: the ancestors of `S` in the context
    /// that have a named shape, going up through the unnamed ones below the faces.
    pub fn find_features_in_ancestors(
        &mut self,
        shape: &Shape,
        context: &Shape,
        found: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let ancestors = self.ancestors(context, shape.shape_type()?)?;
        let Some(list) = ancestors.seek(shape) else {
            // S is not in the solid.
            return Ok(());
        };
        let list: Vec<Shape> = list.iter().cloned().collect();
        let access = live(
            &self.used_shapes,
            "TNaming_Localizer::FindFeaturesInAncestors",
        )?
        .label();
        for ancestor in &list {
            if TNamingTool::named_shape(ancestor, &access)?.is_some() {
                found.add(ancestor.clone());
            } else if ancestor.shape_type()? as i32 > ShapeEnum::Face as i32 {
                self.find_features_in_ancestors(ancestor, context, found)?;
            }
        }
        Ok(())
    }
    /// `GoBack(S, Lab, Evol, OldS, OldLab)`: the shapes `S` was made from by evolutions of
    /// kind `Evol` (with their named shapes); without any, the named ancestors of `S` in the
    /// old shape of the father label.
    pub fn go_back(
        &mut self,
        shape: &Shape,
        label: &TdfLabel,
        evolution: TNamingEvolution,
        old_shapes: &mut Vec<Shape>,
        old_named: &mut Vec<Handle<TNamingNamedShape>>,
    ) -> Result<(), Failure> {
        let mut iterator = TNamingOldShapeIterator::from_used_shapes_in_transaction(
            shape,
            self.cur_trans,
            &self.used_shapes,
        )?;
        if !iterator.more() {
            return self.go_back_in_father(shape, label, old_shapes, old_named);
        }
        while iterator.more() {
            let named = iterator.named_shape()?;
            if live(&named, "TNaming_Localizer::GoBack")?.evolution() == evolution {
                let old_shape = iterator.shape()?;
                if TNamingTool::named_shape(&old_shape, label)?.is_some() {
                    old_named.push(TNamingTool::named_shape(&old_shape, label)?);
                    old_shapes.push(old_shape);
                }
            }
            iterator.next()?;
        }
        Ok(())
    }
    /// The branch of `GoBack` for a shape without ascendant.
    fn go_back_in_father(
        &mut self,
        shape: &Shape,
        label: &TdfLabel,
        old_shapes: &mut Vec<Shape>,
        old_named: &mut Vec<Handle<TNamingNamedShape>>,
    ) -> Result<(), Failure> {
        let father_names = TNamingIterator::from_label(&label.father())?;
        let solid = if father_names.more() {
            father_names.old_shape()?
        } else {
            Shape::default()
        };
        if solid.is_null() {
            return Ok(());
        }
        let mut found = MapOfShape::default();
        self.find_features_in_ancestors(shape, &solid, &mut found)?;
        for ancestor in found.iter() {
            old_shapes.push(ancestor.clone());
            old_named.push(TNamingTool::named_shape(ancestor, label)?);
        }
        Ok(())
    }
    /// `Backward(NS, S, Primitives, Shapes)`: goes back from `S` in `NS` to the primitive
    /// named shapes and to the generated shapes it comes from.
    pub fn backward(
        &mut self,
        named: &Handle<TNamingNamedShape>,
        shape: &Shape,
        primitives: &mut TNamingNamedShapeMap,
        shapes: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let named_ref = live(named, "TNaming_Localizer::Backward")?;
        let mut old_shapes = Vec::new();
        let mut old_named = Vec::new();
        self.go_back(
            shape,
            &named_ref.label(),
            named_ref.evolution(),
            &mut old_shapes,
            &mut old_named,
        )?;
        if old_shapes.is_empty() {
            primitives.add(named.clone());
        }
        for (old_shape, origin) in old_shapes.iter().zip(&old_named) {
            match live(origin, "TNaming_Localizer::Backward")?.evolution() {
                TNamingEvolution::Primitive => {
                    primitives.add(origin.clone());
                }
                TNamingEvolution::Generated => {
                    shapes.add(old_shape.clone());
                }
                _ => self.backward(origin, old_shape, primitives, shapes)?,
            }
        }
        Ok(())
    }
    /// `FindNeighbourg(Cont, S, Neighbourg)`: the shapes of the context that share a
    /// sub-shape (an edge of a face, a vertex of an edge or of a vertex) with `S`.
    pub fn find_neighbourg(
        &mut self,
        context: &Shape,
        shape: &Shape,
        neighbours: &mut MapOfShape,
    ) -> Result<(), Failure> {
        if context.is_null() || shape.is_null() {
            return Ok(());
        }
        let sub_kind = match shape.shape_type()? {
            ShapeEnum::Face => ShapeEnum::Edge,
            // szy 30.03.10: a vertex candidate uses its vertices.
            ShapeEnum::Edge | ShapeEnum::Vertex => ShapeEnum::Vertex,
            _ => ShapeEnum::Compound,
        };
        let ancestors = self.ancestors(context, sub_kind)?;
        let mut explorer = Explorer::new(shape, sub_kind, ShapeEnum::Shape)?;
        while explorer.more() {
            let Some(list) = ancestors.seek(explorer.current()) else {
                break;
            };
            for ancestor in list.iter().filter(|ancestor| !ancestor.is_same(shape)) {
                neighbours.add(ancestor.clone());
            }
            explorer.next()?;
        }
        Ok(())
    }
    /// `IsNew(S, NS)`: `S` is a new shape of `NS` before being an old one.
    pub fn is_new(shape: &Shape, named: &Handle<TNamingNamedShape>) -> Result<bool, Failure> {
        let mut iterator = TNamingIterator::new(named)?;
        while iterator.more() {
            if iterator.old_shape()?.is_same(shape) {
                return Ok(false);
            }
            if iterator.new_shape()?.is_same(shape) {
                return Ok(true);
            }
            iterator.next()?;
        }
        Ok(false)
    }
    /// `FindGenerator(NS, S, theListOfGenerators)`: the old shapes `NS` generated `S` from
    /// (szy 16.10.03: all of them).
    pub fn find_generator(
        named: &Handle<TNamingNamedShape>,
        shape: &Shape,
        generators: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let label = live(named, "TNaming_Localizer::FindGenerator")?.label();
        let used = super::named_shape::used_shapes_of(&label)?;
        let mut iterator = TNamingOldShapeIterator::from_used_shapes(shape, &used)?;
        while iterator.more() {
            if iterator.label()? == label {
                generators.push(iterator.shape()?);
            }
            iterator.next()?;
        }
        Ok(())
    }
    /// `FindShapeContext(NS, theS, theSC)`: the first old shape of the father label that
    /// contains `theS` (szy), replaced by the first new shape of the named shape of the
    /// father of its own named shape, when there is one.
    pub fn find_shape_context(
        named: &Handle<TNamingNamedShape>,
        shape: &Shape,
        context: &mut Shape,
    ) -> Result<(), Failure> {
        let father = live(named, "TNaming_Localizer::FindShapeContext")?
            .label()
            .father();
        let mut olds = Vec::new();
        let mut iterator = TNamingIterator::from_label(&father)?;
        while iterator.more() {
            olds.push(iterator.old_shape()?);
            iterator.next()?;
        }
        for old in olds {
            *context = old;
            if !context.is_null() && contains_same(context, shape)? {
                break;
            }
        }
        if context.is_null() {
            return Ok(());
        }
        if let Some(new_shape) = context_of_context(context, &father)? {
            *context = new_shape;
        }
        Ok(())
    }
}

/// The exploration of `SubShapes`: adds sub-shapes until one is already there.
fn fill_sub_shapes(
    shape: &Shape,
    kind: ShapeEnum,
    sub_shapes: &mut MapOfShape,
) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        if !sub_shapes.add(explorer.current().clone()) {
            break;
        }
        explorer.next()?;
    }
    Ok(())
}

/// The completion branch of `Ancestors` for a cached shape: when the first sub-shape of the
/// type (the explored shape itself when there is none) is missing, the ancestors of the type
/// (faces for edges, edges for vertices, else faces) are added.
fn complete_ancestors(
    shape: &Shape,
    kind: ShapeEnum,
    ancestors: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    let explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    if ancestors.contains(explorer.current()) {
        return Ok(());
    }
    let ancestor_kind = if kind == ShapeEnum::Vertex {
        ShapeEnum::Edge
    } else {
        ShapeEnum::Face
    };
    if ancestor_kind as i32 >= shape.shape_type()? as i32 {
        topexp::map_shapes_and_ancestors(shape, kind, ancestor_kind, ancestors)?;
    }
    Ok(())
}

/// The context test of `FindShapeContext`: a container that has `shape` among its
/// sub-shapes of that type.
fn contains_same(context: &Shape, shape: &Shape) -> Result<bool, Failure> {
    let kind = shape.shape_type()?;
    if context.shape_type()? as i32 >= kind as i32 {
        return Ok(false);
    }
    let mut explorer = Explorer::new(context, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        if explorer.current().is_same(shape) {
            return Ok(true);
        }
        explorer.next()?;
    }
    Ok(false)
}

/// The end of `FindShapeContext`: the first new shape of the named shape on the father of
/// the label that made the context.
fn context_of_context(context: &Shape, father: &TdfLabel) -> Result<Option<Shape>, Failure> {
    let made_by = TNamingTool::named_shape(context, father)?;
    let Some(made_by) = made_by.get() else {
        return Ok(None);
    };
    let mut upper: Handle<TNamingNamedShape> = Handle::default();
    if !made_by
        .label()
        .father()
        .find_attribute_as(&TNamingNamedShape::get_id(), &mut upper)?
    {
        return Ok(None);
    }
    let mut iterator = TNamingIterator::from_label(&live(&upper, "FindShapeContext")?.label())?;
    while iterator.more() {
        let new_shape = iterator.new_shape()?;
        if !new_shape.is_null() {
            return Ok(Some(new_shape));
        }
        iterator.next()?;
    }
    Ok(None)
}
