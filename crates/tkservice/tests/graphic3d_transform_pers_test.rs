//! Graphic3d_TransformPers behaviour that the numeric probe does not print: the union dump,
//! the run-time types and the scaled-above lower bound.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectTypeOfTriedronPosition as Corner;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dTransModeFlags as Flags, Graphic3dTransformPers,
    Graphic3dTransformPersMembers, Graphic3dTransformPersScaledAbove, Graphic3dTransformPersSource,
};

fn dump(pers: &dyn Graphic3dTransformPersSource) -> String {
    let mut stream = DumpStream::default();
    pers.dump_json(&mut stream, -1).unwrap();
    String::from_utf8(stream.bytes().to_vec()).unwrap()
}

#[test]
fn dump_reads_both_union_views() {
    let pers =
        Graphic3dTransformPers::with_corner(Flags::TMF_2D, Corner::RIGHT_UPPER, Vec2::new(3, 4))
            .unwrap();
    let text = dump(&pers);
    for key in [
        "\"Mode\": 64",
        "\"OffsetX\": 3",
        "\"OffsetY\": 4",
        "\"Corner\": 9",
        "\"AttachPoint\":",
    ] {
        assert!(text.contains(key), "{key} missing from {text}");
    }
}

#[test]
fn wrong_modes_raise_program_errors() {
    for mode in [Flags::NONE, Flags::ORTHO_PERS] {
        let error = Graphic3dTransformPers::new(mode).err().unwrap();
        assert_eq!(error.exception_type(), "Standard_ProgramError");
    }
    let pers = Graphic3dTransformPers::new(Flags::CAMERA_PERS).unwrap();
    assert!(pers.anchor_point().is_err() && pers.corner_2d().is_err());
    let zoom = Graphic3dTransformPers::new(Flags::ZOOM_PERS).unwrap();
    zoom.set_anchor_point(&Point3::new(1.0, 2.0, 3.0)).unwrap();
    assert_eq!(zoom.anchor_point().unwrap().z(), 3.0);
    assert!(zoom.set_offset_2d(Vec2::new(1, 1)).is_err());
}

#[test]
fn scaled_above_keeps_its_minimal_scale() {
    let pers = Graphic3dTransformPersScaledAbove::new(1.0e6, &Point3::new(0.0, 0.0, 0.0)).unwrap();
    let camera = Graphic3dCamera::default();
    assert_eq!(pers.persistent_scale(&camera, 800, 600), 1.0e6);
    let handle: Handle<dyn Graphic3dTransformPersSource> = Handle::new(pers).cast();
    assert!(handle.is_kind_name("Graphic3d_TransformPers"));
    assert_eq!(handle.mode(), Flags::ZOOM_PERS);
}
