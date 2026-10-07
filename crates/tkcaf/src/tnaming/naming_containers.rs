// Rust port of the container naming of ApplicationFramework/TKCAF/TNaming/TNaming_Naming.cxx
// (HasAncFace, BuildNameWire, HasAncSolid, BuildNameShell, BuildAggregationName, IsAllIn)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Naming wires, shells and compounds: by their face or solid (WIREIN, SHELLIN) or as the
//! union of their elements.
use super::naming_build::build_name;
use super::naming_checks::{build_ns, explore, test_solution};
use super::{
    TNaming, TNamingName, TNamingNameType, TNamingNamedShape, TNamingNaming, TNamingScope,
    TNamingSelector, TNamingTool,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// The common inputs of the container namings.
pub(crate) struct ContainerRequest<'a> {
    /// `F`.
    pub(crate) father: &'a TdfLabel,
    /// `Selection`.
    pub(crate) selection: &'a Shape,
    /// `Context`.
    pub(crate) context: &'a Shape,
    /// `Stop`.
    pub(crate) stop: &'a Handle<TNamingNamedShape>,
    /// `Geom`.
    pub(crate) geometry: bool,
}

impl ContainerRequest<'_> {
    /// The naming of `F`, created with the selection's type, shape and orientation when it is
    /// missing.
    fn naming(&self) -> Result<Handle<TNamingNaming>, Failure> {
        let mut naming: Handle<TNamingNaming> = Handle::default();
        if self
            .father
            .find_attribute_as(&TNamingNaming::get_id(), &mut naming)?
        {
            return Ok(naming);
        }
        naming = Handle::new(TNamingNaming::new());
        self.father.add_attribute(&naming.cast(), true)?;
        let naming_ref = live(&naming, "BuildName")?;
        let mut name = naming_ref.change_name();
        name.set_shape_type(self.selection.shape_type()?);
        name.set_shape(self.selection);
        name.set_orientation(self.selection.orientation());
        drop(name);
        Ok(naming)
    }
    /// `BuildName(label, MDF, shape, Context, Stop, Geom)` under another label.
    fn name_under(
        &self,
        scope: &mut TNamingScope,
        label: &TdfLabel,
        shape: &Shape,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        build_name(label, scope, shape, self.context, self.stop, self.geometry)
    }
    /// The named shape of the naming's label.
    fn result(naming: &TNamingNaming) -> Result<Handle<TNamingNamedShape>, Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        naming
            .label()
            .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        Ok(named)
    }
}

/// Appends an argument to a naming's name.
fn append(naming: &TNamingNaming, argument: &Handle<TNamingNamedShape>) {
    naming.change_name().append(argument);
}

/// Changes a naming's name.
fn edit(naming: &TNamingNaming, change: impl FnOnce(&mut TNamingName)) {
    change(&mut naming.change_name());
}

/// The edges of a wire that are not null or degenerated.
fn named_edges(wire: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut edges = Vec::new();
    for edge in explore(wire, ShapeEnum::Edge)? {
        if edge.is_null() || BRepTool::degenerated(&topods::edge(&edge)?)? {
            continue;
        }
        edges.push(edge);
    }
    Ok(edges)
}

/// The file-static `HasAncFace(Context, W, Face, isOuter)`: the face of the context that has
/// the wire (with its orientation), and whether the wire is its outer wire.
fn has_anc_face(context: &Shape, wire: &Shape) -> Result<Option<(Shape, bool)>, Failure> {
    if wire.shape_type()? != ShapeEnum::Wire {
        return Ok(None);
    }
    for face in explore(context, ShapeEnum::Face)? {
        if ShapeIterator::new(&face, true, true).any(|child| child.is_equal(wire))
            && !face.is_null()
        {
            let mut outer = Wire::default();
            let is_outer = TNaming::outer_wire(&topods::face(&face)?, &mut outer)?
                && !outer.is_null()
                && outer.is_equal(wire);
            return Ok(Some((face, is_outer)));
        }
    }
    Ok(None)
}

/// The file-static `BuildNameWire(F, MDF, Selection, Context, Stop, Geom)`: a WIREIN name
/// (the face and the outer index, or the names of the edges) when the wire is in a face of
/// the context, else the UNION of the names of its edges.
pub(crate) fn build_name_wire(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let naming = request.naming()?;
    let naming_ref = live(&naming, "BuildNameWire")?;
    let label = naming_ref.label();
    let anc_face = has_anc_face(request.context, request.selection)?;
    let context_type = request.context.shape_type()?;
    let Some((face, is_outer)) = anc_face.filter(|_| {
        request
            .selection
            .shape_type()
            .is_ok_and(|t| t as i32 > context_type as i32)
    }) else {
        // No face.
        edit(naming_ref, |name| name.set_type(TNamingNameType::Union));
        for edge in named_edges(request.selection)? {
            append(naming_ref, &request.name_under(scope, &label, &edge)?);
        }
        return ContainerRequest::result(naming_ref);
    };
    edit(naming_ref, |name| name.set_type(TNamingNameType::WireIn));
    if context_type == ShapeEnum::Face {
        let mut identified: Handle<TNamingNamedShape> = Handle::default();
        let mut found = false;
        for child in ShapeIterator::new(request.context, true, true) {
            if child.is_equal(request.selection)
                && TNamingSelector::is_identified(
                    request.father,
                    request.context,
                    &mut identified,
                    request.geometry,
                )?
            {
                append(naming_ref, &identified);
                found = true;
                break;
            }
        }
        if !found {
            return build_ns(request.father, request.selection, TNamingNameType::Unknown);
        }
        append(naming_ref, &request.name_under(scope, &label, &face)?);
        wire_edges(request, naming_ref, is_outer, true)?;
    } else {
        // The context is not a face.
        append(naming_ref, &request.name_under(scope, &label, &face)?);
        wire_edges(request, naming_ref, is_outer, false)?;
    }
    ContainerRequest::result(naming_ref)
}

/// The end of the WIREIN branches: index 1 for the outer wire, else (index -1 when the
/// context is the face) the names of the edges with orientation.
fn wire_edges(
    request: &ContainerRequest<'_>,
    naming: &TNamingNaming,
    is_outer: bool,
    in_face: bool,
) -> Result<(), Failure> {
    if is_outer {
        edit(naming, |name| name.set_index(1));
        return Ok(());
    }
    if in_face {
        edit(naming, |name| name.set_index(-1));
    }
    let label = naming.label();
    for edge in named_edges(request.selection)? {
        let named = TNamingNaming::name(
            &label,
            &edge,
            request.context,
            request.geometry,
            true,
            false,
        )?;
        append(naming, &named);
    }
    Ok(())
}

/// The file-static `HasAncSolid(Context, Sh, Solid, isOuter)`: the solid of the context that
/// has the shell (with its orientation), and whether it is its outer shell.
fn has_anc_solid(context: &Shape, shell: &Shape) -> Result<Option<(Shape, bool)>, Failure> {
    if shell.shape_type()? != ShapeEnum::Shell {
        return Ok(None);
    }
    for solid in explore(context, ShapeEnum::Solid)? {
        if ShapeIterator::new(&solid, true, true).any(|child| child.is_equal(shell))
            && !solid.is_null()
        {
            let mut outer = Shell::default();
            let is_outer = TNaming::outer_shell(&topods::solid(&solid)?, &mut outer)?
                && !outer.is_null()
                && outer.is_equal(shell);
            return Ok(Some((solid, is_outer)));
        }
    }
    Ok(None)
}

/// The file-static `BuildNameShell(F, MDF, Selection, Context, Stop, Geom)`: a SHELLIN name (the
/// solid and the outer index, or the names of the faces) when the shell is in a solid of the
/// context, else the UNION of the names of its faces.
pub(crate) fn build_name_shell(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let naming = request.naming()?;
    let naming_ref = live(&naming, "BuildNameShell")?;
    let label = naming_ref.label();
    let context_type = request.context.shape_type()?;
    let anc_solid = has_anc_solid(request.context, request.selection)?;
    let Some((solid, is_outer)) = anc_solid.filter(|_| {
        request
            .selection
            .shape_type()
            .is_ok_and(|t| t as i32 > context_type as i32)
    }) else {
        // No solid.
        edit(naming_ref, |name| name.set_type(TNamingNameType::Union));
        set_context_label(request, naming_ref)?;
        name_faces(request, scope, naming_ref, &label)?;
        return ContainerRequest::result(naming_ref);
    };
    edit(naming_ref, |name| name.set_type(TNamingNameType::ShellIn));
    if context_type == ShapeEnum::Solid {
        let found = ShapeIterator::new(request.context, true, true)
            .any(|child| child.is_equal(request.selection));
        if !found {
            return build_ns(request.father, request.selection, TNamingNameType::Unknown);
        }
        // The solid is also the context.
        let context_named = TNamingTool::named_shape(request.context, request.father)?;
        if let Some(context_named_ref) = context_named.get() {
            let context_label = context_named_ref.label();
            edit(naming_ref, |name| name.set_context_label(&context_label));
        }
        append(naming_ref, &context_named);
        if is_outer {
            edit(naming_ref, |name| name.set_index(1));
        } else {
            edit(naming_ref, |name| name.set_index(-1));
            name_faces(request, scope, naming_ref, &label)?;
        }
    } else {
        // The context is not a solid.
        let index = if is_outer { 1 } else { -1 };
        edit(naming_ref, |name| name.set_index(index));
        let solid_named = name_solid_as_union(request, scope, &solid)?;
        append(naming_ref, &solid_named);
        if !is_outer {
            name_faces(request, scope, naming_ref, &label)?;
        }
    }
    ContainerRequest::result(naming_ref)
}

/// The context label of a naming: the label of the context's named shape, when it has one.
fn set_context_label(
    request: &ContainerRequest<'_>,
    naming: &TNamingNaming,
) -> Result<(), Failure> {
    let context_named = TNamingTool::named_shape(request.context, request.father)?;
    if let Some(context_named) = context_named.get() {
        let context_label = context_named.label();
        edit(naming, |name| name.set_context_label(&context_label));
    }
    Ok(())
}

/// The names of the faces of the shell, appended.
fn name_faces(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
    naming: &TNamingNaming,
    label: &TdfLabel,
) -> Result<(), Failure> {
    for face in explore(request.selection, ShapeEnum::Face)? {
        if face.is_null() {
            continue;
        }
        append(naming, &request.name_under(scope, label, &face)?);
    }
    Ok(())
}

/// The solid of a shell named as the UNION of its faces (a new child naming of `F`), solved.
fn name_solid_as_union(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
    solid: &Shape,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let naming = TNamingNaming::insert(request.father)?;
    let naming_ref = live(&naming, "BuildNameShell")?;
    let solid_type = solid.shape_type()?;
    edit(naming_ref, |name| {
        name.set_shape_type(solid_type);
        name.set_shape(solid);
        name.set_type(TNamingNameType::Union);
    });
    set_context_label(request, naming_ref)?;
    let label = naming_ref.label();
    for face in explore(solid, ShapeEnum::Face)? {
        append(naming_ref, &request.name_under(scope, &label, &face)?);
    }
    naming_ref.get_name().solve(&label, scope.get_valid());
    ContainerRequest::result(naming_ref)
}

/// The file-static `IsAllIn(S, Context)`: every child of the collection (recursively for
/// compounds) is a sub-shape of the context with its orientation.
pub(crate) fn is_all_in(shape: &Shape, context: &Shape) -> Result<bool, Failure> {
    if shape.is_null() || context.is_null() {
        return Ok(false);
    }
    let (mut count, mut found) = (0, 0);
    for child in ShapeIterator::new(shape, true, true) {
        count += 1;
        let kind = child.shape_type()?;
        let inside = if kind == ShapeEnum::Compound {
            is_all_in(&child, context)?
        } else {
            explore(context, kind)?
                .iter()
                .any(|sub_shape| sub_shape.is_equal(&child))
        };
        if inside {
            found += 1;
        }
    }
    Ok(count == found)
}

/// The file-static `BuildAggregationName(F, MDF, S, Context, Stop, Geom)`: the name of each
/// element of a compound; a container element gets a UNION naming of its own (a wire or a
/// shell through their builders), kept when it solves to the element.
pub(crate) fn build_aggregation_name(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
) -> Result<(), Failure> {
    let all_in = is_all_in(request.selection, request.context)?;
    let naming = request.naming()?;
    let naming_ref = live(&naming, "BuildAggregationName")?;
    let naming_label = naming_ref.label();
    for element in ShapeIterator::new(request.selection, true, true) {
        let kind = element.shape_type()?;
        let named_solid = kind == ShapeEnum::Solid
            && TNamingTool::named_shape(&element, &naming_label)?.is_some();
        if named_solid || matches!(kind, ShapeEnum::Face | ShapeEnum::Edge | ShapeEnum::Vertex) {
            append(
                naming_ref,
                &request.name_under(scope, request.father, &element)?,
            );
            continue;
        }
        let element_named = aggregate_element(request, scope, naming_ref, &element, all_in)?;
        if let Some(element_named) = element_named {
            append(naming_ref, &element_named);
        }
    }
    Ok(())
}

/// One container element of `BuildAggregationName`: a UNION naming under `F` built from its
/// faces or edges (or recursively), solved; its named shape when it gives the element.
fn aggregate_element(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
    parent: &TNamingNaming,
    element: &Shape,
    all_in: bool,
) -> Result<Option<Handle<TNamingNamedShape>>, Failure> {
    let kind = element.shape_type()?;
    let atom = match kind {
        ShapeEnum::Solid | ShapeEnum::Shell => Some(ShapeEnum::Face),
        ShapeEnum::Wire => Some(ShapeEnum::Edge),
        _ => None,
    };
    let naming = TNamingNaming::insert(request.father)?;
    let naming_ref = live(&naming, "BuildAggregationName")?;
    let label = naming_ref.label();
    edit(naming_ref, |name| {
        name.set_shape_type(kind);
        name.set_shape(element);
        name.set_type(TNamingNameType::Union);
    });
    // Natively the orientation goes to the parent's name, not the new one.
    edit(parent, |name| name.set_orientation(element.orientation()));
    let sub = ContainerRequest {
        father: &label,
        selection: element,
        ..*request
    };
    match (atom, kind) {
        (Some(_), ShapeEnum::Wire) => {
            build_name_wire(&sub, scope)?;
        }
        (Some(_), ShapeEnum::Shell) => {
            build_name_shell(&sub, scope)?;
        }
        (Some(atom), _) => {
            for part in explore(element, atom)? {
                append(naming_ref, &request.name_under(scope, &label, &part)?);
            }
        }
        (None, _) => build_aggregation_name(&sub, scope)?,
    }
    if all_in {
        let context_named = TNamingTool::named_shape(request.context, request.father)?;
        if let Some(context_named) = context_named.get() {
            let context_label = context_named.label();
            edit(naming_ref, |name| name.set_context_label(&context_label));
        }
    }
    naming_ref.get_name().solve(&label, scope.get_valid());
    let named = ContainerRequest::result(naming_ref)?;
    if named.is_null() || request.geometry || !test_solution(scope, &named, element)? {
        return Ok(None);
    }
    Ok(Some(named))
}
