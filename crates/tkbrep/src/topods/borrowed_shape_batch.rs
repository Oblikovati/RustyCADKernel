// SPDX-License-Identifier: GPL-2.0-only
//! ADR0014 scratch containers borrow components before an explicit topology/geometry copy.
use super::{Builder, Compound, Shape};
use rustycad_tkernel::standard::{Failure, exceptions};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Build a fresh compound without freezing its borrowed children; `let batch = borrowed_shape_batch(&[&edge])?;`.
/// Children still share all caller state: this container is neither immutable nor isolated.
/// Its fresh identity cannot occur within an input, so adding children introduces no cycle.
pub fn borrowed_shape_batch(components: &[&Shape]) -> Result<Shape, Failure> {
    for (index, component) in components.iter().enumerate() {
        validate_batch_component(component, index)?;
    }
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let batch = compound.into_shape();
    let root = batch.checked_tshape("Martensite borrowed batch")?;
    for component in components {
        root.tshape_base().append_child((*component).clone());
    }
    Ok(batch)
}
fn validate_batch_component(component: &Shape, index: usize) -> Result<(), Failure> {
    if component.is_null() {
        return Err(exceptions::null_object(&format!(
            "batch component={index} is null; expected a non-null concrete topological shape"
        )));
    }
    if component.shape_type()? == ShapeEnum::Shape {
        return Err(exceptions::type_mismatch(&format!(
            "batch component={index} kind=Shape; expected Compound/CompSolid/Solid/Shell/Face/Wire/Edge/Vertex"
        )));
    }
    Ok(())
}
