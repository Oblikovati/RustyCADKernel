//! Graphic3d_MediaTextureSet without FFmpeg: the frame queue, the callback, the player and
//! the shader programs.
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{Graphic3dMediaTextureSet, Graphic3dShaderProgram};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn frame_queue_presents_released_frames() {
    let set = Graphic3dMediaTextureSet::new().expect("set");
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    set.set_callback(Some(Arc::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
    })));
    let queue = set.frame_queue();
    let frame = queue.lock_frame();
    assert!(frame.is_locked() && queue.lock_frame().is_null());
    queue.release_frame(&frame);
    assert!(!frame.is_locked() && queue.lock_frame().is_null());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!set.swap_frames());
}

#[test]
fn player_and_programs() {
    let set = Graphic3dMediaTextureSet::new().expect("set");
    set.open_input(&AsciiString::default(), false);
    assert!(set.player_context().is_null());
    set.open_input(&AsciiString::from("clip.mp4"), true);
    assert!(!set.player_context().is_null() && set.input().is_equal(AsciiString::from("clip.mp4")));
    assert!(!set.swap_frames() && set.progress() == 0.0);
    let header =
        |program: &rustycad_tkernel::standard::Handle<Graphic3dShaderProgram>| program.header();
    assert!(header(&set.shader_yuv()).is_equal(AsciiString::from("#version 150")));
    assert_eq!(set.shader_yuvj().shader_objects().len(), 2);
    assert!(set.shader_program().is_null());
}
