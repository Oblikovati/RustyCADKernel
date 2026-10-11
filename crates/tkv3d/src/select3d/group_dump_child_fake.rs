//! Named native-equivalent diagnostic child injects typed failures, I/O failures and policy mutation.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitiveGroup,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::{bvh::BvhBox, gp::Point3};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};
pub(super) struct GroupDumpContext {
    pub(super) stage: AtomicI32,
    pub(super) trace: SharedState<Vec<i32>>,
    pub(super) parent: SharedState<Handle<Select3DSensitiveGroup>>,
}
impl GroupDumpContext {
    pub(super) fn new(parent: Handle<Select3DSensitiveGroup>) -> Self {
        // The parent handle is cleared explicitly after each test, matching the native context lifetime.
        Self {
            stage: AtomicI32::new(0),
            trace: SharedState::new(Vec::new()),
            parent: SharedState::new(parent),
        }
    }
}
pub(super) struct GroupDumpChildFake {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    context: Arc<GroupDumpContext>,
    identity: i32,
    invalid_bounds: bool,
}
rustycad_tkernel::impl_transient!(GroupDumpChildFake,"Martensite_GroupDumpChild",dyn Select3DSensitiveEntity,transient,interfaces:[dyn Select3DSensitiveEntity]);
impl GroupDumpChildFake {
    pub(super) fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        context: Arc<GroupDumpContext>,
        identity: i32,
        invalid_bounds: bool,
    ) -> Self {
        // Virtual diagnostics are independent of stored sphere geometry and group membership.
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            context,
            identity,
            invalid_bounds,
        }
    }
    fn diagnostic_failure(&self, stage: i32) -> Failure {
        // Every failure includes its offending callback identity and the expected operation.
        Failure::with_message(
            "Standard_Failure",
            &format!(
                "group dump callback {stage} child {}: expected successful diagnostic output",
                self.identity
            ),
        )
    }
}
impl Select3DSensitiveEntity for GroupDumpChildFake {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // The inherited owner fields must be visible through retained member references.
        &self.base
    }
    fn matches(
        &self,
        _volume: &dyn SelectBasicsSelectingVolumeManager,
        _pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // This diagnostic fixture never invents a geometric hit in its unrelated selection path.
        Ok(false)
    }
    fn nb_sub_elements(&self) -> i32 {
        // The native diagnostic child inherits the sphere's single primitive count.
        1
    }
    fn center_of_geometry(&self) -> Point3 {
        // Native group construction caches the same analytic coordinates before diagnostics run.
        Point3::new(f64::from(self.identity), 0., 0.)
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.context.trace.write().push(self.identity * 10 + 1);
        let stage = self.context.stage.load(Ordering::Relaxed);
        if stage == 4 && self.identity == 2 {
            super::triangulation_failure::native_checked::<()>(Err(self.diagnostic_failure(4)));
        }
        if self.invalid_bounds && stage == 0 {
            return Select3DBndBox3d::default();
        }
        let x = f64::from(self.identity);
        BvhBox::from_corners(
            Vec3::new(x - 0.25, -0.25, -0.25),
            Vec3::new(x + 0.25, 0.25, 0.25),
        )
    }
    fn dump_json(&self, output: &mut DumpStream, _depth: i32) -> io::Result<()> {
        self.context.trace.write().push(self.identity * 10 + 2);
        output.append(&format!("\"DumpStage\": {}", self.identity));
        if self.identity != 2 {
            return Ok(());
        }
        match self.context.stage.load(Ordering::Relaxed) {
            1 => super::triangulation_failure::native_checked(Err(self.diagnostic_failure(1))),
            2 => Err(io::Error::other(
                "group dump stream callback 2: expected successful diagnostic output",
            )),
            3 => {
                let parent = self.context.parent.read().clone();
                parent.set_match_type(false);
                parent.set_check_overlap_all(true);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
