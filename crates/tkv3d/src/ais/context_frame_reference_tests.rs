//! Native viewer/manager references keep field identity and observe replacements across owner callbacks.
use super::{context_frame_reference_fixture::*, *};
use crate::{
    prs_mgr::PrsMgrPresentationManager, select_mgr::SelectMgrEntityOwnerSource, v3d::V3dViewer,
};
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use std::panic::{AssertUnwindSafe, catch_unwind};
struct ContextFrameReferenceScene {
    original: Handle<V3dViewer>,
    replacement: Handle<V3dViewer>,
    context: Handle<AisInteractiveContext>,
    first: Handle<PrsMgrPresentationManager>,
    second: Handle<PrsMgrPresentationManager>,
    owner: Handle<ViewerManagerCallbackOwner>,
}
#[test]
fn native_context_references_release_locks_and_observe_owner_callback_replacement_and_retry() {
    let rows: Vec<String> = (0..3).flat_map(context_frame_reference_rows).collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/context-viewer-manager-callbacks-native.txt")
    );
}
fn context_frame_reference_rows(mode: i32) -> Vec<String> {
    let scene = context_frame_reference_scene(mode);
    (0..2)
        .map(|frame| context_frame_reference_row(mode, frame, &scene))
        .collect()
}
fn context_frame_reference_scene(mode: i32) -> ContextFrameReferenceScene {
    let original = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let replacement = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&original).unwrap();
    let first = context.main_prs_mgr_ref().clone_handle();
    let second = PrsMgrPresentationManager::new(&replacement.structure_manager());
    let owner = ViewerManagerCallbackOwner::new(&context, &replacement, &second, mode);
    ContextFrameReferenceScene {
        original,
        replacement,
        context,
        first,
        second,
        owner,
    }
}
fn context_frame_reference_row(
    mode: i32,
    frame: i32,
    scene: &ContextFrameReferenceScene,
) -> String {
    let failure = apply_context_frame_reference(scene);
    let mut row = vec![mode, frame, failure];
    row.extend(*scene.owner.records.read());
    row.extend(context_frame_reference_state(scene));
    row.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
fn context_frame_reference_state(scene: &ContextFrameReferenceScene) -> [i32; 6] {
    [
        i32::from(
            scene.context.current_viewer_ref().object_address()
                == scene.replacement.object_address(),
        ),
        i32::from(
            scene.context.main_prs_mgr_ref().object_address() == scene.second.object_address(),
        ),
        scene.original.ref_count(),
        scene.replacement.ref_count(),
        scene.first.ref_count(),
        scene.second.ref_count(),
    ]
}
fn apply_context_frame_reference(scene: &ContextFrameReferenceScene) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        catch_native_failure(|| {
            scene.owner.update_highlight_trsf(
                &scene.context.current_viewer_ref(),
                &scene.context.main_prs_mgr_ref(),
                0,
            )
        })
    }));
    context_frame_reference_failure(result)
}
fn context_frame_reference_failure(
    result: Result<Result<(), Failure>, Box<dyn std::any::Any + Send>>,
) -> i32 {
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(failure)) => {
            assert!(
                failure
                    .message()
                    .contains("viewer manager callback fault=1")
            );
            1
        }
        Err(payload) => {
            assert_eq!(payload.downcast_ref::<u32>(), Some(&73));
            2
        }
    }
}
