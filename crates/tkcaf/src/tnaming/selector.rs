// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Selector.hxx and .cxx
// (with ORIENTATION_DSOPT, which the native file defines)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Selector`: selects a shape on a label (a SELECTED named shape and an IDENTITY
//! naming of the name that re-finds it) and solves the selection again later.
use super::{
    TNamingBuilder, TNamingIdentifier, TNamingIterator, TNamingNameType, TNamingNamedShape,
    TNamingNaming, TNamingNamingTool, TNamingNewShapeIterator, TNamingTool,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{
    TdfAttributeMap, TdfAttributeState, TdfChildIdIterator, TdfLabel, TdfLabelMap, TdfTool,
};

/// `TNaming_Selector`.
///
/// ```
/// use rustycad_tkcaf::tnaming::TNamingSelector;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let selector = TNamingSelector::new(&data.root().find_child(1, true).unwrap());
/// assert!(selector.named_shape().unwrap().is_null());
/// ```
pub struct TNamingSelector {
    label: TdfLabel,
}

impl TNamingSelector {
    /// `IsIdentified(access, selection, NS, Geometry)`: the selection is already named alone:
    /// by its feature (the current shape of the feature is the selection alone), or by a
    /// generation whose generators all generated exactly it. `NS` receives the named shape.
    pub fn is_identified(
        access: &TdfLabel,
        selection: &Shape,
        named: &mut Handle<TNamingNamedShape>,
        geometry: bool,
    ) -> Result<bool, Failure> {
        let only_one = !geometry;
        let mut identifier =
            TNamingIdentifier::new(access, selection, &Shape::default(), only_one)?;
        if identifier.is_feature() {
            if !only_one {
                return Ok(false);
            }
            *named = identifier.feature_arg()?;
            // mpv: an external condition.
            let shapes = current_shapes(named)?;
            return Ok(shapes.contains(selection) && shapes.extent() == 1);
        }
        if identifier.name_type() != TNamingNameType::Generation {
            return Ok(false);
        }
        *named = identifier.named_shape_of_generation();
        if named.is_null() {
            return Ok(false);
        }
        let shapes = current_shapes(named)?;
        if !(shapes.contains(selection) && shapes.extent() == 1) {
            return Ok(false);
        }
        let mut generated = Vec::new();
        find_generated(named, &identifier.shape_arg()?, &mut generated)?;
        identifier.next_arg();
        while identifier.more_args() {
            find_generated(named, &identifier.shape_arg()?, &mut generated)?;
            identifier.next_arg();
        }
        let current = shapes.find_key(1)?;
        Ok(all_equal(&generated, current))
    }
    /// `TNaming_Selector(aLabel)`.
    pub fn new(label: &TdfLabel) -> Self {
        Self {
            label: label.clone(),
        }
    }
    /// `Select(Selection, Context, Geometry, KeepOrientation)`: forgets the label's
    /// attributes, names the selection in the context, then records it as a SELECTED named
    /// shape (the current compound of the name when the type migrates) with an IDENTITY
    /// naming. A vertex (or a compound of vertices) does not keep its orientation.
    pub fn select(
        &self,
        selection: &Shape,
        context: &Shape,
        geometry: bool,
        keep_orientation: bool,
    ) -> Result<bool, Failure> {
        self.label.forget_all_attributes(true)?;
        let keep = keep_orientation && !is_vertex_selection(selection)?;
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if keep {
            let problem = is_specific_case(&self.label, context)?
                || is_specific_case2(&self.label, selection)?;
            named = TNamingNaming::name(&self.label, selection, context, geometry, keep, problem)?;
        } else if !Self::is_identified(&self.label, selection, &mut named, geometry)? {
            named = TNamingNaming::name(&self.label, selection, context, geometry, keep, false)?;
        }
        if named.is_null() {
            return Ok(false);
        }
        self.record(selection, &named)?;
        Ok(true)
    }
    /// The end of `Select`: the SELECTED named shape and the IDENTITY naming.
    fn record(&self, selection: &Shape, named: &Handle<TNamingNamedShape>) -> Result<(), Failure> {
        let builder = TNamingBuilder::new(&self.label)?;
        // mpv: an old shape of the selection from the used shapes makes cycles (szy).
        let current = TNamingTool::current_shape(named)?;
        if current.shape_type()? == ShapeEnum::Compound
            && current.shape_type()? != selection.shape_type()?
        {
            // Type migration.
            builder.select(&current, &current)?;
        } else {
            builder.select(selection, selection)?;
        }
        let naming = Handle::new(TNamingNaming::new());
        {
            let naming_ref = live(&naming, "TNaming_Selector::Select")?;
            let mut name = naming_ref.change_name();
            name.set_type(TNamingNameType::Identity);
            name.append(named);
            name.set_orientation(selection.orientation());
            // vro 06.09.00.
            name.set_shape_type(selection.shape_type()?);
        }
        self.label.add_attribute(&naming.cast(), true)
    }
    /// `Select(Selection, Geometry, KeepOrientation)`: the selection is its own context
    /// (temporary, natively).
    pub fn select_alone(
        &self,
        selection: &Shape,
        geometry: bool,
        keep_orientation: bool,
    ) -> Result<bool, Failure> {
        self.select(selection, selection, geometry, keep_orientation)
    }
    /// `Solve(Valid)`: solves the label's naming; false without one.
    pub fn solve(&self, valid: &mut TdfLabelMap) -> Result<bool, Failure> {
        let mut naming: Handle<TNamingNaming> = Handle::default();
        if !self
            .label
            .find_attribute_as(&TNamingNaming::get_id(), &mut naming)?
        {
            return Ok(false);
        }
        live(&naming, "TNaming_Selector::Solve")?.solve(valid)
    }
    /// `Arguments(args)`: the attributes the label refers to (`TDF_Tool::OutReferences`).
    pub fn arguments(&self, arguments: &mut TdfAttributeMap) -> Result<(), Failure> {
        TdfTool::out_references(&self.label, arguments)
    }
    /// `NamedShape`: the label's named shape (null without one).
    pub fn named_shape(&self) -> Result<Handle<TNamingNamedShape>, Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        self.label
            .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        Ok(named)
    }
}

/// The current shapes of a named shape over the whole framework.
fn current_shapes(named: &Handle<TNamingNamedShape>) -> Result<IndexedMapOfShape, Failure> {
    let none = TdfLabelMap::default();
    let mut shapes = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(&none, &none, named, &mut shapes)?;
    Ok(shapes)
}

/// Every generated shape equals the current shape (with orientation); true for none.
fn all_equal(generated: &[Shape], current: &Shape) -> bool {
    let mut equal = false;
    for shape in generated {
        equal = shape == current;
        if !equal {
            break;
        }
    }
    equal
}

/// A vertex, or a compound of vertices only.
fn is_vertex_selection(selection: &Shape) -> Result<bool, Failure> {
    match selection.shape_type()? {
        ShapeEnum::Vertex => Ok(true),
        ShapeEnum::Compound => {
            for child in ShapeIterator::new(selection, true, true) {
                if child.shape_type()? != ShapeEnum::Vertex {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// The file-static `FindGenerated(NS, S, theList)`: the shapes the label of `NS` generated
/// from `S`.
fn find_generated(
    named: &Handle<TNamingNamedShape>,
    shape: &Shape,
    list: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let label = live(named, "FindGenerated")?.label();
    let mut iterator = TNamingNewShapeIterator::from_label(shape, &label)?;
    while iterator.more() {
        if iterator.label()? == label {
            list.push(iterator.shape()?);
        }
        iterator.next()?;
    }
    Ok(())
}

/// The file-static `MapOfOrientedShapes(S, M)`: the shape and every sub-shape, with
/// orientation (`NCollection_Map<TopoDS_Shape>`).
fn map_of_oriented_shapes(shape: &Shape, map: &mut Map<Shape>) {
    map.add(shape.clone());
    for child in ShapeIterator::new(shape, true, true) {
        map_of_oriented_shapes(&child, map);
    }
}

/// The file-static `BuildAtomicMap(S, M)`: the simple shapes of a (nested) collection.
fn build_atomic_map(shape: &Shape, map: &mut Map<Shape>) -> Result<(), Failure> {
    if (shape.shape_type()? as i32) > ShapeEnum::CompSolid as i32 {
        return Ok(());
    }
    for child in ShapeIterator::new(shape, true, true) {
        if (child.shape_type()? as i32) > ShapeEnum::CompSolid as i32 {
            map.add(child);
        } else {
            build_atomic_map(&child, map)?;
        }
    }
    Ok(())
}

/// The file-static `FindPrevNDS(CNS)`: the named shape of the first old shape of `CNS`.
fn find_prev_nds(
    context: &Handle<TNamingNamedShape>,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let iterator = TNamingIterator::new(context)?;
    if iterator.more() {
        let old = iterator.old_shape()?;
        if !old.is_null() {
            return TNamingTool::named_shape(&old, &live(context, "FindPrevNDS")?.label());
        }
    }
    Ok(Handle::default())
}

/// The file-static `IsSpecificCase(F, Context)`: a current shape under the context's label
/// (or under the label of its previous state) is not a sub-shape of the context with its
/// orientation; true when the context's label has no named shape below and no previous one.
fn is_specific_case(access: &TdfLabel, context: &Shape) -> Result<bool, Failure> {
    let mut oriented = Map::<Shape>::default();
    map_of_oriented_shapes(context, &mut oriented);
    let named = TNamingTool::named_shape(context, access)?;
    let Some(named_ref) = named.get() else {
        return Ok(false);
    };
    let id = TNamingNamedShape::get_id();
    let mut children = TdfChildIdIterator::new(&named_ref.label(), &id, false)?;
    if !children.more() {
        // The naming data structure is empty: look at the old shape's one.
        let previous = find_prev_nds(&named)?;
        let Some(previous) = previous.get() else {
            return Ok(true);
        };
        children.initialize(&previous.label(), &id, false)?;
    }
    let mut found = false;
    while children.more() {
        let child = children.value().cast::<TNamingNamedShape>();
        if child.is_some() && !all_in_context(&child, &oriented)? {
            found = true;
            break;
        }
        children.next()?;
    }
    Ok(found)
}

/// One child of `IsSpecificCase`: its current shape (or every simple shape of a compound) is
/// in the context.
fn all_in_context(
    child: &Handle<TNamingNamedShape>,
    oriented: &Map<Shape>,
) -> Result<bool, Failure> {
    let shape = TNamingTool::current_shape(child)?;
    if shape.is_null() {
        return Ok(true);
    }
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(oriented.contains(&shape));
    }
    let mut atoms = Map::<Shape>::default();
    build_atomic_map(&shape, &mut atoms)?;
    Ok(atoms.iter().all(|atom| oriented.contains(atom)))
}

/// The file-static `IsSpecificCase2(F, Selection)`: an edge whose current shape is an edge of
/// another orientation.
fn is_specific_case2(access: &TdfLabel, selection: &Shape) -> Result<bool, Failure> {
    if selection.shape_type()? != ShapeEnum::Edge {
        return Ok(false);
    }
    let named = TNamingTool::named_shape(selection, access)?;
    if named.is_null() {
        return Ok(false);
    }
    let current = TNamingTool::current_shape(&named)?;
    Ok(!current.is_null()
        && current.shape_type()? == selection.shape_type()?
        && selection.orientation() != current.orientation())
}
