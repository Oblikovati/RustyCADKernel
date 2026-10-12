//! Native two-pass external-compound routing; live source nodes permit defined same-map output.
use super::{AisColoredDrawer, AisColoredShapeSource, AisShapeDrawerMap};
use super::{
    colored_shape_expansion::bind_colored_shape_leaves,
    shape_drawer_cursor::ShapeDrawerNativeCursor,
};
use rustycad_tkbrep::{
    topods::{Shape, ShapeIterator},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{
    ncollection::Map,
    standard::{Failure, Handle, HandleFieldRef, HandleReference},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
type OwnedCompounds = Map<Shape, ShapeMapHasher>;
/// Expand custom aggregate styles into the actual output map; `ais_colored_shape_fill_subshape_drawer_map(object,&output)?`.
pub fn ais_colored_shape_fill_subshape_drawer_map<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    output: &AisShapeDrawerMap,
) -> Result<(), Failure> {
    let owned = own_colored_compounds(object)?;
    let source = &object.colored_shape_base().shape_colors;
    visit_colored_map_entries(source, |key, drawer| {
        expand_external_compound(source, output, &owned, key, drawer)
    })?;
    visit_colored_map_entries(source, |key, drawer| {
        expand_explicit_entry(object, output, &owned, key, drawer)
    })
}
fn own_colored_compounds<S: AisColoredShapeSource + ?Sized>(
    object: &S,
) -> Result<OwnedCompounds, Failure> {
    let mut owned = OwnedCompounds::with_hasher(ShapeMapHasher, 1);
    let fields = object.shape_base().fields.read();
    if fields.shape.shape_type()? == ShapeEnum::Compound {
        owned.add(fields.shape.clone());
        collect_colored_compounds(&mut owned, &fields.shape)?;
    }
    Ok(owned)
}
fn collect_colored_compounds(owned: &mut OwnedCompounds, shape: &Shape) -> Result<(), Failure> {
    let mut children = ShapeIterator::new(shape, true, true);
    while children.more() {
        let child = children.value()?;
        if child.shape_type()? == ShapeEnum::Compound && owned.add(child.clone()) {
            collect_colored_compounds(owned, child)?;
        }
        children.next();
    }
    Ok(())
}
fn expand_external_compound(
    source: &AisShapeDrawerMap,
    output: &AisShapeDrawerMap,
    owned: &OwnedCompounds,
    key: &Shape,
    drawer: &impl HandleReference<AisColoredDrawer>,
) -> Result<(), Failure> {
    if key.shape_type()? != ShapeEnum::Compound || owned.contains(key) {
        return Ok(());
    }
    let mut children = ShapeIterator::new(key, true, true);
    while children.more() {
        let child = children.value()?;
        if !source.is_bound(child) {
            bind_colored_shape_leaves(output, child, drawer)?;
        }
        children.next();
    }
    Ok(())
}
fn expand_explicit_entry<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    output: &AisShapeDrawerMap,
    owned: &OwnedCompounds,
    key: &Shape,
    drawer: &impl HandleReference<AisColoredDrawer>,
) -> Result<(), Failure> {
    if object.shape_base().fields.read().shape.is_equal(key) {
        return Ok(());
    }
    if key.shape_type()? == ShapeEnum::Compound && !owned.contains(key) {
        return Ok(());
    }
    // Preserve native leaf rebind priority over aggregate shell/wire styles, not first-entry wins.
    bind_colored_shape_leaves(output, key, drawer)
}
fn visit_colored_map_entries(
    source: &AisShapeDrawerMap,
    mut visit: impl FnMut(
        &Shape,
        &HandleFieldRef<'_, Option<Handle<AisColoredDrawer>>, AisColoredDrawer>,
    ) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut cursor = ShapeDrawerNativeCursor::new(source)?;
    while let Some(node) = cursor.current()? {
        node.with_entry(|key, drawer| visit(key, drawer))??;
        cursor.next()?;
    }
    Ok(())
}
