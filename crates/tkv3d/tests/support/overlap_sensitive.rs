//! Named client entity observes unlocked selector callbacks and replaces its owner slot.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkv3d::{select_basics::*, select_mgr::*, select3d::*};
use std::sync::atomic::{AtomicI32, Ordering};
pub struct OverlapSensitivePoint {
    transient: StandardTransient,
    point: Select3DSensitivePoint,
    pub pick: SharedState<SelectBasicsPickResult>,
    pub decision: AtomicI32,
    pub match_owner: SharedState<Option<Handle<dyn SelectMgrEntityOwnerSource>>>,
    pub location_owner: SharedState<Option<Handle<dyn SelectMgrEntityOwnerSource>>>,
    pub selector: SharedState<WeakHandle<dyn SelectMgrViewerSelectorSource>>,
    pub events: SharedState<Vec<i32>>,
}
rustycad_tkernel::impl_transient!(OverlapSensitivePoint,"OverlapSensitivePoint",dyn Select3DSensitiveEntity,transient,interfaces:[dyn Select3DSensitiveEntity]);
impl OverlapSensitivePoint {
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            point: Select3DSensitivePoint::new(owner, Point3::default()),
            pick: SharedState::default(),
            decision: AtomicI32::new(1),
            match_owner: SharedState::default(),
            location_owner: SharedState::default(),
            selector: SharedState::default(),
            events: SharedState::default(),
        })
    }
    fn observe(&self, event: i32) {
        self.events.write().push(event);
        if let Some(selector) = self.selector.read().upgrade().get() {
            let _ = selector
                .viewer_selector_base()
                .fields
                .read()
                .results
                .stored
                .len();
        }
    }
}
impl Select3DSensitiveEntity for OverlapSensitivePoint {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.point.sensitive_base()
    }
    fn matches(
        &self,
        _: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.observe(1);
        if let Some(owner) = self.match_owner.read().clone() {
            self.set(&owner);
        }
        *pick = *self.pick.read();
        if self.decision.load(Ordering::Acquire) == 2 {
            return Err(Failure::with_message(
                "Standard_Failure",
                "fixture match failed; expected decision 0 or 1",
            ));
        }
        Ok(self.decision.load(Ordering::Acquire) != 0)
    }
    fn has_init_location(&self) -> bool {
        self.observe(2);
        if let Some(owner) = self.location_owner.read().clone() {
            self.set(&owner);
        }
        false
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        GeneralTransform3::default()
    }
    fn nb_sub_elements(&self) -> i32 {
        self.point.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.point.bounding_box()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.point.center_of_geometry()
    }
}
