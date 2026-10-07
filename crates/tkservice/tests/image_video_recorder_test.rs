//! Direct translation of Image_VideoRecorder_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! The oracle build has no FFmpeg (`USE_FFMPEG=OFF`), so the `#ifdef HAVE_FFMPEG` bodies of
//! OpenVideoFile, InvalidParameters and WriteFrames compile to empty tests natively; they are
//! translated as the same empty tests.
#![allow(non_snake_case, non_camel_case_types)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::image::{ImageVideoParams, ImageVideoRecorder};

/// The fixture: a recorder created in SetUp and closed in TearDown.
struct Image_VideoRecorderTest {
    myRecorder: Handle<ImageVideoRecorder>,
}
impl Image_VideoRecorderTest {
    fn set_up() -> Self {
        Self {
            myRecorder: Handle::new(ImageVideoRecorder::default()),
        }
    }
}
impl Drop for Image_VideoRecorderTest {
    fn drop(&mut self) {
        if let Some(recorder) = self.myRecorder.get() {
            recorder.close();
        }
    }
}

#[test]
fn Image_VideoRecorderTest__DefaultConstructor() {
    let fixture = Image_VideoRecorderTest::set_up();
    assert!(!fixture.myRecorder.is_null());
    assert_eq!(0, fixture.myRecorder.get().unwrap().frame_count());
}

#[test]
fn Image_VideoRecorderTest__VideoParamsStructure() {
    let _fixture = Image_VideoRecorderTest::set_up();
    let mut params = ImageVideoParams::default();
    assert_eq!(0, params.width);
    assert_eq!(0, params.height);
    assert_eq!(0, params.fps_num);
    assert_eq!(1, params.fps_den);
    assert!(params.format.as_bytes().is_empty());
    assert!(params.video_codec.as_bytes().is_empty());
    assert!(params.pixel_format.as_bytes().is_empty());
    params.set_framerate(30);
    assert_eq!(30, params.fps_num);
    assert_eq!(1, params.fps_den);
    params.set_framerate_fraction(25, 2);
    assert_eq!(25, params.fps_num);
    assert_eq!(2, params.fps_den);
}

#[test]
fn Image_VideoRecorderTest__OpenVideoFile() {
    // HAVE_FFMPEG is not defined in the oracle configuration: the native body is empty.
    let _fixture = Image_VideoRecorderTest::set_up();
}

#[test]
fn Image_VideoRecorderTest__InvalidParameters() {
    // HAVE_FFMPEG is not defined in the oracle configuration: the native body is empty.
    let _fixture = Image_VideoRecorderTest::set_up();
}

#[test]
fn Image_VideoRecorderTest__WriteFrames() {
    // HAVE_FFMPEG is not defined in the oracle configuration: the native body is empty.
    let _fixture = Image_VideoRecorderTest::set_up();
}

#[test]
fn Image_VideoRecorderTest__CloseWithoutOpen() {
    let fixture = Image_VideoRecorderTest::set_up();
    let recorder = fixture.myRecorder.get().unwrap();
    recorder.close();
    assert_eq!(0, recorder.frame_count());
}
