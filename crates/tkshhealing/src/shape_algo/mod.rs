// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAlgo at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeAlgo: the container of the Shape Healing algorithms that data
//! exchange uses (`ShapeAlgo_AlgoContainer`), its tool factory
//! (`ShapeAlgo_ToolContainer`) and the process-wide current container.
mod algo_container;
mod approx_bspline;
mod homo_wires;
mod tool_container;

pub use algo_container::{ShapeAlgoAlgoContainer, ShapeAlgoWireConnection};
pub use tool_container::ShapeAlgoToolContainer;

use crate::shape_extend::ShapeExtend;
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The native static `theContainer`.
static CONTAINER: Mutex<Option<Handle<ShapeAlgoAlgoContainer>>> = Mutex::new(None);
/// The native function-local `init` flag of `Init`.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// The package `ShapeAlgo` (static functions only).
///
/// ```
/// use rustycad_tkshhealing::shape_algo::ShapeAlgo;
/// ShapeAlgo::init().unwrap();
/// assert!(!ShapeAlgo::algo_container().is_null());
/// ```
pub struct ShapeAlgo;

impl ShapeAlgo {
    /// `Init()`: once per process, a new algorithm container becomes current and
    /// ShapeExtend::Init loads the messages; `ShapeAlgo::init()?`.
    pub fn init() -> Result<(), Failure> {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        Self::set_algo_container(&Handle::new(ShapeAlgoAlgoContainer::new()));
        ShapeExtend::init()
    }

    /// `SetAlgoContainer(aContainer)`: the current container (a null handle is kept);
    /// `ShapeAlgo::set_algo_container(&container)`.
    pub fn set_algo_container(container: &Handle<ShapeAlgoAlgoContainer>) {
        *lock() = Some(container.clone());
    }

    /// `AlgoContainer()`: the current container, null before `Init` or `SetAlgoContainer`;
    /// `ShapeAlgo::algo_container()`.
    pub fn algo_container() -> Handle<ShapeAlgoAlgoContainer> {
        lock().clone().unwrap_or_default()
    }
}

fn lock() -> std::sync::MutexGuard<'static, Option<Handle<ShapeAlgoAlgoContainer>>> {
    // A panic while the lock is held cannot leave the handle half written.
    CONTAINER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
