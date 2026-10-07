//! Graphic3d_ClipPlane behaviour that the numeric probe does not print: identifiers, the
//! refused chain loops and null aspects, the copy constructor and the dump.
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::aspect::AspectHatchStyle;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectFillArea3d, Graphic3dClipPlane, Graphic3dClipState, Graphic3dHatchStyle,
};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};

fn plane(a: f64, b: f64, c: f64, d: f64) -> Handle<Graphic3dClipPlane> {
    Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(a, b, c, d)).unwrap())
}

#[test]
fn identifiers_are_unique_and_prefixed() {
    let (first, second) = (Graphic3dClipPlane::default(), Graphic3dClipPlane::default());
    let id = String::from_utf8(first.get_id().as_bytes().to_vec()).unwrap();
    assert!(id.starts_with("Graphic3d_ClipPlane_"), "{id}");
    assert_ne!(first.get_id().as_bytes(), second.get_id().as_bytes());
}

#[test]
fn null_normal_raises_construction_error() {
    let error = Graphic3dClipPlane::from_equation(Vec4::new(0.0, 0.0, 0.0, 1.0))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn chain_loop_is_refused_without_changes() {
    let (a, b) = (plane(1.0, 0.0, 0.0, 0.0), plane(0.0, 1.0, 0.0, 0.0));
    Graphic3dClipPlane::set_chain_next_plane(&a, b.clone()).unwrap();
    assert_eq!(a.nb_chain_next_planes(), 2);
    let counter = b.m_count_equation();
    let error = Graphic3dClipPlane::set_chain_next_plane(&b, a.clone()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(Graphic3dClipPlane::set_chain_next_plane(&a, a.clone()).is_err());
    assert_eq!((b.m_count_equation(), b.is_chain()), (counter, false));
}

#[test]
fn chain_union_keeps_points_inside_any_plane() {
    let (a, b) = (plane(1.0, 0.0, 0.0, 0.0), plane(0.0, 1.0, 0.0, 0.0));
    Graphic3dClipPlane::set_chain_next_plane(&a, b.clone()).unwrap();
    let point = Vec4::new(-1.0, 1.0, 0.0, 1.0);
    assert_eq!(a.probe_point(&point), Graphic3dClipState::In);
    assert_eq!(a.probe_point_halfspace(&point), Graphic3dClipState::Out);
    assert_eq!(b.chain_previous_plane(), a);
    assert!(b.set_on(false).is_err());
}

#[test]
fn destroyed_predecessor_leaves_a_null_previous_plane() {
    let b = plane(0.0, 1.0, 0.0, 0.0);
    {
        let a = plane(1.0, 0.0, 0.0, 0.0);
        Graphic3dClipPlane::set_chain_next_plane(&a, b.clone()).unwrap();
    }
    assert!(b.chain_previous_plane().is_null());
}

#[test]
fn null_capping_aspect_is_refused() {
    let p = Graphic3dClipPlane::default();
    let error = p.set_capping_aspect(Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert_eq!(p.m_count_aspect(), 0);
    p.set_capping_aspect(Handle::new(Graphic3dAspectFillArea3d::default()))
        .unwrap();
    assert_eq!(p.m_count_aspect(), 1);
}

#[test]
fn capping_hatch_raises_without_a_predefined_style() {
    let p = Graphic3dClipPlane::default();
    assert_eq!(p.capping_hatch().unwrap(), AspectHatchStyle::Horizontal);
    p.set_capping_hatch(AspectHatchStyle::Solid);
    assert!(p.capping_hatch().is_err());
    let pattern = ImagePixMap::default();
    assert!(pattern.init_zero(ImageFormat::Gray, 32, 32, 0, 0));
    let custom = Graphic3dHatchStyle::from_pattern(&Handle::new(pattern)).unwrap();
    p.set_capping_custom_hatch(Handle::new(custom));
    let error = p.capping_hatch().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn copy_keeps_the_aspect_but_not_the_chain_or_counters() {
    let (a, b) = (plane(1.0, 0.0, 0.0, -2.0), plane(0.0, 1.0, 0.0, 0.0));
    Graphic3dClipPlane::set_chain_next_plane(&a, b).unwrap();
    a.set_capping_color(Color::named(NamedColor::RED)).unwrap();
    a.set_use_object_shader(true);
    let copy = a.clone_plane();
    assert_eq!(copy.get_equation(), a.get_equation());
    assert_eq!(copy.capping_color(), a.capping_color());
    assert!(copy.to_use_object_shader());
    assert!(!copy.is_chain());
    assert_eq!((copy.m_count_aspect(), copy.m_count_equation()), (0, 0));
    assert!(copy.capping_aspect() != a.capping_aspect());
}

#[test]
fn dump_lists_the_equations_and_counters() {
    let p = plane(0.0, 0.0, 1.0, -3.0);
    let mut stream = DumpStream::default();
    p.dump_json(&mut stream, -1).unwrap();
    let text = String::from_utf8(stream.bytes().to_vec()).unwrap();
    assert!(
        text.starts_with("\"className\": \"Graphic3d_ClipPlane\""),
        "{text}"
    );
    for key in [
        "\"Equation\": [0, 0, 1, -3]",
        "\"EquationRev\": [",
        "\"ChainLenFwd\": 1",
        "\"IsOn\": 1",
    ] {
        assert!(text.contains(key), "{key} missing from {text}");
    }
}
