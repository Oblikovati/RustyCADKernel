//! Graphic3d_MarkerImage: construction checks and the standard markers; the bytes are compared
//! with the native class by the sv_marker probe.
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tkservice::graphic3d::Graphic3dMarkerImage;
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};

fn image(format: ImageFormat, size: usize) -> Handle<ImagePixMap> {
    let pixmap = ImagePixMap::default();
    pixmap.init_zero(format, size, size, 0, 0);
    Handle::new(pixmap)
}

#[test]
fn alpha_images_must_match_the_image() {
    let error =
        Graphic3dMarkerImage::from_images(image(ImageFormat::Rgba, 4), image(ImageFormat::Rgba, 4))
            .err()
            .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    let error = Graphic3dMarkerImage::from_images(
        image(ImageFormat::Rgba, 4),
        image(ImageFormat::Alpha, 3),
    )
    .err()
    .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    let marker =
        Graphic3dMarkerImage::from_images(image(ImageFormat::Rgba, 4), image(ImageFormat::Gray, 4))
            .unwrap();
    assert!(marker.is_colored_image());
    assert_eq!(marker.get_texture_size(), (4, 4));
    assert!(Graphic3dMarkerImage::from_images(Handle::default(), Handle::default()).is_err());
}

#[test]
fn standard_markers_use_type_and_scale_keys() {
    let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
    let plus = Graphic3dMarkerImage::standard_marker(AspectTypeOfMarker::Plus, 3.0, white).unwrap();
    assert_eq!(
        plus.get_image_id().as_bytes(),
        b"Graphic3d_MarkerImage_+_30"
    );
    assert!(!plus.is_colored_image());
    let ball = Graphic3dMarkerImage::standard_marker(AspectTypeOfMarker::Ball, 2.0, white).unwrap();
    assert_eq!(
        ball.get_image_id().as_bytes(),
        b"Graphic3d_MarkerImage_ball_20FFFFFF"
    );
    assert!(ball.is_colored_image());
    assert!(
        Graphic3dMarkerImage::standard_marker(AspectTypeOfMarker::Userdefined, 1.0, white)
            .unwrap()
            .is_null()
    );
    assert!(Graphic3dMarkerImage::standard_marker(AspectTypeOfMarker::Ring1, 0.5, white).is_err());
}

#[test]
fn bitmaps_become_alpha_images_with_a_margin() {
    let marker = Graphic3dMarkerImage::standard_marker(
        AspectTypeOfMarker::Point,
        1.0,
        Vec4::new(1.0, 1.0, 1.0, 1.0),
    )
    .unwrap();
    let image = marker.get_image();
    assert_eq!(image.format(), ImageFormat::Alpha);
    assert!(marker.get_image_alpha() == image);
}
