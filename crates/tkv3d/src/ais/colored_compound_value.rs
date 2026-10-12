//! Native TopoDS value fields retain their old TShape pointer through assignment/destructor callbacks.
use rustycad_tkbrep::topods::{Compound, Shape, TShape};
use rustycad_tkernel::standard::{Failure, HandleReference, IntrusiveReferenceSlot, SharedState};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;
struct ColoredCompoundPlacement {
    location: Location,
    orientation: Orientation,
}
pub(super) struct ColoredCompoundValue {
    topology: IntrusiveReferenceSlot<dyn TShape>,
    placement: SharedState<Option<ColoredCompoundPlacement>>,
}
impl ColoredCompoundValue {
    pub(super) fn new(compound: &Compound) -> Self {
        let topology = IntrusiveReferenceSlot::default();
        topology
            .assign(compound.tshape())
            .expect("compound topology assignment failed; expected an idle new slot");
        let placement = ColoredCompoundPlacement {
            location: compound.location().clone(),
            orientation: compound.orientation(),
        };
        Self {
            topology,
            placement: SharedState::new(Some(placement)),
        }
    }
    pub(super) fn assign(&self, compound: &Compound) -> Result<(), Failure> {
        // Native implicit Shape assignment releases/retains TShape before copying Location and Orientation.
        self.topology.assign(compound.tshape())?;
        let mut placement = self.placement.write();
        let placement = placement
            .as_mut()
            .ok_or_else(super::colored_compound_node::colored_compound_removed)?;
        placement.location = compound.location().clone();
        placement.orientation = compound.orientation();
        Ok(())
    }
    pub(super) fn clone_compound(&self) -> Result<Compound, Failure> {
        let topology = self.topology.clone_handle();
        let placement = self.placement.read();
        let placement = placement
            .as_ref()
            .ok_or_else(super::colored_compound_node::colored_compound_removed)?;
        Ok(Compound::unchecked(Shape::from_parts(
            topology,
            placement.location.clone(),
            placement.orientation,
        )))
    }
    pub(super) fn with_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let (location, orientation) = self.reference_placement()?;
        Shape::with_reference_parts(&self.topology, &location, orientation, project)
    }
    fn reference_placement(&self) -> Result<(Location, Orientation), Failure> {
        let placement = self.placement.read();
        let placement = placement
            .as_ref()
            .ok_or_else(super::colored_compound_node::colored_compound_removed)?;
        Ok((placement.location.clone(), placement.orientation))
    }
    #[cfg(test)]
    pub(super) fn object_address(&self) -> *const () {
        let topology = &self.topology;
        topology.object_address()
    }
    pub(super) fn ref_count(&self) -> i32 {
        // Native const Value references may read TShape count0 inside the retiring target destructor.
        self.topology.ref_count()
    }
    pub(super) fn release(&self) {
        // Key has already been destroyed, while Size and this authoritative pointer remain observable.
        self.topology
            .clear()
            .expect("compound topology retirement failed; expected a single owning-map release");
        let placement = self.placement.write().take();
        drop(placement);
    }
}
