//! Native noexcept context destruction must not discard a virtual Clear failure.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, v3d::V3dViewer};
struct FailingSelection {
    transient: StandardTransient,
    base: AisSelectionBase,
}
rustycad_tkernel::impl_transient!(FailingSelection,"FailingSelection",dyn AisSelectionSource,transient,interfaces:[dyn AisSelectionSource]);
impl AisSelectionSource for FailingSelection {
    fn selection_base(&self) -> &AisSelectionBase {
        &self.base
    }
    fn clear(&self) -> Result<(), Failure> {
        Err(Failure::with_message(
            "Standard_Failure",
            "Clear failed; expected successful fixture callback",
        ))
    }
}
#[test]
fn context_clear_failure_aborts() {
    if std::env::var_os("RUSTYCAD_AIS_FATAL_CHILD").is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "context_clear_failure_aborts"])
            .env("RUSTYCAD_AIS_FATAL_CHILD", "1")
            .output()
            .unwrap();
        assert!(!result.status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(result.status.signal(), Some(6));
        }
        return;
    }
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let selection = Handle::new(FailingSelection {
        transient: StandardTransient::default(),
        base: AisSelectionBase::new(),
    });
    context.set_selection(&selection.cast());
    drop(context);
    panic!("context destructor returned after Clear failure; expected process termination");
}
