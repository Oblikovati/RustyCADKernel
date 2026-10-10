//! Named heavyweight entity makes prebuild thresholds observable without huge geometry.
use rustycad_tkernel::{
    ncollection::Vec3,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::{select_basics::*, select3d::*};
use std::sync::atomic::{AtomicUsize, Ordering};
pub struct PrebuildEntity {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    pub count: i32,
    pub calls: AtomicUsize,
    pub fail: bool,
    completion: (std::sync::Mutex<bool>, std::sync::Condvar),
}
rustycad_tkernel::impl_transient!(PrebuildEntity,"Test_PrebuildEntity",dyn Select3DSensitiveEntity,transient,interfaces:[dyn Select3DSensitiveEntity]);
impl PrebuildEntity {
    pub fn wait_built(&self) {
        let (lock, ready) = &self.completion;
        let (completed, _) = ready
            .wait_timeout_while(
                lock.lock().unwrap(),
                std::time::Duration::from_secs(5),
                |completed| !*completed,
            )
            .unwrap();
        assert!(
            *completed,
            "expected queued entity to build within five seconds"
        );
    }

    pub fn new(count: i32, fail: bool) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(&Handle::default()),
            count,
            calls: AtomicUsize::new(0),
            fail,
            completion: (std::sync::Mutex::new(false), std::sync::Condvar::new()),
        })
    }
}
impl Select3DSensitiveEntity for PrebuildEntity {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    fn matches(
        &self,
        _: &dyn SelectBasicsSelectingVolumeManager,
        _: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.count
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        Select3DBndBox3d::from_corners(Vec3::new(0., 0., 0.), Vec3::new(1., 1., 1.))
    }
    fn center_of_geometry(&self) -> Point3 {
        Point3::new(0.5, 0.5, 0.5)
    }
    fn bvh(&self) -> Result<(), Failure> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        *self.completion.0.lock().unwrap() = true;
        self.completion.1.notify_all();
        if self.fail {
            return Err(Failure::with_message(
                "Standard_Failure",
                "prebuild fixture failure; expected a buildable entity",
            ));
        }
        Ok(())
    }
    fn to_build_bvh(&self) -> bool {
        true
    }
}
