//! Exact native descriptor counters require an exclusive test observation window.
use std::sync::{Mutex, MutexGuard, PoisonError};
/// Guard only shared descriptor-count assertions; `DescriptorCountScope::acquire(&scope)`.
pub(super) struct DescriptorCountScope<'a> {
    _permit: MutexGuard<'a, ()>,
}
impl<'a> DescriptorCountScope<'a> {
    /// Exclude other count observers until native owners drop; `Self::acquire(&scope)`.
    pub(super) fn acquire(scope: &'a Mutex<()>) -> Self {
        // Concurrent real owners correctly change the counter, so snapshots must exclude them.
        Self {
            _permit: scope.lock().unwrap_or_else(PoisonError::into_inner),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::{RuntimeTyped, StandardTransient, Transient};
    use std::sync::{
        Arc,
        mpsc::{Receiver, SyncSender, sync_channel},
    };
    #[derive(Default)]
    struct DescriptorScopeProbe {
        base: StandardTransient,
    }
    rustycad_tkernel::impl_transient!(
        DescriptorScopeProbe,
        "Martensite_DescriptorScopeProbe",
        StandardTransient,
        base
    );
    #[test]
    fn shared_count_scope_preserves_each_actual_owner_during_concurrent_attempts() {
        let baseline = DescriptorScopeProbe::static_type()
            .transient_base()
            .ref_count();
        let scope = Arc::new(Mutex::new(()));
        let (ready, observed) = sync_channel(1);
        let (release, held) = sync_channel(1);
        let signals = DescriptorOwnerSignals {
            ready: Some(ready),
            release: Some(held),
        };
        let first = spawn_count_owner(scope.clone(), signals);
        assert_eq!(observed.recv().unwrap(), baseline + 1);
        assert!(scope.try_lock().is_err());
        let second = spawn_count_owner(scope, DescriptorOwnerSignals::default());
        release.send(()).unwrap();
        assert_eq!(first.join().unwrap(), baseline + 1);
        assert_eq!(second.join().unwrap(), baseline + 1);
    }
    #[derive(Default)]
    struct DescriptorOwnerSignals {
        ready: Option<SyncSender<i32>>,
        release: Option<Receiver<()>>,
    }
    fn spawn_count_owner(
        scope: Arc<Mutex<()>>,
        signals: DescriptorOwnerSignals,
    ) -> std::thread::JoinHandle<i32> {
        std::thread::spawn(move || {
            let _permit = DescriptorCountScope::acquire(&scope);
            let owner = DescriptorScopeProbe::static_type_handle();
            if let Some(ready) = signals.ready {
                ready.send(owner.ref_count()).unwrap();
            }
            if let Some(release) = signals.release {
                release.recv().unwrap();
            }
            owner.ref_count()
        })
    }
}
