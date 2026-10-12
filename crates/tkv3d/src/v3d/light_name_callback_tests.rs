//! Both genuine virtual callbacks run through public ToPixMap, with no GPU or encoding shortcut.
#![allow(unsafe_code)] // The named test buffer is anchored through the whole callback transaction.
use super::light_name_callback_fixture::{
    CallbackNameStorage, CallbackNameView, CallbackObservedView,
};
use super::{V3dImageDumpOptions, V3dLightName, V3dTypeOfView, V3dView, V3dViewSource, V3dViewer};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::Graphic3dBufferType, image::ImagePixMap,
    null_driver::NullGraphicDriver,
};

fn callback_view(
    viewer: &Handle<V3dViewer>,
    control: &std::sync::Arc<CallbackNameStorage>,
) -> Handle<dyn V3dViewSource> {
    let mut base = V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap();
    base.view = CallbackObservedView::create(&viewer.structure_manager(), control);
    let view = Handle::new(CallbackNameView {
        base,
        control: control.clone(),
    })
    .cast();
    V3dView::bind_source(&view).unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(8, 8);
    view.set_window(&window.cast(), 0).unwrap();
    view
}

#[test]
fn actual_redraw_mutation_is_copied_fresh_at_shadow_callback() {
    let control = CallbackNameStorage::new();
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = callback_view(&viewer, &control);
    control.overwrite(*b"sun\0\0\0\0\0");
    let name = unsafe { V3dLightName::from_ptr(control.pointer()) };
    let options = callback_options(name);
    assert!(!view.to_pix_map(&ImagePixMap::default(), &options).unwrap());
    let actual = callback_rows(&control, name);
    assert_eq!(
        actual,
        include_str!("../../tests/fixtures/light-name-redraw-callback-native.txt")
    );
}

fn callback_options(name: V3dLightName<'_>) -> V3dImageDumpOptions<'_> {
    V3dImageDumpOptions {
        width: 8,
        height: 8,
        buffer_type: Graphic3dBufferType::ShadowMap,
        light_name: name,
        ..Default::default()
    }
}

fn callback_rows(control: &CallbackNameStorage, name: V3dLightName<'_>) -> String {
    let consumed = control.rows.read().join("\n");
    let bytes = name.to_ascii_string().unwrap();
    let after = bytes
        .as_bytes()
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    format!("{consumed}\nafter {} {after}\n", bytes.len())
}
