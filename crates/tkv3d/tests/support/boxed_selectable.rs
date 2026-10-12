//! Named selectable with observable bounds reads for subset update tests and probes.
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{prs_mgr::*, select_mgr::*};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BoxedSelectable {
    transient: StandardTransient,
    presentable: PrsMgrPresentableObjectBase,
    selectable: SelectMgrSelectableObjectBase,
    bounds: SharedState<BoundingBox3>,
    calls: AtomicUsize,
}
rustycad_tkernel::impl_transient!(BoxedSelectable, "Test_BoxedSelectable", dyn SelectMgrSelectableObject, transient,
    interfaces: [dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);
impl BoxedSelectable {
    /// Make a bound selectable; `BoxedSelectable::new(1.0, false)`.
    pub fn new(offset: f64, void: bool) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            presentable: PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            selectable: SelectMgrSelectableObjectBase::default(),
            bounds: SharedState::default(),
            calls: AtomicUsize::new(0),
        });
        object.set_bounds(offset, void);
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
    /// Change geometry without notifying the set; `object.set_bounds(2.0, false)`.
    pub fn set_bounds(&self, offset: f64, void: bool) {
        let mut bounds = BoundingBox3::default();
        if !void {
            bounds.update_bounds([offset, -2., -3.], [offset + 1., 4., 5.]);
        }
        *self.bounds.write() = bounds;
    }
    /// Count virtual bounding calls; `object.bounds_calls()`.
    pub fn bounds_calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}
impl PrsMgrPresentableObject for BoxedSelectable {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.presentable
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        _: &dyn HandleReference<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
    fn bounding_box(&self, bounds: &mut BoundingBox3) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        *bounds = self.bounds.read().clone();
    }
}
impl SelectMgrSelectableObject for BoxedSelectable {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.selectable
    }
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
