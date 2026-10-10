//! Supplementary TopOpeBRepDS_BuildTool geometry construction tests.
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSBuildTool, TopOpeBRepDSCurve, TopOpeBRepDSHDataStructure, TopOpeBRepDSPoint,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};

#[test]
fn vertices_and_edges_are_made_from_ds_geometries() -> Result<(), Failure> {
    let bt = TopOpeBRepDSBuildTool::new();
    let mut v = Shape::default();
    bt.make_vertex(
        &mut v,
        &TopOpeBRepDSPoint::new(Point3::new(1., 2., 3.), 1e-5),
    )?;
    assert_eq!(
        BRepTool::pnt(&topods::vertex(&v)?)?,
        Point3::new(1., 2., 3.)
    );
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    let mut e = Shape::default();
    bt.make_edge(&mut e, &TopOpeBRepDSCurve::new(&line, 1e-4, false))?;
    assert_eq!(BRepTool::edge_tolerance(&topods::edge(&e)?)?, 1e-4);
    let mut degenerated = Shape::default();
    bt.make_edge(&mut degenerated, &TopOpeBRepDSCurve::default())?;
    assert!(BRepTool::degenerated(&topods::edge(&degenerated)?)?);
    assert!(
        bt.make_vertex(&mut e, &TopOpeBRepDSPoint::default())
            .is_err()
    );
    Ok(())
}

#[test]
fn the_approximation_follows_the_curve_type() {
    let mut bt = TopOpeBRepDSBuildTool::new();
    assert!(bt.approximation() && bt.over_write() && bt.translate());
    bt.change_geom_tool()
        .define_type(rustycad_tkbool::top_ope_brep_tool::TopOpeBRepToolOutCurveType::BSpline1);
    assert!(!bt.approximation());
}

#[test]
fn a_made_curve_copies_the_interferences_of_its_mother() -> Result<(), Failure> {
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let h = hds.get().ok_or_else(|| Failure::new("null"))?;
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    let mut c = TopOpeBRepDSCurve::new(&line, 1e-7, true);
    let i1 = h.change_ds().add_curve(&mut c);
    assert_eq!((i1, c.ds_index()), (1, 1));
    let mut new_c = TopOpeBRepDSCurve::default();
    let i2 = h.make_curve(&h.curve(i1), &mut new_c)?;
    assert_eq!((i2, new_c.mother(), h.nb_curves()), (2, 1, 2));
    let reversed = h.curve(i2).is_existing_edge_reversed();
    assert!(!reversed);
    let mut e = Shape::default();
    TopOpeBRepDSBuildTool::new().make_edge(&mut e, &h.curve(i2))?;
    assert_eq!(e.orientation(), Orientation::Forward);
    Ok(())
}
