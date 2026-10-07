// Rust port of TNaming_Naming::Name and its file-static helpers (Validate, UnValidate,
// BuildScope, IsOneIn, RepeatabilityInContext) of
// ApplicationFramework/TKCAF/TNaming/TNaming_Naming.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Naming::Name`: builds the name of a selection in its context.
use super::naming_build::build_name;
use super::naming_checks::{build_ns, explore, next_modif, test_solution};
use super::naming_containers::{
    ContainerRequest, build_aggregation_name, build_name_shell, build_name_wire,
};
use super::{
    TNaming, TNamingNameType, TNamingNamedShape, TNamingNaming, TNamingNewShapeIterator,
    TNamingOldShapeIterator, TNamingScope, TNamingSelector, TNamingTool,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// The warning the native code prints on the standard output when a name fails.
const FAILED: &str = " %%% WARNING: TNaming_Naming::Name:  FAILED";

impl TNamingNaming {
    /// `Name(where, Selection, Context, Geometry, KeepOrientation, BNproblem)`: names the
    /// selection under `where` (a new child naming) in the context, and returns the named
    /// shape that re-finds it; an UNKNOWN name when no naming gives it back. With
    /// `KeepOrientation`, a selection repeated in the context gets an ORIENTATION name.
    pub fn name(
        father: &TdfLabel,
        selection: &Shape,
        context: &Shape,
        geometry: bool,
        keep_orientation: bool,
        bn_problem: bool,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if keep_orientation {
            let repeats = repeatability_in_context(selection, context)?;
            let problem = bn_problem && repeats != 0 && selection != context;
            if repeats > 1 || problem {
                let orientation = OrientationName {
                    father,
                    selection,
                    context,
                    geometry,
                    problem,
                };
                if let Some(result) = orientation.build()? {
                    return Ok(result);
                }
            } else if TNamingSelector::is_identified(father, selection, &mut named, geometry)? {
                return Ok(named);
            }
        }
        // The scope in which the context is the last valid state (a posteriori naming).
        let mut scope = TNamingScope::new();
        build_scope(&mut scope, context, father)?;
        let stop: Handle<TNamingNamedShape> = Handle::default();
        let kind = selection.shape_type()?;
        let simple = matches!(kind, ShapeEnum::Face | ShapeEnum::Edge | ShapeEnum::Vertex)
            || (kind == ShapeEnum::Solid && TNamingTool::named_shape(selection, father)?.is_some());
        let request = ContainerRequest {
            father,
            selection,
            context,
            stop: &stop,
            geometry,
        };
        if simple {
            // The selection as a simple element.
            let named = build_name(father, &mut scope, selection, context, &stop, geometry)?;
            if geometry || test_solution(&scope, &named, selection)? {
                return Ok(named);
            }
        } else if let Some(named) = name_collection(&request, &mut scope)? {
            return Ok(named);
        }
        println!("{FAILED}");
        // The naming is not satisfactory.
        build_ns(father, selection, TNamingNameType::Unknown)
    }
}

/// The ORIENTATION branch of `Name`.
struct OrientationName<'a> {
    father: &'a TdfLabel,
    selection: &'a Shape,
    context: &'a Shape,
    geometry: bool,
    problem: bool,
}

impl OrientationName<'_> {
    /// The unique context of the selection (or of a compound selection's items) named with
    /// the selection itself; `None` without a unique context (the general naming goes on).
    fn build(&self) -> Result<Option<Handle<TNamingNamedShape>>, Failure> {
        let mut unique = TNaming::find_unique_context(self.selection, self.context)?;
        let mut items: Handle<HArray1<Shape>> = Handle::default();
        if unique.is_null() && self.selection.shape_type()? == ShapeEnum::Compound {
            unique = TNaming::find_unique_context_set(self.selection, self.context, &mut items)?;
        }
        if unique.is_null() {
            return Ok(None);
        }
        let naming = TNamingNaming::insert(self.father)?;
        let naming_ref = live(&naming, "TNaming_Naming::Name")?;
        let label = naming_ref.label();
        {
            let mut name = naming_ref.change_name();
            name.set_shape_type(self.selection.shape_type()?);
            name.set_shape(self.selection);
            name.set_type(TNamingNameType::Orientation);
            name.set_orientation(self.selection.orientation());
        }
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if !TNamingSelector::is_identified(self.father, self.selection, &mut named, self.geometry)?
        {
            named = TNamingNaming::name(
                &label,
                self.selection,
                self.context,
                self.geometry,
                false,
                false,
            )?;
        }
        naming_ref.change_name().append(&named);
        self.seam_index(naming_ref, &unique)?;
        self.name_contexts(naming_ref, &unique, &items)?;
        self.solve(naming_ref)
    }
    /// szy 21.10.2009: a seam edge of a face is told apart by its position in its wire.
    fn seam_index(&self, naming: &TNamingNaming, unique: &Shape) -> Result<(), Failure> {
        if self.selection.shape_type()? != ShapeEnum::Edge
            || unique.shape_type()? != ShapeEnum::Face
        {
            return Ok(());
        }
        if repeatability_in_context(self.selection, unique)? != 2 {
            return Ok(());
        }
        for wire in ShapeIterator::new(unique, true, true) {
            let position = ShapeIterator::new(&wire, true, true)
                .position(|edge| edge.is_equal(self.selection));
            if let Some(position) = position {
                // The index saves the position of a seam shape (it was for GENERATED only).
                naming
                    .change_name()
                    .set_index(i32::try_from(position + 1).unwrap_or(i32::MAX));
                break;
            }
        }
        Ok(())
    }
    /// The names of the contexts: each item's container for a compound of several, else the
    /// unique context.
    fn name_contexts(
        &self,
        naming: &TNamingNaming,
        unique: &Shape,
        items: &Handle<HArray1<Shape>>,
    ) -> Result<(), Failure> {
        let label = naming.label();
        let compound = self.selection.shape_type()? == ShapeEnum::Compound;
        if compound && items.is_null() {
            // Natively `Arr->Length()` dereferences the null array of a compound whose unique
            // context `FindUniqueContext` found directly.
            return Err(rustycad_tkernel::standard::exceptions::null_object(
                "TNaming_Naming::Name: no context array for the compound selection; expected FindUniqueContextSet",
            ));
        }
        let containers: Vec<Shape> = items
            .get()
            .map(|array| array.array1().iter().cloned().collect())
            .unwrap_or_default();
        if compound && containers.len() > 1 {
            // N arguments: to be optimized to avoid duplication of the same context shape.
            for container in &containers {
                let named = TNamingNaming::name(
                    &label,
                    container,
                    self.context,
                    self.geometry,
                    true,
                    self.problem,
                )?;
                naming.change_name().append(&named);
            }
            return Ok(());
        }
        let named = TNamingNaming::name(
            &label,
            unique,
            self.context,
            self.geometry,
            true,
            self.problem,
        )?;
        naming.change_name().append(&named);
        Ok(())
    }
    /// The orientation name solved in the scope of the context, checked against the
    /// selection.
    fn solve(&self, naming: &TNamingNaming) -> Result<Option<Handle<TNamingNamedShape>>, Failure> {
        let mut scope = TNamingScope::new();
        build_scope(&mut scope, self.context, self.father)?;
        let label = naming.label();
        naming.get_name().solve(&label, scope.get_valid());
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        // Natively the context label is read from the result before its null check.
        let result_label = live(&named, "TNaming_Naming::Name")?.label();
        naming.change_name().set_context_label(&result_label);
        if self.geometry {
            return Ok(Some(named));
        }
        if test_solution(&scope, &named, self.selection)? {
            return Ok(Some(named));
        }
        println!("{FAILED}");
        // The naming is unsatisfactory.
        Ok(Some(build_ns(
            self.father,
            self.selection,
            TNamingNameType::Unknown,
        )?))
    }
}

/// The collection branch of `Name`: a UNION naming (WIREIN or SHELLIN for a wire or shell, an
/// aggregation for a compound) of the simple elements, solved; `None` when it does not give
/// the selection back.
fn name_collection(
    request: &ContainerRequest<'_>,
    scope: &mut TNamingScope,
) -> Result<Option<Handle<TNamingNamedShape>>, Failure> {
    let (father, selection, context) = (request.father, request.selection, request.context);
    let kind = selection.shape_type()?;
    let naming = TNamingNaming::insert(father)?;
    let naming_ref = live(&naming, "TNaming_Naming::Name")?;
    let label = naming_ref.label();
    {
        let mut name = naming_ref.change_name();
        // Modified by vro 05.09.00.
        name.set_shape_type(kind);
        name.set_shape(selection);
        name.set_orientation(selection.orientation());
        if kind != ShapeEnum::Wire {
            name.set_type(TNamingNameType::Union);
        }
    }
    let in_context = !context.is_null()
        && (context.shape_type()? as i32) < kind as i32
        && is_one_in(selection, context)?;
    if in_context {
        let context_named = TNamingTool::named_shape(context, father)?;
        if let Some(context_named) = context_named.get() {
            naming_ref
                .change_name()
                .set_context_label(&context_named.label());
        }
    }
    let sub = ContainerRequest {
        father: &label,
        ..*request
    };
    match kind {
        ShapeEnum::Wire => {
            build_name_wire(&sub, scope)?;
        }
        ShapeEnum::Shell => {
            build_name_shell(&sub, scope)?;
        }
        ShapeEnum::CompSolid | ShapeEnum::Solid => {
            naming_ref.change_name().set_type(TNamingNameType::Union);
            for face in explore(selection, ShapeEnum::Face)? {
                let named = build_name(
                    &label,
                    scope,
                    &face,
                    context,
                    request.stop,
                    request.geometry,
                )?;
                naming_ref.change_name().append(&named);
            }
        }
        ShapeEnum::Compound => build_aggregation_name(&sub, scope)?,
        _ => {
            for child in ShapeIterator::new(selection, true, true) {
                let named = build_name(
                    &label,
                    scope,
                    &child,
                    context,
                    request.stop,
                    request.geometry,
                )?;
                naming_ref.change_name().append(&named);
            }
        }
    }
    naming_ref.get_name().solve(&label, scope.get_valid());
    let mut named: Handle<TNamingNamedShape> = Handle::default();
    label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
    if request.geometry {
        return Ok(Some(named));
    }
    if named.is_null() {
        return Ok(Some(build_ns(father, selection, TNamingNameType::Unknown)?));
    }
    Ok(test_solution(scope, &named, selection)?.then_some(named))
}

/// The file-static `Validate(MDF, it)`: the label of an ancestor evolution, its children, and
/// recursively the older ones.
fn validate(scope: &mut TNamingScope, iterator: &TNamingOldShapeIterator) -> Result<(), Failure> {
    let label = iterator.label()?;
    scope.valid(&label);
    scope.valid_children(&label, true)?;
    let mut older = TNamingOldShapeIterator::from_old_shape_iterator(iterator)?;
    while older.more() {
        validate(scope, &older)?;
        older.next()?;
    }
    Ok(())
}

/// The file-static `UnValidate(MDF, it)`: the label of a later evolution, its children, and
/// recursively the newer ones.
fn unvalidate(scope: &mut TNamingScope, iterator: &TNamingNewShapeIterator) -> Result<(), Failure> {
    let label = iterator.label()?;
    scope.unvalid(&label);
    scope.unvalid_children(&label, true)?;
    let mut newer = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
    while newer.more() {
        unvalidate(scope, &newer)?;
        newer.next()?;
    }
    Ok(())
}

/// The file-static `BuildScope(MDF, Context, Acces)`: a context that has been modified since
/// limits the scope to its history (a posteriori naming); else the whole framework.
fn build_scope(
    scope: &mut TNamingScope,
    context: &Shape,
    access: &TdfLabel,
) -> Result<(), Failure> {
    if context.is_null() {
        scope.set_with_valid(false);
        return Ok(());
    }
    // Is the context the current state?
    let named = TNamingTool::named_shape(context, access)?;
    // A next modification of the context's named shape.
    if next_modif(&named)?.is_null() {
        scope.set_with_valid(false);
        return Ok(());
    }
    scope.set_with_valid(true);
    let label = live(&named, "BuildScope")?.label();
    scope.valid(&label);
    scope.valid_children(&label, true)?;
    let mut olds = TNamingOldShapeIterator::from_label(context, access)?;
    while olds.more() {
        validate(scope, &olds)?;
        olds.next()?;
    }
    let mut news = TNamingNewShapeIterator::from_label(context, access)?;
    while news.more() {
        unvalidate(scope, &news)?;
        news.next()?;
    }
    Ok(())
}

/// The file-static `IsOneIn(S, Context)`: the selection is a sub-shape of the context, with
/// its orientation.
fn is_one_in(shape: &Shape, context: &Shape) -> Result<bool, Failure> {
    if shape.is_null() || context.is_null() {
        return Ok(false);
    }
    Ok(explore(context, shape.shape_type()?)?
        .iter()
        .any(|sub_shape| sub_shape.is_equal(shape)))
}

/// The file-static `RepeatabilityInContext(Selection, Context)`: how many times the selection
/// (not a shell) is a sub-shape of the context; for a compound, the most repeated item.
fn repeatability_in_context(selection: &Shape, context: &Shape) -> Result<i32, Failure> {
    if context.is_null() || selection.is_null() {
        return Ok(0);
    }
    let kind = selection.shape_type()?;
    if (context.shape_type()? as i32) < kind as i32 {
        if kind == ShapeEnum::Shell {
            return Ok(0);
        }
        return count_same(context, selection);
    }
    if kind != ShapeEnum::Compound {
        return Ok(0);
    }
    let mut most = 0;
    for item in ShapeIterator::new(selection, true, true) {
        most = most.max(count_same(context, &item)?);
    }
    Ok(most)
}

/// The number of sub-shapes of the context that are the same as the shape.
fn count_same(context: &Shape, shape: &Shape) -> Result<i32, Failure> {
    let count = explore(context, shape.shape_type()?)?
        .iter()
        .filter(|sub| sub.is_same(shape))
        .count();
    Ok(i32::try_from(count).unwrap_or(i32::MAX))
}
