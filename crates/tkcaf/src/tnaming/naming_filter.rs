// Rust port of the file-static Filter of ApplicationFramework/TKCAF/TNaming/TNaming_Naming.cxx
// (with ALLOW_CHILD_NBS, which the native file defines)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The filter by neighbours: when a name is ambiguous, the selection is told apart by the
//! names of the shapes around it.
use super::naming_build::build_name;
use super::naming_checks::{compare, is_cont_same, is_multiple_case, next_modif};
use super::{
    TNamingLocalizer, TNamingNameType, TNamingNamedShape, TNamingNaming, TNamingScope, TNamingTool,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// The inputs of one filter.
pub(crate) struct FilterRequest<'a> {
    /// `F`: the label the filter naming goes under.
    pub(crate) father: &'a TdfLabel,
    /// `S`: the selection.
    pub(crate) shape: &'a Shape,
    /// `Context`.
    pub(crate) context: &'a Shape,
    /// `Lev`: the depth of nested filters (at most 3).
    pub(crate) level: i32,
}

/// The file-static `Filter(F, MDF, S, Context, Localizer, NS, Lev)`: a FILTERBYNEIGHBOURGS
/// naming under `F` that keeps, among the shapes of `NS`, the one connected to the named
/// neighbours of the selection; `NS` becomes its result. True when it selects `S` alone.
pub(crate) fn filter(
    request: &FilterRequest<'_>,
    scope: &mut TNamingScope,
    localizer: &mut TNamingLocalizer,
    named: &mut Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    let (father, shape, context) = (request.father, request.shape, request.context);
    let mut neighbours = MapOfShape::default();
    localizer.find_neighbourg(context, shape, &mut neighbours)?;
    // mpv: the named shape and the shape must be the same.
    if !is_new_shape_of(named, shape)? {
        let made_by = TNamingTool::named_shape(shape, father)?;
        if made_by.is_some() {
            *named = made_by;
        }
    }
    if neighbours.is_empty() {
        // The search of the true context.
        let Some(context) = true_context(named, shape)? else {
            return Ok(false);
        };
        localizer.find_neighbourg(&context, shape, &mut neighbours)?;
    }
    if neighbours.is_empty() {
        return Ok(false);
    }
    let level = request.level + 1;
    if level > 3 {
        return Ok(false);
    }
    let mut father_naming: Handle<TNamingNaming> = Handle::default();
    father.find_attribute_as(&TNamingNaming::get_id(), &mut father_naming)?;
    let father_candidate = father_candidate(&father_naming)?;
    if shape.shape_type()? == ShapeEnum::Edge
        && father_candidate.is_null()
        && is_compound_named(named)?
        && is_multiple_case(shape, context, &neighbours)?
    {
        return multiple_case(scope, shape, context, father_naming, named);
    }
    filter_by_neighbours(request, level, scope, &neighbours, &father_candidate, named)
}

/// The `mpv` check: the selection is a new shape of the named shape.
fn is_new_shape_of(named: &Handle<TNamingNamedShape>, shape: &Shape) -> Result<bool, Failure> {
    let mut iterator = super::TNamingIterator::new(named)?;
    while iterator.more() {
        if iterator.new_shape()?.is_same(shape) {
            return Ok(true);
        }
        iterator.next()?;
    }
    Ok(false)
}

/// The search of the true context: the shape of the father of the label that made `S`.
fn true_context(
    named: &Handle<TNamingNamedShape>,
    shape: &Shape,
) -> Result<Option<Shape>, Failure> {
    let label = live(named, "TNaming_Naming Filter")?.label();
    let made_by = TNamingTool::named_shape(shape, &label)?;
    let Some(made_by) = made_by.get() else {
        return Ok(None);
    };
    let father = made_by.label().father();
    let mut father_named: Handle<TNamingNamedShape> = Handle::default();
    father.find_attribute_as(&TNamingNamedShape::get_id(), &mut father_named)?;
    // Natively the named shape of `S` is kept when the father has none.
    let context = if father_named.is_some() {
        father_named
    } else {
        TNamingTool::named_shape(shape, &label)?
    };
    Ok(Some(TNamingTool::get_shape(&context)?))
}

/// The shape of the first argument of the naming of `F`, when it is a filter.
fn father_candidate(naming: &Handle<TNamingNaming>) -> Result<Shape, Failure> {
    let Some(naming) = naming.get() else {
        return Ok(Shape::default());
    };
    let name = naming.get_name();
    if name.name_type() != TNamingNameType::FilterByNeighbourgs {
        return Ok(Shape::default());
    }
    let first = super::name_solve::argument(name.arguments(), 0)?;
    live(first, "TNaming_Naming Filter")?.get()
}

/// The applicability test of the multiple case: the named shape holds a compound.
fn is_compound_named(named: &Handle<TNamingNamedShape>) -> Result<bool, Failure> {
    let Some(named) = named.get() else {
        return Ok(false);
    };
    let shape = named.get()?;
    Ok(!shape.is_null() && shape.shape_type()? == ShapeEnum::Compound)
}

/// The multiple case: the edge is identified by its position in a face of the intersection
/// naming (encoded in the name's index), then the name is regenerated. Natively the naming
/// variable still holds the father's naming when the label of `NS` has none, and that one is
/// used then.
fn multiple_case(
    scope: &mut TNamingScope,
    shape: &Shape,
    context: &Shape,
    father_naming: Handle<TNamingNaming>,
    named: &mut Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    let label = live(named, "TNaming_Naming Filter")?.label();
    let mut naming = father_naming;
    label.find_attribute_as(&TNamingNaming::get_id(), &mut naming)?;
    let Some(naming_ref) = naming.get() else {
        return Ok(false);
    };
    let name = naming_ref.get_name();
    if name.name_type() != TNamingNameType::Intersection {
        return Ok(false);
    }
    for (position, argument) in name.arguments().iter().enumerate() {
        let face = TNamingTool::current_shape(argument)?;
        let Some(index) = edge_position(&face, shape, position + 1) else {
            continue;
        };
        naming_ref.change_name().set_index(index);
        naming_ref.regenerate(scope.get_valid());
        naming_ref
            .label()
            .find_attribute_as(&TNamingNamedShape::get_id(), named)?;
        let until =
            TNamingTool::named_shape(context, &live(named, "TNaming_Naming Filter")?.label())?;
        let stop = next_modif(&until)?;
        return compare(named, scope, &stop, shape);
    }
    Ok(false)
}

/// The index of the multiple case: edge position, number of edges seen, wire position,
/// number of wires and face position, packed in bytes and nibbles as natively (the edge
/// count runs over every wire up to the one holding the edge).
fn edge_position(face: &Shape, edge: &Shape, face_position: usize) -> Option<i32> {
    let (mut wire_count, mut edge_count) = (0, 0);
    let mut found: Option<(i32, i32)> = None;
    for (wire_position, wire) in ShapeIterator::new(face, true, true).enumerate() {
        wire_count += 1;
        if found.is_some() {
            continue;
        }
        for (position, candidate) in ShapeIterator::new(&wire, true, true).enumerate() {
            edge_count += 1;
            if edge.is_equal(&candidate) {
                found = Some((position as i32 + 1, wire_position as i32 + 1));
            }
        }
    }
    let (edge_index, wire_index) = found?;
    let face_index = i32::try_from(face_position).unwrap_or(i32::MAX);
    Some(
        (edge_index & 0xFF)
            | (edge_count << 8)
            | (wire_index << 16)
            | (wire_count << 20)
            | (face_index << 24),
    )
}

/// The standard filter: a FILTERBYNEIGHBOURGS naming with the named shape and the names of the
/// neighbours (a neighbour named by a compound is filtered again, child level).
fn filter_by_neighbours(
    request: &FilterRequest<'_>,
    level: i32,
    scope: &mut TNamingScope,
    neighbours: &MapOfShape,
    father_candidate: &Shape,
    named: &mut Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    let (shape, context) = (request.shape, request.context);
    let naming = TNamingNaming::insert(request.father)?;
    let naming_ref = live(&naming, "TNaming_Naming Filter")?;
    let label = live(named, "TNaming_Naming Filter")?.label();
    let until = TNamingTool::named_shape(context, &label)?;
    let stop = next_modif(&until)?;
    {
        let mut name = naming_ref.change_name();
        name.set_shape_type(shape.shape_type()?);
        name.set_shape(shape);
        name.set_orientation(shape.orientation());
        name.set_type(TNamingNameType::FilterByNeighbourgs);
        name.append(named);
        name.set_stop_named_shape(&until);
    }
    let filter_label = naming_ref.label();
    for neighbour in neighbours.iter() {
        let mut neighbour_named =
            build_name(&filter_label, scope, neighbour, context, &stop, true)?;
        let child = ChildFilter {
            filter_label: &filter_label,
            context,
            stop: &stop,
            level,
        };
        child.refine(scope, neighbour, &mut neighbour_named, father_candidate)?;
        naming_ref.change_name().append(&neighbour_named);
    }
    naming_ref.regenerate(scope.get_valid());
    naming_ref
        .label()
        .find_attribute_as(&TNamingNamedShape::get_id(), named)?;
    compare(named, scope, &stop, shape)
}

/// The child-level filter of a neighbour (ALLOW_CHILD_NBS).
struct ChildFilter<'a> {
    filter_label: &'a TdfLabel,
    context: &'a Shape,
    stop: &'a Handle<TNamingNamedShape>,
    level: i32,
}

impl ChildFilter<'_> {
    /// A neighbour named by a compound (not the father's own candidate) whose naming is not a
    /// generation and does not name it alone is filtered again under the filter naming; the
    /// filter's result replaces the neighbour's named shape.
    fn refine(
        &self,
        scope: &mut TNamingScope,
        neighbour: &Shape,
        neighbour_named: &mut Handle<TNamingNamedShape>,
        father_candidate: &Shape,
    ) -> Result<(), Failure> {
        let named_shape = live(neighbour_named, "TNaming_Naming Filter")?.get()?;
        let mut allow_child = true;
        if !named_shape.is_null()
            && named_shape.shape_type()? == ShapeEnum::Compound
            && !father_candidate.is_null()
        {
            allow_child = !is_cont_same(father_candidate, &named_shape)?;
        }
        let compound = !named_shape.is_null()
            && neighbour.shape_type()? != named_shape.shape_type()?
            && named_shape.shape_type()? == ShapeEnum::Compound;
        if !allow_child || !compound {
            return Ok(());
        }
        let mut naming: Handle<TNamingNaming> = Handle::default();
        live(neighbour_named, "TNaming_Naming Filter")?
            .find_attribute_as(&TNamingNaming::get_id(), &mut naming)?;
        let Some(naming) = naming.get() else {
            return Ok(());
        };
        if naming.get_name().name_type() == TNamingNameType::Generation {
            return Ok(());
        }
        if compare(neighbour_named, scope, self.stop, neighbour)? {
            return Ok(());
        }
        let mut localizer = TNamingLocalizer::new();
        let request = FilterRequest {
            father: self.filter_label,
            shape: neighbour,
            context: self.context,
            level: self.level,
        };
        filter(&request, scope, &mut localizer, neighbour_named)?;
        Ok(())
    }
}
