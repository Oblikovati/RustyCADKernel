//! Named sensitive-child callbacks expose defined exception and partial-state behavior.
use super::triangulation_failure::native_checked;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers, Select3DSensitiveSphere,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::Point3;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI32, Ordering},
};
pub(super) struct GroupFaultContext {
    pub(super) stage: AtomicI32,
    pub(super) target: i32,
    pub(super) miss: i32,
    pub(super) depth: i32,
    pub(super) trace: Mutex<Vec<i32>>,
}
impl GroupFaultContext {
    pub(super) fn new(target: i32) -> Self {
        // Shared injected context keeps callback evidence independent of any global state.
        Self {
            stage: AtomicI32::new(0),
            target,
            miss: 3,
            depth: 0,
            trace: Mutex::new(Vec::new()),
        }
    }
    fn record(&self, child: i32, operation: i32) {
        // The native trace identifies both the child and the exact virtual callback.
        self.trace.lock().unwrap().push(child * 10 + operation);
    }
    fn check(&self, child: i32, operation: i32) -> Result<(), Failure> {
        if self.stage.load(Ordering::Relaxed) == operation && self.target == child {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!(
                    "group child {child} callback {operation}: expected successful sensitive entity operation"
                ),
            ));
        }
        Ok(())
    }
}
pub(super) struct GroupFaultChildFake {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    context: Arc<GroupFaultContext>,
    identity: i32,
}
rustycad_tkernel::impl_transient!(GroupFaultChildFake, "GroupFaultChild", dyn Select3DSensitiveEntity, transient, interfaces:[dyn Select3DSensitiveEntity]);
impl GroupFaultChildFake {
    pub(super) fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        context: Arc<GroupFaultContext>,
        identity: i32,
    ) -> Self {
        // Each member has independent owner fields but records into the injected shared scenario.
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            context,
            identity,
        }
    }
    fn callback_depth(&self) -> f64 {
        match self.context.depth {
            1 => 9.,
            value if value == self.identity + 1 => f64::NAN,
            _ => f64::from(10 - self.identity),
        }
    }
    fn analytic_center(&self) -> Point3 {
        // The fault fixture retains ordinary sphere geometry beneath its virtual extension.
        Point3::new(f64::from(self.identity), 0., 0.)
    }
}
impl Select3DSensitiveEntity for GroupFaultChildFake {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Composite Set must update this exact retained child field.
        &self.base
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.context.record(self.identity, 1);
        native_checked(self.context.check(self.identity, 1));
        let x = f64::from(self.identity);
        BvhBox::from_corners(
            Vec3::new(x - 0.25, -0.25, -0.25),
            Vec3::new(x + 0.25, 0.25, 0.25),
        )
    }
    fn center_of_geometry(&self) -> Point3 {
        self.context.record(self.identity, 2);
        native_checked(self.context.check(self.identity, 2));
        self.analytic_center()
    }
    fn matches(
        &self,
        _volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.context.record(self.identity, 3);
        pick.set_depth(self.callback_depth());
        pick.set_picked_point(Point3::new(f64::from(self.identity), 4., 5.));
        self.context.check(self.identity, 3)?;
        Ok(self.identity != self.context.miss)
    }
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.context.record(self.identity, 4);
        self.base.fields.write().owner_id = owner.clone();
        native_checked(self.context.check(self.identity, 4));
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        self.context.record(self.identity, 5);
        self.context.check(self.identity, 5)?;
        Ok(Handle::new(Select3DSensitiveSphere::new(
            &self.owner_id(),
            self.analytic_center(),
            0.25,
        ))
        .cast())
    }
    fn nb_sub_elements(&self) -> i32 {
        // Native fault children inherit the single analytic sphere primitive count.
        1
    }
}

pub(super) fn fault_children(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    context: &Arc<GroupFaultContext>,
) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
    // Child order drives observable early failure, owner propagation and reusable-pick semantics.
    (1..=3)
        .map(|identity| {
            Handle::new(GroupFaultChildFake::new(owner, context.clone(), identity)).cast()
        })
        .collect()
}
pub(super) fn fault_trace_columns(
    context: &GroupFaultContext,
    ordinal: i32,
    step: i32,
) -> Vec<f64> {
    let trace = context.trace.lock().unwrap();
    let mut columns = vec![f64::from(ordinal), f64::from(step), trace.len() as f64];
    columns.extend(trace.iter().map(|value| f64::from(*value)));
    columns
}
