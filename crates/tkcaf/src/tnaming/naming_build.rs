// Rust port of the file-static BuildName and BuildNameInNS of
// ApplicationFramework/TKCAF/TNaming/TNaming_Naming.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Naming one simple shape: identify it, write its name (with the names of its arguments),
//! solve the name and, for a single selection, refine it with a filter by neighbours.
use super::naming_checks::{
    build_ns, compare, compare_in_generation, compare_in_modification, find_index,
    find_new_shape_in_father,
};
use super::naming_filter::{FilterRequest, filter};
use super::{
    TNamingIdentifier, TNamingIterator, TNamingLocalizer, TNamingNameType, TNamingNamedShape,
    TNamingNaming, TNamingScope, TNamingTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// The file-static `BuildNameInNS(F, MDF, S, Context, Stop, Geometry)`: names a generator in
/// the context of its generation (the stop becomes the named shape of the generation's
/// father when the generator has a context there).
fn build_name_in_ns(
    father: &TdfLabel,
    scope: &mut TNamingScope,
    shape: &Shape,
    context: &Handle<TNamingNamedShape>,
    stop: &Handle<TNamingNamedShape>,
    geometry: bool,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let mut shape_context = Shape::default();
    let mut new_stop = stop.clone();
    TNamingLocalizer::find_shape_context(context, shape, &mut shape_context)?;
    if !shape_context.is_null() {
        let context_father = live(context, "BuildNameInNS")?.label().father();
        context_father.find_attribute_as(&TNamingNamedShape::get_id(), &mut new_stop)?;
    }
    build_name(father, scope, shape, &shape_context, &new_stop, geometry)
}

/// The file-static `BuildName(F, MDF, Selection, Context, Stop, Geom)`.
pub(crate) fn build_name(
    father: &TdfLabel,
    scope: &mut TNamingScope,
    selection: &Shape,
    context: &Shape,
    stop: &Handle<TNamingNamedShape>,
    geometry: bool,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let only_one = !geometry;
    let mut identifier = TNamingIdentifier::new(father, selection, context, only_one)?;
    if !identifier.is_done() {
        return build_ns(father, selection, TNamingNameType::Unknown);
    }
    let mut naming: Handle<TNamingNaming> = Handle::default();
    let mut is_generation = false;
    let mut named;
    if identifier.is_feature() && stop.is_null() {
        // Already named.
        if !only_one {
            return identifier.feature_arg();
        }
        named = identifier.feature_arg()?;
    } else {
        let request = NameRequest {
            father,
            selection,
            context,
            stop,
            geometry,
        };
        let built = request.write_name(scope, &mut identifier)?;
        naming = built.naming;
        is_generation = built.is_generation;
        named = built.named;
        if named.is_null() {
            return Ok(named);
        }
        if scope.with_valid() {
            scope.valid(&live(&named, "BuildName")?.label());
        }
    }
    if only_one {
        if named.is_null() {
            return Ok(named);
        }
        let check = SingleCheck {
            father,
            selection,
            context,
            stop,
            is_generation,
        };
        check.refine(scope, &identifier, &mut naming, &mut named)?;
    }
    if scope.with_valid() {
        scope.valid(&live(&named, "BuildName")?.label());
    }
    Ok(named)
}

/// The inputs of the identification function written by `BuildName`.
struct NameRequest<'a> {
    father: &'a TdfLabel,
    selection: &'a Shape,
    context: &'a Shape,
    stop: &'a Handle<TNamingNamedShape>,
    geometry: bool,
}

/// What the identification function gives: its naming, whether it is a generation, and the
/// named shape it solved to.
struct BuiltName {
    naming: Handle<TNamingNaming>,
    is_generation: bool,
    named: Handle<TNamingNamedShape>,
}

impl NameRequest<'_> {
    /// The identification function: a new child naming of the identifier's type with its
    /// feature, generation, index, stop and the names of its arguments, then regenerated.
    fn write_name(
        &self,
        scope: &mut TNamingScope,
        identifier: &mut TNamingIdentifier,
    ) -> Result<BuiltName, Failure> {
        let naming = TNamingNaming::insert(self.father)?;
        let naming_ref = live(&naming, "BuildName")?;
        let name_type = identifier.name_type();
        {
            let mut name = naming_ref.change_name();
            name.set_shape_type(self.selection.shape_type()?);
            name.set_shape(self.selection);
            name.set_orientation(self.selection.orientation());
            name.set_type(name_type);
            if identifier.is_feature() {
                name.append(&identifier.feature_arg()?);
            }
            if name_type == TNamingNameType::Generation {
                name.append(&identifier.named_shape_of_generation());
            }
        }
        if name_type == TNamingNameType::ConstShape {
            let index = find_index(&identifier.feature_arg()?, self.selection)?;
            naming_ref.change_name().set_index(index);
        }
        naming_ref.change_name().set_stop_named_shape(self.stop);
        self.name_arguments(scope, identifier, naming_ref)?;
        naming_ref.regenerate(scope.get_valid());
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        naming_ref
            .label()
            .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        let is_generation = name_type == TNamingNameType::Generation;
        Ok(BuiltName {
            naming,
            is_generation,
            named,
        })
    }
    /// The identification of the arguments: features as they are, shapes by their own names.
    fn name_arguments(
        &self,
        scope: &mut TNamingScope,
        identifier: &mut TNamingIdentifier,
        naming: &TNamingNaming,
    ) -> Result<(), Failure> {
        let label = naming.label();
        identifier.init_args();
        while identifier.more_args() {
            let argument = if identifier.arg_is_feature() {
                identifier.feature_arg()?
            } else if naming.get_name().name_type() == TNamingNameType::Generation {
                let generation = identifier.named_shape_of_generation();
                build_name_in_ns(
                    &label,
                    scope,
                    &identifier.shape_arg()?,
                    &generation,
                    self.stop,
                    self.geometry,
                )?
            } else {
                build_name(
                    &label,
                    scope,
                    &identifier.shape_arg()?,
                    self.context,
                    self.stop,
                    self.geometry,
                )?
            };
            naming.change_name().append(&argument);
            identifier.next_arg();
        }
        Ok(())
    }
}

/// The inputs of the single-selection check of `BuildName`.
struct SingleCheck<'a> {
    father: &'a TdfLabel,
    selection: &'a Shape,
    context: &'a Shape,
    stop: &'a Handle<TNamingNamedShape>,
    is_generation: bool,
}

impl SingleCheck<'_> {
    /// Check + filter: a generation not reduced to the selection is filtered in the context
    /// of its father; a MODIFUNTIL (or single-argument INTERSECTION) split 1:n on one label is
    /// described in detail (a feature becomes a GENERATION) and filtered; else a name that does
    /// not give the selection alone is filtered.
    fn refine(
        &self,
        scope: &mut TNamingScope,
        identifier: &TNamingIdentifier,
        naming: &mut Handle<TNamingNaming>,
        named: &mut Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        let mut localizer = TNamingLocalizer::new();
        if !TNamingIterator::new(named)?.more() {
            return Ok(());
        }
        let mut standard_filter = !self.is_generation;
        if self.is_generation {
            if !compare_in_generation(named, self.selection)? {
                let mut new_context = Shape::default();
                find_new_shape_in_father(
                    &identifier.named_shape_of_generation(),
                    &mut new_context,
                )?;
                self.filter(scope, &new_context, &mut localizer, named)?;
            }
        } else if let Some(parent) = self.modification_parent(identifier, naming, named)? {
            let new_named = compare_in_modification(&parent, self.selection)?;
            if new_named.is_some() {
                // The name must be described in detail: a 1:n modification on one label.
                standard_filter = false;
                if identifier.is_feature() {
                    self.rename_as_generation(scope, &new_named, naming, named)?;
                }
                self.filter(scope, self.context, &mut localizer, named)?;
            }
        }
        if standard_filter && !compare(named, scope, self.stop, self.selection)? {
            self.filter(scope, self.context, &mut localizer, named)?;
        }
        Ok(())
    }
    /// The named shape `CompareInModification` starts from: `NS` for a MODIFUNTIL, the only
    /// argument of a single-argument INTERSECTION; `None` otherwise.
    fn modification_parent(
        &self,
        identifier: &TNamingIdentifier,
        naming: &Handle<TNamingNaming>,
        named: &Handle<TNamingNamedShape>,
    ) -> Result<Option<Handle<TNamingNamedShape>>, Failure> {
        if identifier.name_type() == TNamingNameType::ModifUntil {
            return Ok(Some(named.clone()));
        }
        if identifier.name_type() != TNamingNameType::Intersection {
            return Ok(None);
        }
        let name = live(naming, "BuildName")?.get_name();
        if name.arguments().len() != 1 {
            return Ok(None);
        }
        Ok(Some(name.arguments()[0].clone()))
    }
    /// For MODIFUNTIL: the name becomes a GENERATION of the selection by the split shape.
    fn rename_as_generation(
        &self,
        scope: &mut TNamingScope,
        split: &Handle<TNamingNamedShape>,
        naming: &mut Handle<TNamingNaming>,
        named: &mut Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        *naming = TNamingNaming::insert(self.father)?;
        let naming_ref = live(naming, "BuildName")?;
        {
            let mut name = naming_ref.change_name();
            name.set_shape_type(self.selection.shape_type()?);
            name.set_shape(self.selection);
            name.set_orientation(self.selection.orientation());
            name.set_type(TNamingNameType::Generation);
            name.append(&TNamingTool::named_shape(self.selection, self.father)?);
            name.append(split);
        }
        naming_ref.regenerate(scope.get_valid());
        naming_ref
            .label()
            .find_attribute_as(&TNamingNamedShape::get_id(), named)?;
        Ok(())
    }
    /// `Filter(F, MDF, Selection, context, Localizer, NS, 0)`.
    fn filter(
        &self,
        scope: &mut TNamingScope,
        context: &Shape,
        localizer: &mut TNamingLocalizer,
        named: &mut Handle<TNamingNamedShape>,
    ) -> Result<bool, Failure> {
        let request = FilterRequest {
            father: self.father,
            shape: self.selection,
            context,
            level: 0,
        };
        filter(&request, scope, localizer, named)
    }
}
