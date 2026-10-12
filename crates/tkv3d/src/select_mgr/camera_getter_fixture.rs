//! Real rectangular setters mutate captured fields before named failure/retry observations.
use super::{camera_argument_fixture::CameraArgumentFrustum, *};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, WeakHandle, raise_native_failure,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
pub(super) struct CameraGetterSetterProbe {
    next: WeakHandle<Graphic3dCamera>,
    armed: AtomicBool,
    fault: i32,
}
impl CameraGetterSetterProbe {
    pub(super) fn new(next: &Handle<Graphic3dCamera>, fault: i32) -> Arc<Self> {
        Arc::new(Self {
            next: next.downgrade(),
            armed: AtomicBool::new(true),
            fault,
        })
    }
    pub(super) fn modify_captured_field(&self, original: &CameraArgumentFrustum) {
        if !self.armed.swap(false, Ordering::Relaxed) {
            return;
        }
        original.volume.set_camera(&self.next.observe().clone());
        self.raise_getter_setter_failure();
    }
    fn raise_getter_setter_failure(&self) {
        match self.fault {
            1 => raise_native_failure(Err(Failure::new(
                "Camera getter setter fault=1; expected success",
            ))),
            2 => std::panic::panic_any(107_u32),
            _ => {}
        }
    }
}
pub(super) fn getter_test_volume() -> Handle<CameraArgumentFrustum> {
    let view: Handle<dyn V3dViewSource> = Handle::default();
    let camera = Arc::new(SharedState::new(Handle::default()));
    let volume = CameraArgumentFrustum::new(&view, &camera, 0);
    volume.armed.store(false, Ordering::Relaxed);
    volume
}
pub(super) struct CameraGetterFields<'a> {
    pub alias: &'a dyn HandleReference<Graphic3dCamera>,
    pub fresh: &'a dyn HandleReference<Graphic3dCamera>,
    pub original: &'a dyn HandleReference<Graphic3dCamera>,
    pub empty: &'a dyn HandleReference<Graphic3dCamera>,
}
pub(super) struct CameraGetterCounts<'a> {
    pub first: &'a Handle<Graphic3dCamera>,
    pub second: &'a Handle<Graphic3dCamera>,
    pub owner_counts: [i32; 2],
}
pub(super) fn camera_getter_row(
    labels: [i32; 3],
    fields: CameraGetterFields<'_>,
    counts: CameraGetterCounts<'_>,
) -> String {
    let mut values = labels.to_vec();
    values.extend(camera_getter_value_codes(&fields, &counts));
    values.extend([counts.first.ref_count(), counts.second.ref_count()]);
    values.extend(counts.owner_counts);
    values.extend(camera_getter_alias_identities(&fields));
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
fn camera_getter_code(
    value: &dyn HandleReference<Graphic3dCamera>,
    counts: &CameraGetterCounts<'_>,
) -> i32 {
    if value.is_null() {
        return 0;
    }
    if value.object_address() == counts.first.object_address() {
        return 1;
    }
    if value.object_address() == counts.second.object_address() {
        return 2;
    }
    3
}

fn camera_getter_value_codes(
    fields: &CameraGetterFields<'_>,
    counts: &CameraGetterCounts<'_>,
) -> [i32; 3] {
    [
        camera_getter_code(fields.alias, counts),
        camera_getter_code(fields.fresh, counts),
        camera_getter_code(fields.original, counts),
    ]
}
fn camera_getter_alias_identities(fields: &CameraGetterFields<'_>) -> [i32; 3] {
    [
        i32::from(fields.alias.reference_address() == fields.original.reference_address()),
        i32::from(fields.alias.reference_address() == fields.empty.reference_address()),
        i32::from(fields.alias.reference_address() == fields.fresh.reference_address()),
    ]
}
