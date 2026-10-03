//! Native removal of eligible wires containing the same edge twice.
use super::{FixRoot, ShapeFixFace};
use crate::{shape_extend::ShapeExtendWireData, shape_fix::ShapeFixIntersectionTool};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topods::{self, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
impl ShapeFixFace {
    /// Remove doubled-edge wires when at least two eligible wires exist; `tool.fix_wires_two_coinc_edges()?`.
    pub fn fix_wires_two_coinc_edges(&self) -> Result<bool, Failure> {
        let original = self.apply_coincident_wire_context()?;
        let children = ShapeIterator::new(&original, false, true).collect::<Vec<_>>();
        if children.iter().filter(|shape| eligible_wire(shape)).count() < 2 {
            return Ok(false);
        }
        let mut replacement = topods::face(&original.empty_copied()?)?;
        replacement.set_orientation(Orientation::Forward);
        let changed = copy_retained_children(&children, &mut replacement)?;
        if !changed {
            return Ok(false);
        }
        replacement.set_orientation(original.orientation());
        if !self.context().is_null() {
            self.context().replace(&original, &replacement)?;
        }
        *self.face_state.write() = replacement.into_shape();
        Ok(true)
    }
    fn apply_coincident_wire_context(&self) -> Result<Face, Failure> {
        if !self.context().is_null() {
            let applied = self
                .context()
                .apply(&self.face()?.into_shape(), ShapeEnum::Shape)?;
            *self.face_state.write() = topods::face(&applied)?.into_shape();
        }
        self.face()
    }
    /// Delegate cross-wire repair through the native shared context; `tool.fix_intersecting_wires()?`.
    pub fn fix_intersecting_wires(&self) -> Result<bool, Failure> {
        let tool = ShapeFixIntersectionTool::with_max_tolerance(
            &self.context(),
            self.precision(),
            self.max_tolerance(),
        );
        let mut face = self.face()?;
        let result = tool.fix_intersecting_wires(&mut face);
        // The native method passes myFace by reference, including failure-side mutations.
        *self.face_state.write() = face.into_shape();
        result
    }
}
fn eligible_wire(shape: &Shape) -> bool {
    shape.shape_type().ok() == Some(ShapeEnum::Wire)
        && matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        )
}
fn copy_retained_children(children: &[Shape], face: &mut Face) -> Result<bool, Failure> {
    let mut changed = false;
    for child in children {
        if eligible_wire(child) && doubled_edge_wire(child)? {
            changed = true;
            continue;
        }
        BRepBuilder::default().add(face, child)?;
    }
    Ok(changed)
}
fn doubled_edge_wire(shape: &Shape) -> Result<bool, Failure> {
    let wire = ShapeExtendWireData::from_wire(&topods::wire(shape)?, true, true)?;
    if wire.nb_edges() != 2 {
        return Ok(false);
    }
    let mut first = wire.edge(1)?;
    let mut second = wire.edge(2)?;
    first.set_orientation(Orientation::Forward);
    second.set_orientation(Orientation::Forward);
    Ok(first == second)
}
