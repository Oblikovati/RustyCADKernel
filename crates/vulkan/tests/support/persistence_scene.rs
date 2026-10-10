//! Identical persistence scenes are driven through the native and Martensite viewer paths.
use Graphic3dTransModeFlags as Flags;
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, Xyz};
use rustycad_tkservice::{aspect::AspectTypeOfTriedronPosition as Corner, graphic3d::*};
pub const MODES: [&str; 19] = [
    "pers-zoom",
    "pers-rotate",
    "pers-zoom-rotate",
    "pers-axial",
    "pers-zoom-axial",
    "pers-2d",
    "pers-triedron",
    "pers-ortho",
    "pers-2d-ortho",
    "pers-triedron-ortho",
    "pers-camera",
    "pers-camera-near",
    "pers-camera-far",
    "pers-group-zoom",
    "pers-group-rotate",
    "pers-group-2d",
    "pers-nested",
    "pers-scaled-above",
    "pers-group-camera-near",
];
pub fn configure_persistence(
    structure: &Graphic3dStructure,
    group: &dyn Graphic3dGroup,
    vertices: &Graphic3dArrayOfPrimitives,
    camera: &Graphic3dCamera,
    mode: &str,
) {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a persistence scene"
    );
    camera
        .set_eye_and_center(&Point3::new(8., 6., 20.), &Point3::default())
        .unwrap();
    camera.set_axial_scale(Xyz::new(1.5, 0.75, 1.25)).unwrap();
    let persistence = scene_persistence(mode);
    if mode.starts_with("pers-group-") || mode == "pers-nested" {
        group.set_transform_persistence(&persistence);
        group.set_transformation(&Transform3::translation_vector(Vector3::new(1., 2., 0.)));
    } else {
        structure.set_transform_persistence(&persistence);
    }
    if mode == "pers-nested" {
        structure.set_transform_persistence(&corner_persistence(Flags::TMF_2D));
    }
    if mode.contains("camera") {
        set_camera_vertices(vertices, mode);
    }
}
pub fn scene_persistence(mode: &str) -> Handle<dyn Graphic3dTransformPersSource> {
    let flags = match mode {
        "pers-zoom" | "pers-group-zoom" | "pers-nested" => Flags::ZOOM_PERS,
        "pers-rotate" | "pers-group-rotate" => Flags::ROTATE_PERS,
        "pers-zoom-rotate" => Flags::ZOOM_ROTATE_PERS,
        "pers-axial" => Flags::AXIAL_SCALE_PERS,
        "pers-zoom-axial" => Flags::AXIAL_ZOOM_PERS,
        "pers-ortho" => Flags(Flags::ZOOM_PERS.0 | Flags::ORTHO_PERS.0),
        "pers-2d" | "pers-group-2d" => return corner_persistence(Flags::TMF_2D),
        "pers-triedron" => return corner_persistence(Flags::TRIEDRON_PERS),
        "pers-2d-ortho" => return corner_persistence(Flags(Flags::TMF_2D.0 | Flags::ORTHO_PERS.0)),
        "pers-triedron-ortho" => {
            return corner_persistence(Flags(Flags::TRIEDRON_PERS.0 | Flags::ORTHO_PERS.0));
        }
        "pers-scaled-above" => {
            return Handle::new(
                Graphic3dTransformPersScaledAbove::new(0.5, &Point3::new(1., -1., 0.)).unwrap(),
            )
            .cast();
        }
        _ => return Handle::new(Graphic3dTransformPers::new(Flags::CAMERA_PERS).unwrap()).cast(),
    };
    Handle::new(Graphic3dTransformPers::with_point(flags, &Point3::new(1., -1., 0.)).unwrap())
        .cast()
}
fn corner_persistence(flags: Flags) -> Handle<dyn Graphic3dTransformPersSource> {
    Handle::new(
        Graphic3dTransformPers::with_corner(
            flags,
            Corner(Corner::LEFT.0 | Corner::BOTTOM.0),
            Vec2::new(18, 18),
        )
        .unwrap(),
    )
    .cast()
}
fn set_camera_vertices(vertices: &Graphic3dArrayOfPrimitives, mode: &str) {
    let depth = if mode.ends_with("near") {
        -0.5
    } else if mode.ends_with("far") {
        -150.
    } else {
        -20.
    };
    for (index, [x, y]) in [[-8., -8.], [8., -8.], [0., 8.]].into_iter().enumerate() {
        vertices
            .set_vertice((index + 1) as i32, x, y, depth)
            .unwrap();
    }
}
