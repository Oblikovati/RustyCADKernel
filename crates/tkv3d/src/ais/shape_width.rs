//! Native AIS_Shape line-width ownership; float publication precedes validating six aspects.
use super::{shape::AisShapeSource, shape_appearance::*, shape_unset_appearance::*};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;

/// Apply inherited width to all six native line categories; `ais_shape_set_width(&shape,2.0)?`.
pub fn ais_shape_set_width<S: AisShapeSource + ?Sized>(
    object: &S,
    width: f64,
) -> Result<(), Failure> {
    object.presentable_base().fields.write().own_width = width as f32;
    let drawer = object.attributes();
    let recompute = set_shape_drawer_width(&drawer, width)?;
    refresh_shape_aspect_ownership(object, recompute);
    object.recompute_computed();
    Ok(())
}

/// Restore linked widths for colored shapes, otherwise release owned lines; `ais_shape_unset_width(&shape)?`.
pub fn ais_shape_unset_width<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    if !object.has_width() {
        return Ok(());
    }
    object.presentable_base().fields.write().own_width = 0.0;
    if object.has_color() {
        restore_shape_linked_widths(object)?;
        object.synchronize_aspects();
    } else {
        release_shape_width_aspects(object)?;
    }
    object.recompute_computed();
    Ok(())
}

fn release_shape_width_aspects<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let drawer = object.attributes();
    let linked = required_shape_drawer_link(&drawer)?;
    let mut replacements = Graphic3dAspectsReplaceMap::default();
    for (old, new) in shape_colored_line_aspects(&drawer)
        .into_iter()
        .zip(shape_colored_line_aspects(&linked))
    {
        bind_shape_default_aspect(&mut replacements, old.aspect().cast(), new.aspect().cast());
    }
    clear_shape_line_slots(&drawer);
    object.replace_aspects(&replacements);
    Ok(())
}

fn restore_shape_linked_widths<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let drawer = object.attributes();
    let linked = drawer.link();
    let defaults = linked.get().map(|_| shape_colored_line_aspects(&linked));
    for (index, line) in shape_colored_line_aspects(&drawer).into_iter().enumerate() {
        let width = defaults
            .as_ref()
            .map_or(1.0, |lines| f64::from(lines[index].aspect().width()));
        line.set_width(width)?;
    }
    Ok(())
}

/// `AIS_Shape::setWidth` (protected): the width of the drawer line aspects, true when new own
/// line aspects were created; `set_shape_drawer_width(&drawer, 2.0)?`.
pub fn set_shape_drawer_width(
    drawer: &Handle<crate::prs3d::Prs3dDrawer>,
    width: f64,
) -> Result<bool, Failure> {
    let recompute = drawer.set_own_line_aspects(&Handle::default())?;
    for line in shape_colored_line_aspects(drawer) {
        line.set_width(width)?;
    }
    Ok(recompute)
}
