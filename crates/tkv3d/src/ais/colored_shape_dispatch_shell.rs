//! Native closed-shell classification accounts for hidden faces and actual front/back alpha policy.
use super::{
    AisColoredDrawer,
    colored_shape_dispatch::{
        ColoredDispatchContext, ColoredDispatchParent, dispatch_colored_topology,
    },
};
use crate::std_prs::StdPrsToolTriangulatedShape;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, ShapeIterator},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::graphic3d::*;
pub(super) fn dispatch_colored_aggregate(
    drawer: &Handle<AisColoredDrawer>,
    shape: &Shape,
    parent: ColoredDispatchParent,
    own: bool,
    context: &ColoredDispatchContext<'_>,
) -> Result<bool, Failure> {
    let closed = colored_shell_closed(shape, parent.kind, context)?;
    let inherited = ColoredDispatchParent {
        kind: shape.shape_type()?,
        closed,
    };
    let mut iterator = ShapeIterator::new(shape, true, true);
    let mut overridden = own;
    while iterator.more() {
        overridden |= dispatch_colored_topology(drawer, iterator.value()?, inherited, context)?;
        iterator.next();
    }
    Ok(overridden)
}
fn colored_shell_closed(
    shape: &Shape,
    parent: ShapeEnum,
    context: &ColoredDispatchContext<'_>,
) -> Result<bool, Failure> {
    if parent != ShapeEnum::Solid || shape.shape_type()? != ShapeEnum::Shell {
        return Ok(false);
    }
    if !BRepTool::is_closed(shape)? || !StdPrsToolTriangulatedShape::is_triangulated(shape)? {
        return Ok(false);
    }
    let mut faces = ShapeIterator::new(shape, true, true);
    while faces.more() {
        if colored_face_opens_shell(faces.value()?, context)? {
            return Ok(false);
        }
        faces.next();
    }
    Ok(true)
}
fn colored_face_opens_shell(
    face: &Shape,
    context: &ColoredDispatchContext<'_>,
) -> Result<bool, Failure> {
    if face.shape_type()? != ShapeEnum::Face {
        return Ok(false);
    }
    let nodes = context.styles.nodes.read();
    let Some(node) = super::shape_drawer_map::find_shape_drawer_node(&nodes, face) else {
        return Ok(false);
    };
    node.read_fields(|fields| colored_face_alpha_opens_shell(fields.drawer))
}
fn colored_face_alpha_opens_shell(drawer: &AisColoredDrawer) -> bool {
    if drawer.is_hidden() {
        return true;
    }
    if !drawer.has_own_shading_aspect() {
        return false;
    }
    let aspect = drawer.shading_aspect().aspect();
    let alpha = aspect.alpha_mode();
    if alpha == Graphic3dAlphaMode::Opaque {
        return false;
    }
    alpha != Graphic3dAlphaMode::BlendAuto
        || aspect.front_material().alpha() < 1.0
        || (aspect.distinguish() && aspect.back_material().alpha() < 1.0)
}
