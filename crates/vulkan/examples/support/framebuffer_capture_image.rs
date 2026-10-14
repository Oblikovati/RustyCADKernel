//! Completed framebuffer publication uses owned image storage and atomic file replacement.
#![allow(dead_code)]
use martensite_vulkan::FrameCapture;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::image::{ImageAlienPixMap, ImageFormat, ImagePixMap};
use std::{io, path::Path};
/// Inject publication I/O; e.g. publish_capture(prefix, frame, &FileFrameImageStore).
pub trait FrameImageStore {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    fn write_image(&self, path: &Path, image: &ImagePixMap) -> io::Result<()>;
    fn rename(&self, temporary: &Path, target: &Path) -> io::Result<()>;
}
pub struct FileFrameImageStore;
impl FrameImageStore for FileFrameImageStore {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(path, bytes)
    }
    fn write_image(&self, path: &Path, image: &ImagePixMap) -> io::Result<()> {
        let output = ImageAlienPixMap::default();
        if !output.init_copy(image) {
            return Err(io::Error::other("expected owned image copy"));
        }
        let name = path
            .to_str()
            .ok_or_else(|| io::Error::other(format!("path={path:?}; expected UTF8")))?;
        let saved = output
            .save(&AsciiString::from(name))
            .map_err(io::Error::other)?;
        saved
            .then_some(())
            .ok_or_else(|| io::Error::other(format!("image={path:?}; expected saved PPM")))
    }
    fn rename(&self, temporary: &Path, target: &Path) -> io::Result<()> {
        std::fs::rename(temporary, target)
    }
}
/// Publish color/depth and an actual image after rendering; e.g. publish_capture(Path::new("frame"),&frame,&FileFrameImageStore).
pub fn publish_capture(
    prefix: &Path,
    frame: &FrameCapture,
    store: &impl FrameImageStore,
) -> io::Result<()> {
    let image = owned_capture_image(frame)?;
    publish_bytes(&capture_artifact_path(prefix, "rgba"), &frame.rgba, store)?;
    let depth: Vec<_> = frame
        .depth
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    publish_bytes(&capture_artifact_path(prefix, "depth"), &depth, store)?;
    let temporary = capture_artifact_path(prefix, "tmp.ppm");
    store.write_image(&temporary, &image)?;
    store.rename(&temporary, &capture_artifact_path(prefix, "ppm"))
}
fn capture_artifact_path(prefix: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = prefix.as_os_str().to_os_string();
    name.push(format!(".{suffix}"));
    name.into()
}

fn owned_capture_image(frame: &FrameCapture) -> io::Result<ImagePixMap> {
    validate_capture_extent(frame)?;
    let image = ImagePixMap::default();
    if !image.init_wrapper(
        ImageFormat::Rgba,
        Some(frame.rgba.clone()),
        frame.width as usize,
        frame.height as usize,
        0,
    ) {
        return Err(io::Error::other("expected owned RGBA ImagePixMap"));
    }
    image.set_top_down(true);
    Ok(image)
}
fn validate_capture_extent(frame: &FrameCapture) -> io::Result<()> {
    let expected = (frame.width as usize)
        .checked_mul(frame.height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    let Some(bytes) = expected else {
        return Err(invalid_capture_shape(frame, expected));
    };
    if frame.rgba.len() != bytes || frame.depth.len() != bytes / 4 || bytes == 0 {
        return Err(invalid_capture_shape(frame, expected));
    }
    Ok(())
}
fn invalid_capture_shape(frame: &FrameCapture, expected: Option<usize>) -> io::Error {
    let shape = match expected {
        Some(bytes) => format!(
            "{bytes} RGBA bytes and {} depth values for positive extent",
            bytes / 4
        ),
        None => "positive extent with representable usize RGBA byte length".into(),
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "RGBA bytes={}, depth values={}, extent={}x{}; expected {shape}",
            frame.rgba.len(),
            frame.depth.len(),
            frame.width,
            frame.height
        ),
    )
}

fn publish_bytes(path: &Path, bytes: &[u8], store: &impl FrameImageStore) -> io::Result<()> {
    // Oblikovati viewport_screenshot.go publishes only after completed render via temporary→rename.
    let temporary = path.with_extension("tmp");
    store.write(&temporary, bytes)?;
    store.rename(&temporary, path)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap, path::PathBuf};
    #[derive(Default)]
    struct FakeFrameImageStore {
        files: RefCell<BTreeMap<PathBuf, Vec<u8>>>,
        fail_write: bool,
        fail_rename: bool,
    }
    impl FrameImageStore for FakeFrameImageStore {
        fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
            if self.fail_write {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "named fake denied write",
                ));
            }
            self.files.borrow_mut().insert(path.into(), bytes.into());
            Ok(())
        }
        fn write_image(&self, path: &Path, image: &ImagePixMap) -> io::Result<()> {
            assert!(image.is_top_down());
            assert_eq!((image.width(), image.height()), (1, 1));
            assert_eq!(image.pixel_color(0, 0, false).unwrap().rgb().green(), 1.);
            self.write(path, b"named fake serialized image")
        }
        fn rename(&self, temporary: &Path, target: &Path) -> io::Result<()> {
            if self.fail_rename {
                return Err(io::Error::other("named fake denied rename"));
            }
            let bytes = self
                .files
                .borrow_mut()
                .remove(temporary)
                .expect("complete temporary image");
            self.files.borrow_mut().insert(target.into(), bytes);
            Ok(())
        }
    }
    fn captured_green() -> FrameCapture {
        FrameCapture {
            width: 1,
            height: 1,
            rgba: vec![0, 255, 0, 255],
            depth: vec![1.],
        }
    }
    #[test]
    fn publishes_owned_color_depth_and_image_without_partial_targets() {
        let store = FakeFrameImageStore::default();
        publish_capture(Path::new("frame"), &captured_green(), &store).unwrap();
        let files = store.files.borrow();
        assert_eq!(files.len(), 3);
        assert_eq!(files[Path::new("frame.rgba")], [0, 255, 0, 255]);
        assert_eq!(files[Path::new("frame.depth")], 1_f32.to_le_bytes());
        assert_eq!(
            files[Path::new("frame.ppm")],
            b"named fake serialized image"
        );
    }
    #[test]
    fn failure_and_malformed_capture_never_publish_success() {
        let store = FakeFrameImageStore {
            fail_write: true,
            ..Default::default()
        };
        let error = publish_capture(Path::new("frame"), &captured_green(), &store).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        let mut malformed = captured_green();
        malformed.rgba.clear();
        let error = publish_capture(Path::new("frame"), &malformed, &store).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(store.files.borrow().is_empty());
    }
    #[test]
    fn failed_rename_preserves_existing_published_bytes() {
        let store = FakeFrameImageStore {
            fail_rename: true,
            ..Default::default()
        };
        store
            .files
            .borrow_mut()
            .insert("frame.rgba".into(), vec![9; 4]);
        assert!(publish_capture(Path::new("frame"), &captured_green(), &store).is_err());
        assert_eq!(store.files.borrow()[Path::new("frame.rgba")], [9; 4]);
        assert!(!store.files.borrow().contains_key(Path::new("frame.ppm")));
    }
    #[test]
    fn published_image_storage_is_isolated_from_source_mutation() {
        let mut frame = captured_green();
        let image = owned_capture_image(&frame).unwrap();
        frame.rgba.fill(0);
        assert!(image.is_top_down());
        assert_eq!(image.pixel_color(0, 0, false).unwrap().rgb().green(), 1.);
    }
    #[test]
    fn extreme_extent_returns_invalid_data_without_publishing() {
        let store = FakeFrameImageStore::default();
        let frame = FrameCapture {
            width: u32::MAX,
            height: u32::MAX,
            rgba: Vec::new(),
            depth: Vec::new(),
        };
        let error = publish_capture(Path::new("extreme"), &frame, &store).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("extent=4294967295x4294967295"));
        assert!(
            error
                .to_string()
                .contains("representable usize RGBA byte length")
        );
        assert!(store.files.borrow().is_empty());
    }
    #[test]
    fn malformed_depth_reports_actual_lengths_and_expected_shape() {
        let store = FakeFrameImageStore::default();
        let mut frame = captured_green();
        frame.depth.clear();
        let error = publish_capture(Path::new("depth"), &frame, &store).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            error.to_string(),
            "RGBA bytes=4, depth values=0, extent=1x1; expected 4 RGBA bytes and 1 depth values for positive extent"
        );
        assert!(store.files.borrow().is_empty());
    }
}
