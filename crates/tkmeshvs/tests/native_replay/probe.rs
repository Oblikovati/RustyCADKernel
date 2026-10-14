// SPDX-License-Identifier: GPL-2.0-only
//! Named derived override records actual intrusive counts and preserves faults after copy.
use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerSource, meshvs_drawer_assign};
use rustycad_tkernel::standard::{
    Handle, HandleReference, SharedState, Transient, catch_native_failure, raise_native_failure,
};
use std::{fmt::Write, sync::Arc};
struct AssignProbe {
    base: MeshVSDrawer,
    fault: Arc<SharedState<i32>>,
    rows: Arc<SharedState<String>>,
}
rustycad_tkernel::impl_transient!(AssignProbe,"AssignProbe",MeshVSDrawer,base,interfaces:[dyn MeshVSDrawerSource]);
impl MeshVSDrawerSource for AssignProbe {
    fn drawer_base(&self) -> &MeshVSDrawer {
        &self.base
    }
    fn assign(&self, source: &dyn HandleReference<dyn MeshVSDrawerSource>) {
        let observed = source.observe();
        writeln!(
            *self.rows.write(),
            "callback {} {}",
            self.transient_base().ref_count(),
            observed.get().unwrap().transient_base().ref_count()
        )
        .unwrap();
        meshvs_drawer_assign(&self.base, source);
        raise_assign_fault(*self.fault.read());
    }
}
fn raise_assign_fault(fault: i32) {
    match fault {
        1 => raise_native_failure::<()>(Err(
            rustycad_tkernel::standard::exceptions::construction_error("drawer assign after copy"),
        )),
        2 => std::panic::panic_any(ForeignAssignFault),
        _ => (),
    }
}
#[derive(Debug)]
struct ForeignAssignFault;
pub(super) fn dispatch(source: &Handle<MeshVSDrawer>, rows: &mut String) {
    let shared = Arc::new(SharedState::new(String::new()));
    let fault = Arc::new(SharedState::new(0));
    let target = Handle::new(AssignProbe {
        base: MeshVSDrawer::default(),
        fault: fault.clone(),
        rows: shared.clone(),
    })
    .into_cast::<dyn MeshVSDrawerSource>();
    for selected in 0..3 {
        *fault.write() = selected;
        let result = invoke_assignment(&target, source);
        rows.push_str(&std::mem::take(&mut *shared.write()));
        writeln!(rows, "return {}", fault_code(result)).unwrap();
        after_row(&target, source, rows);
    }
}
fn invoke_assignment(
    target: &Handle<dyn MeshVSDrawerSource>,
    source: &Handle<MeshVSDrawer>,
) -> std::thread::Result<Result<(), rustycad_tkernel::standard::Failure>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        catch_native_failure(|| {
            target.assign(source);
            Ok(())
        })
    }))
}
fn fault_code(result: std::thread::Result<Result<(), rustycad_tkernel::standard::Failure>>) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(cause)) => {
            assert!(cause.is_kind("Standard_ConstructionError"));
            1
        }
        Err(cause) => {
            assert!(cause.is::<ForeignAssignFault>());
            2
        }
    }
}

fn after_row(
    target: &Handle<dyn MeshVSDrawerSource>,
    source: &Handle<MeshVSDrawer>,
    rows: &mut String,
) {
    let mut value = 0;
    target.get_integer(7, &mut value);
    writeln!(
        rows,
        "after {} {} {}",
        value,
        source.ref_count(),
        target.ref_count()
    )
    .unwrap();
}
