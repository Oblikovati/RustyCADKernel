//! Named deterministic jobs and host fakes for background BVH tests.
use rustycad_tkernel::{
    message::{Gravity, Messenger, Printer, PrinterBase, PrinterType},
    osd::{SignalHost, SignalMode},
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, WeakHandle,
    },
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::{select_basics::*, select_mgr::SelectMgrBvhThreadPool, select3d::*};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Default)]
pub struct RecordingPoolSignals {
    pub enabled: AtomicBool,
    pub applied: SharedState<Vec<bool>>,
}
impl SignalHost for RecordingPoolSignals {
    fn set_floating_signal(&self, enable: bool) {
        self.applied.write().push(enable);
    }
    fn to_catch_floating_signals(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
    fn set_signal_handlers(&self, _: SignalMode) {}
    fn debug_mode_requested(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct RecordingPoolPrinter {
    base: PrinterBase,
    pub messages: SharedState<Vec<String>>,
}
impl Printer for RecordingPoolPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        assert_eq!(gravity, Gravity::Fail);
        self.messages
            .write()
            .push(String::from_utf8_lossy(text.as_bytes()).into_owned());
    }
}
pub fn pool_fixture(
    count: i32,
) -> (
    Handle<SelectMgrBvhThreadPool>,
    Arc<RecordingPoolSignals>,
    Arc<RecordingPoolPrinter>,
) {
    let signals = Arc::new(RecordingPoolSignals::default());
    signals.enabled.store(true, Ordering::Release);
    let printer = Arc::new(RecordingPoolPrinter::default());
    let messenger = Arc::new(Messenger::with_printer(printer.clone()));
    let pool = Handle::new(SelectMgrBvhThreadPool::with_services(
        count,
        messenger,
        signals.clone(),
    ));
    (pool, signals, printer)
}

pub struct PoolBuildJob {
    transient: StandardTransient,
    sensitive: Select3DSensitiveEntityBase,
    pub calls: AtomicUsize,
    pub buildable: AtomicBool,
    pub outcome: i32,
    pub inspect_pool: WeakHandle<SelectMgrBvhThreadPool>,
    pub followup: Handle<dyn Select3DSensitiveEntity>,
    pub entered: Option<Arc<Barrier>>,
    pub release: Option<Arc<Barrier>>,
    pub order: Arc<SharedState<Vec<i32>>>,
    pub tag: i32,
}
rustycad_tkernel::impl_transient!(PoolBuildJob, "Test_PoolBuildJob", dyn Select3DSensitiveEntity, transient, interfaces: [dyn Select3DSensitiveEntity]);
impl PoolBuildJob {
    pub fn new(tag: i32, outcome: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            sensitive: Select3DSensitiveEntityBase::new(&Handle::default()),
            calls: AtomicUsize::new(0),
            buildable: AtomicBool::new(true),
            outcome,
            inspect_pool: WeakHandle::default(),
            followup: Handle::default(),
            entered: None,
            release: None,
            order: Arc::new(SharedState::default()),
            tag,
        }
    }
}
impl Select3DSensitiveEntity for PoolBuildJob {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.sensitive
    }
    fn matches(
        &self,
        _: &dyn SelectBasicsSelectingVolumeManager,
        _: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        Select3DBndBox3d::default()
    }
    fn center_of_geometry(&self) -> Point3 {
        Point3::default()
    }
    fn to_build_bvh(&self) -> bool {
        self.buildable.load(Ordering::Acquire)
    }
    fn bvh(&self) -> Result<(), Failure> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        self.order.write().push(self.tag);
        if let Some(entered) = &self.entered {
            entered.wait();
        }
        if let Some(release) = &self.release {
            release.wait();
        }
        let pool = self.inspect_pool.upgrade();
        if let Some(pool) = pool.get() {
            assert!(!pool.threads().is_empty());
            if self.followup.is_some() {
                pool.add_entity(&self.followup)?;
            }
        }
        match self.outcome {
            1 => Err(Failure::new("fixture failure")),
            2 => panic!("fixture panic"),
            _ => Ok(()),
        }
    }
}
