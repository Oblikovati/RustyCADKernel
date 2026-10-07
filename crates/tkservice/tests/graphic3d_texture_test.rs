//! Texture classes of Graphic3d: root, map, 1D, 2D, plane, environment and set.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkservice::image::{ImageFormat, ImagePixMap, ImageSupportedFormats};

fn pixmap(format: ImageFormat) -> Handle<ImagePixMap> {
    let image = ImagePixMap::default();
    image.init_zero(format, 2, 2, 0, 0);
    Handle::new(image)
}

#[test]
fn texture_2d_constructors_apply_the_public_defaults() {
    let texture = Graphic3dTexture2D::from_pix_map(pixmap(ImageFormat::Rgba));
    assert!(texture.is_done());
    assert!(
        texture.has_mipmaps()
            && texture.is_modulate()
            && texture.is_repeat()
            && texture.is_smoothed()
    );
    assert_eq!(texture.texture_type(), Graphic3dTypeOfTexture::TOT_2D);
    assert_eq!(texture.name(), Graphic3dNameOfTexture2D::Unknown);
    assert!(texture.get_id().starts_with("Graphic3d_TextureRoot_"));
    assert_eq!(texture.get_params().sampler_revision(), 2);
}

#[test]
fn file_textures_are_not_done_without_the_file() {
    let texture = Graphic3dTexture2D::from_file(&AsciiString::from("/nonexistent/wood.png"));
    assert!(!texture.is_done());
    let supported = Handle::new(ImageSupportedFormats::default());
    assert!(texture.get_image(&supported).is_null());
    assert!(texture.get_compressed_image(&supported).is_null());
}

#[test]
fn get_image_returns_the_pixmap_and_its_row_order() {
    let image = pixmap(ImageFormat::Bgra);
    image.set_top_down(false);
    let texture = Graphic3dTexture2D::from_pix_map(image.clone());
    let supported = Handle::new(ImageSupportedFormats::default());
    assert!(texture.get_image(&supported) == image);
    assert!(!texture.is_top_down());
    texture.set_image(Handle::default());
    assert!(!texture.is_done());
}

#[test]
fn convert_to_compatible_swaps_unsupported_bgr_images() {
    let image = pixmap(ImageFormat::Bgr32);
    let supported = Handle::new(ImageSupportedFormats::default());
    graphic3d_convert_to_compatible(&supported, &image);
    assert_eq!(image.format(), ImageFormat::Rgb32);
    supported.add(ImageFormat::Rgb32);
    let other = pixmap(ImageFormat::Rgb32);
    graphic3d_convert_to_compatible(&supported, &other);
    assert_eq!(other.format(), ImageFormat::Rgb32);
}

#[test]
fn predefined_texture_names_strip_prefix_and_extension() {
    assert_eq!(Graphic3dTexture2DBase::number_of_textures(), 22);
    assert_eq!(
        Graphic3dTexture2DBase::texture_name(1).unwrap().as_bytes(),
        b"MatraDatavision"
    );
    assert_eq!(
        Graphic3dTexture2DBase::texture_name(22).unwrap().as_bytes(),
        b"chess"
    );
    assert_eq!(
        Graphic3dTexture1DBase::texture_name(1).unwrap().as_bytes(),
        b"elevation"
    );
    assert_eq!(
        Graphic3dTextureEnv::texture_name(5).unwrap().as_bytes(),
        b"sky1"
    );
    let error = Graphic3dTextureEnv::texture_name(9).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn unknown_predefined_names_are_refused() {
    let error = Graphic3dTexture2D::from_name(Graphic3dNameOfTexture2D::Unknown)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn segment_textures_generate_along_the_segment() {
    let texture = Graphic3dTexture1Dsegment::from_pix_map(pixmap(ImageFormat::Rgb));
    assert_eq!(texture.texture_type(), Graphic3dTypeOfTexture::TOT_1D);
    texture.set_segment(0.0, 0.0, 1.0, 0.0, 0.0, 3.0);
    let params = texture.get_params();
    assert_eq!(params.gen_mode(), Graphic3dTypeOfTextureMode::Object);
    assert_eq!(params.gen_plane_s().z, 0.5);
    assert_eq!(params.gen_plane_s().w, -0.5);
    assert_eq!(texture.segment(), [0.0, 0.0, 1.0, 0.0, 0.0, 3.0]);
}

#[test]
fn plane_textures_track_the_plane_name() {
    let texture = Graphic3dTexture2Dplane::from_pix_map(pixmap(ImageFormat::Rgb));
    assert_eq!(texture.plane(), Graphic3dNameOfTexturePlane::Unknown);
    texture.set_plane(Graphic3dNameOfTexturePlane::Yz);
    assert_eq!(texture.plane_s(), [0.0, 1.0, 0.0, 0.0]);
    texture.set_plane_t(1.0, 2.0, 3.0, 4.0);
    assert_eq!(texture.plane(), Graphic3dNameOfTexturePlane::Unknown);
    texture.set_scale_t(3.0);
    texture.set_translate_s(0.25);
    assert_eq!(
        (texture.scale_s(), texture.scale_t(), texture.translate_s()),
        (1.0, 3.0, 0.25)
    );
}

#[test]
fn environment_textures_use_sphere_coordinates() {
    let texture = Graphic3dTextureEnv::from_pix_map(pixmap(ImageFormat::Rgb));
    assert_eq!(
        texture.get_params().gen_mode(),
        Graphic3dTypeOfTextureMode::Sphere
    );
    assert!(texture.has_mipmaps());
    assert_eq!(texture.name(), Graphic3dNameOfTextureEnv::Unknown);
}

#[test]
fn texture_sets_index_from_zero() {
    let texture: Handle<dyn Graphic3dTextureMap> =
        Handle::new(Graphic3dTexture2D::from_pix_map(pixmap(ImageFormat::Rgb))).cast();
    let set = Handle::new(Graphic3dTextureSet::with_size(2).unwrap());
    assert_eq!((set.lower(), set.upper(), set.size()), (0, 1, 2));
    set.set_value(1, texture.clone()).unwrap();
    assert!(set.value(1).unwrap() == texture);
    assert!(set.first().unwrap().is_null());
    assert_eq!(
        set.value(2).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(Graphic3dTextureSet::iter(&set).count(), 2);
    assert!(Graphic3dTextureSet::with_size(-1).is_err());
    assert!(Graphic3dTextureSet::default().is_empty());
}

#[test]
fn revisions_and_ids_are_per_texture() {
    let a = Graphic3dTexture1Dmanual::from_pix_map(pixmap(ImageFormat::Rgb));
    let b = Graphic3dTexture1Dmanual::from_file(&AsciiString::from("ramp.png"));
    assert_ne!(a.get_id(), b.get_id());
    a.update_revision();
    assert_eq!((a.revision(), b.revision()), (1, 0));
    assert_eq!(b.path().system_name().as_bytes(), b"ramp.png");
}

#[test]
fn separate_cube_maps_keep_matching_images_only() {
    let images: Vec<Handle<ImagePixMap>> = (0..6).map(|_| pixmap(ImageFormat::Rgb)).collect();
    let cube = Graphic3dCubeMapSeparate::from_images(&images).unwrap();
    assert!(cube.is_done());
    let supported = Handle::new(ImageSupportedFormats::default());
    assert!(cube.value(&supported) == images[0]);
    assert!(cube.get_image(&supported).is_null());
    let mut mixed = images.clone();
    mixed[3] = pixmap(ImageFormat::Rgba);
    assert!(
        Graphic3dCubeMapSeparate::from_images(&mixed)
            .unwrap()
            .images()[0]
            .is_null()
    );
    assert!(Graphic3dCubeMapSeparate::from_images(&images[..5]).is_err());
}

#[test]
fn cube_map_iteration_marks_the_end_after_neg_z() {
    let paths: Vec<AsciiString> = (0..6)
        .map(|i| AsciiString::from(format!("/nonexistent/{i}.png").as_str()))
        .collect();
    let cube = Graphic3dCubeMapSeparate::from_paths(&paths).unwrap();
    assert!(!cube.is_done());
    let mut count = 0;
    while cube.more() {
        count += 1;
        cube.next();
    }
    assert_eq!(
        (count, cube.current_side()),
        (6, Some(Graphic3dCubeMapSide::NegZ))
    );
    cube.next();
    assert_eq!(cube.current_side(), None);
    cube.reset();
    assert_eq!(cube.current_side(), Some(Graphic3dCubeMapSide::PosX));
    assert_eq!(cube.texture_type(), Graphic3dTypeOfTexture::Cubemap);
}

#[test]
fn texture_3d_slices_fail_without_an_image_library() {
    let files = [AsciiString::from("s0.png"), AsciiString::from("s1.png")];
    let texture = Graphic3dTexture3D::from_slices(&files);
    let supported = Handle::new(ImageSupportedFormats::default());
    assert!(texture.get_image(&supported).is_null());
    let volume = pixmap(ImageFormat::Gray);
    texture.set_image(volume.clone());
    assert!(texture.slice_paths().is_empty());
    assert!(texture.get_image(&supported) == volume);
    assert!(texture.is_smoothed());
}
