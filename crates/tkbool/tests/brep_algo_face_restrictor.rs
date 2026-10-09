//! BRepAlgo_FaceRestrictor: faces of a plane bounded by wires, as given or with their
//! orientations and nesting corrected.
use rustycad_tkbool::brep_algo::BRepAlgoFaceRestrictor;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, BRepLibMakePolygon};

/// The square [x0, x1]^2 of the plane z = 0, counter-clockwise or clockwise.
fn square(x0: f64, x1: f64, ccw: bool) -> Result<Wire, Failure> {
    let mut points = vec![(x0, x0), (x1, x0), (x1, x1), (x0, x1)];
    if !ccw {
        points.reverse();
    }
    let mut mp = BRepLibMakePolygon::new();
    for (x, y) in points {
        mp.add_point(Point3::new(x, y, 0.))?;
    }
    mp.close()?;
    mp.wire()
}

/// The wire count of each face of the restrictor.
fn wire_counts(fr: &mut BRepAlgoFaceRestrictor) -> Result<Vec<usize>, Failure> {
    let mut counts = Vec::new();
    while fr.more() {
        let face: Face = fr.current()?;
        counts.push(Explorer::new(face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?.count());
        fr.next()?;
    }
    Ok(counts)
}

/// A restrictor on the face of the square [0, 10]^2 with `wires`.
fn restrictor(wires: &[Wire], control: bool) -> Result<BRepAlgoFaceRestrictor, Failure> {
    let face = BRepLibMakeFace::from_wire(&square(0., 10., true)?, true)?.face()?;
    let mut fr = BRepAlgoFaceRestrictor::new();
    fr.init(&face, false, control);
    for w in wires {
        fr.add(w);
    }
    fr.perform()?;
    Ok(fr)
}

#[test]
fn oriented_wires_bound_one_face_with_a_hole() -> Result<(), Failure> {
    let mut fr = restrictor(&[square(0., 10., true)?, square(3., 5., false)?], false)?;
    assert!(fr.is_done());
    assert_eq!(wire_counts(&mut fr)?, vec![2]);
    assert!(fr.current().is_err());
    Ok(())
}

#[test]
fn the_correction_turns_holes_and_nests_islands() -> Result<(), Failure> {
    let wires = [
        square(0., 10., true)?,
        square(3., 6., true)?,
        square(4., 5., false)?,
    ];
    let mut fr = restrictor(&wires, true)?;
    assert!(fr.is_done());
    assert_eq!(wire_counts(&mut fr)?, vec![2, 1]);
    fr.clear();
    assert!(!fr.more());
    Ok(())
}

#[test]
fn a_restrictor_is_not_done_before_perform() {
    assert!(!BRepAlgoFaceRestrictor::new().is_done());
}
