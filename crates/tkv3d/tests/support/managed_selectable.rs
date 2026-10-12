//! Named object records callbacks, hierarchy traversal and recoverable compute failures.
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{prs_mgr::*, select_mgr::*, select3d::*};
use std::{
    sync::Arc,
    sync::atomic::{AtomicI32, Ordering},
};

pub struct ManagedSelectionObject {
    transient: StandardTransient,
    presentable: PrsMgrPresentableObjectBase,
    selectable: SelectMgrSelectableObjectBase,
    pub tag: i32,
    pub provided: SharedState<Option<Vec<Handle<dyn Select3DSensitiveEntity>>>>,
    pub fail_mode: AtomicI32,
    pub empty: AtomicI32,
    pub events: Arc<SharedState<Vec<i32>>>,
}
rustycad_tkernel::impl_transient!(ManagedSelectionObject,"Test_ManagedSelectionObject",dyn SelectMgrSelectableObject,transient,
    interfaces:[dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl ManagedSelectionObject {
    pub fn new(tag: i32, events: Arc<SharedState<Vec<i32>>>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            presentable: PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            selectable: SelectMgrSelectableObjectBase::default(),
            tag,
            provided: SharedState::default(),
            fail_mode: AtomicI32::new(-999),
            empty: AtomicI32::new(0),
            events,
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    pub fn own_presentations(&self, own: bool) {
        self.presentable.fields.write().has_own_presentations = own;
    }
    pub fn erased(&self, erased: bool) {
        self.presentable.fields.write().display_status = if erased {
            PrsMgrDisplayStatus::Erased
        } else {
            PrsMgrDisplayStatus::Displayed
        };
    }
}
impl PrsMgrPresentableObject for ManagedSelectionObject {
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
        bounds.update_bounds([0., 0., 0.], [1., 1., 1.]);
    }
    fn update_transformation(&self) -> Result<(), Failure> {
        self.events.write().push(200 + self.tag);
        select_mgr_update_transformation(self)
    }
}
impl SelectMgrSelectableObject for ManagedSelectionObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.events.write().push(self.tag * 10 + mode);
        if self.fail_mode.load(Ordering::Acquire) == mode {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!("fixture failed mode {mode}; expected a successful mode"),
            ));
        }
        if self.empty.load(Ordering::Acquire) != 0 {
            return Ok(());
        }
        if append_provided_fixture_entities(&self.provided, selection)? {
            return Ok(());
        }
        append_managed_fixture_point(selection, mode)
    }

    fn update_transformations(&self, _: &Handle<SelectMgrSelection>) {
        self.events.write().push(100 + self.tag);
    }
}

fn append_managed_fixture_point(
    selection: &dyn HandleReference<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let owner = SelectMgrEntityOwner::new(1).cast();
    let point = Handle::new(Select3DSensitivePoint::new(
        &owner,
        Point3::new(f64::from(mode), 0., 0.),
    ));
    point.set_sensitivity_factor(mode + 2)?;
    selection.observe().add(&point.cast())
}
fn append_provided_fixture_entities(
    provided: &SharedState<Option<Vec<Handle<dyn Select3DSensitiveEntity>>>>,
    selection: &dyn HandleReference<SelectMgrSelection>,
) -> Result<bool, Failure> {
    let Some(entities) = provided.snapshot() else {
        return Ok(false);
    };
    for entity in entities {
        selection.observe().add(&entity)?;
    }
    Ok(true)
}
