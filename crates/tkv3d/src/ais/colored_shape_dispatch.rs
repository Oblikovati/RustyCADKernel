//! Native depth-first style dispatch preserves full faces and strips overridden lower-dimensional children.
use super::{AisColoredDrawer, AisColoredDrawerCompounds, AisShapeDrawerMap};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, ShapeIterator},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Native eight opened topology categories; `let opened:AisColoredOpenedCompounds=std::array::from_fn(|_|Default::default())`.
pub type AisColoredOpenedCompounds = [AisColoredDrawerCompounds; 8];
pub(super) struct ColoredDispatchContext<'a> {
    pub styles: &'a AisShapeDrawerMap,
    pub opened: &'a AisColoredOpenedCompounds,
    pub closed: &'a AisColoredDrawerCompounds,
}
#[derive(Clone, Copy)]
pub(super) struct ColoredDispatchParent {
    pub kind: ShapeEnum,
    pub closed: bool,
}
impl ColoredDispatchParent {
    fn child(kind: ShapeEnum, closed: bool) -> Self {
        Self { kind, closed }
    }
}
/// Append native per-style topology groups, supporting closed-output aliases; `ais_colored_shape_dispatch_colors(&drawer,&shape,&styles,ShapeEnum::Compound,false,&opened,&opened[4])?`.
#[allow(clippy::too_many_arguments)] // Exact pinned protected overload exposes the opened array and possibly aliased closed map.
pub fn ais_colored_shape_dispatch_colors(
    parent: &Handle<AisColoredDrawer>,
    shape: &Shape,
    styles: &AisShapeDrawerMap,
    parent_kind: ShapeEnum,
    parent_closed: bool,
    opened: &AisColoredOpenedCompounds,
    closed: &AisColoredDrawerCompounds,
) -> Result<bool, Failure> {
    let context = ColoredDispatchContext {
        styles,
        opened,
        closed,
    };
    let inherited = ColoredDispatchParent {
        kind: parent_kind,
        closed: parent_closed,
    };
    dispatch_colored_topology(parent, shape, inherited, &context)
}
pub(super) fn dispatch_colored_topology(
    parent: &Handle<AisColoredDrawer>,
    shape: &Shape,
    inherited: ColoredDispatchParent,
    context: &ColoredDispatchContext<'_>,
) -> Result<bool, Failure> {
    let kind = shape.shape_type()?;
    if kind == ShapeEnum::Shape {
        return Ok(false);
    }
    let (drawer, overridden) = dispatch_colored_drawer(parent, shape, context.styles)?;
    if overridden && drawer.is_hidden() {
        return Ok(true);
    }
    if kind <= ShapeEnum::Shell {
        use super::colored_shape_dispatch_shell::dispatch_colored_aggregate;
        return dispatch_colored_aggregate(&drawer, shape, inherited, overridden, context);
    }
    dispatch_colored_leaf(&drawer, shape, kind, inherited, overridden, context)
}
pub(super) fn dispatch_colored_drawer(
    parent: &Handle<AisColoredDrawer>,
    shape: &Shape,
    styles: &AisShapeDrawerMap,
) -> Result<(Handle<AisColoredDrawer>, bool), Failure> {
    let mut drawer = parent.clone();
    let nodes = styles.nodes.read();
    let Some(node) = super::shape_drawer_map::find_shape_drawer_node(&nodes, shape) else {
        return Ok((drawer, false));
    };
    let replacement = node.read_fields(|fields| fields.drawer.clone())?;
    drawer = replacement;
    Ok((drawer, true))
}
fn dispatch_colored_leaf(
    drawer: &Handle<AisColoredDrawer>,
    shape: &Shape,
    kind: ShapeEnum,
    inherited: ColoredDispatchParent,
    overridden: bool,
    context: &ColoredDispatchContext<'_>,
) -> Result<bool, Failure> {
    let child = ColoredDispatchParent::child(kind, inherited.closed);
    let (children, sub_override) = dispatch_colored_children(drawer, shape, child, context)?;
    let Some(remainder) = colored_leaf_remainder(shape, kind, sub_override, &children)? else {
        return Ok(overridden || sub_override);
    };
    if should_group_colored_leaf(overridden, sub_override, inherited.kind) {
        bind_colored_compound(drawer, &remainder, kind, inherited.closed, context)?;
    }
    Ok(overridden || sub_override)
}
fn dispatch_colored_children(
    drawer: &Handle<AisColoredDrawer>,
    shape: &Shape,
    inherited: ColoredDispatchParent,
    context: &ColoredDispatchContext<'_>,
) -> Result<(Vec<Shape>, bool), Failure> {
    let mut children = Vec::new();
    let mut any_override = false;
    let mut iterator = ShapeIterator::new(shape, true, true);
    while iterator.more() {
        let child = iterator.value()?;
        let overridden = dispatch_colored_topology(drawer, child, inherited, context)?;
        any_override |= overridden;
        if !overridden && inherited.kind != ShapeEnum::Face {
            children.push(child.clone());
        }
        iterator.next();
    }
    Ok((children, any_override))
}
fn colored_leaf_remainder(
    shape: &Shape,
    kind: ShapeEnum,
    overridden: bool,
    children: &[Shape],
) -> Result<Option<Shape>, Failure> {
    if kind == ShapeEnum::Face || !overridden {
        return Ok(Some(shape.clone()));
    }
    if children.is_empty() {
        return Ok(None);
    }
    let mut copy = shape.empty_copied()?;
    copy.set_closed(shape.closed()?)?;
    for child in children {
        BRepBuilder::default().add(&mut copy, child)?;
    }
    Ok(Some(copy))
}
fn should_group_colored_leaf(own: bool, descendant: bool, parent: ShapeEnum) -> bool {
    // Native suppresses redundant edge groups under wires/faces even for a vertex override.
    own || (descendant && parent != ShapeEnum::Wire && parent != ShapeEnum::Face)
        || (parent <= ShapeEnum::Shell && !(own || descendant))
}
fn bind_colored_compound(
    drawer: &Handle<AisColoredDrawer>,
    shape: &Shape,
    kind: ShapeEnum,
    closed: bool,
    context: &ColoredDispatchContext<'_>,
) -> Result<(), Failure> {
    let output = if closed && kind == ShapeEnum::Face {
        context.closed
    } else {
        &context.opened[kind as usize]
    };
    let mut compound = Compound::default();
    if !output.find_into(drawer, &mut compound)? {
        BRepBuilder::default().make_compound(&mut compound);
        output.add(drawer, &compound)?;
    }
    BRepBuilder::default().add(&mut compound, shape)
}
