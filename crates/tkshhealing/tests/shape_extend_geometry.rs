//! Behavior tests of ShapeExtend_CompositeSurface and ShapeExtend_ComplexCurve.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray2};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomGeometry, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSurface, dump_curve,
};
use rustycad_tkmath::gp::{Axis3, Point2, Point3, Transform3, Vector3};
use rustycad_tkshhealing::shape_extend::{
    ShapeExtendComplexCurve, ShapeExtendCompositeSurface, ShapeExtendParametrisation, SurfaceGrid,
};
use std::io;

/// A trimmed patch `[u1, u2] x [v1, v2]` of the plane z = 0.
fn patch(u: (f64, f64), v: (f64, f64)) -> Handle<dyn GeomSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Handle::new(GeomRectangularTrimmedSurface::new(&plane, u, v, (true, true)).unwrap()).cast()
}

/// Two patches in U: `[0, 1] x [0, 1]` and `[1, 3] x [0, 1]`.
fn row_grid() -> SurfaceGrid {
    let grid = HArray2::with_value(1, 2, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
    grid.change_array2()
        .set_value(1, 1, patch((0.0, 1.0), (0.0, 1.0)))
        .unwrap();
    grid.change_array2()
        .set_value(2, 1, patch((1.0, 3.0), (0.0, 1.0)))
        .unwrap();
    Handle::new(grid)
}

#[test]
fn natural_joints_add_the_patch_ranges_and_points_go_to_their_patch() {
    let surface =
        ShapeExtendCompositeSurface::with_grid(&row_grid(), ShapeExtendParametrisation::Natural)
            .unwrap();
    assert_eq!(surface.bounds(), (0.0, 3.0, 0.0, 1.0));
    assert_eq!(
        surface.locate_uv_point(Point2::new(1.5, 0.5)).unwrap(),
        (2, 1)
    );
    let p = surface.value(2.0, 0.25).unwrap();
    assert_eq!((p.x(), p.y()), (2.0, 0.25));
    assert!(!surface.is_u_closed().unwrap());
}

#[test]
fn unitary_joints_map_the_patches_onto_the_unit_square() {
    let surface =
        ShapeExtendCompositeSurface::with_grid(&row_grid(), ShapeExtendParametrisation::Unitary)
            .unwrap();
    assert_eq!(surface.bounds(), (0.0, 1.0, 0.0, 1.0));
    assert_eq!(surface.u_global_to_local(2, 1, 0.75).unwrap(), 2.0);
    assert_eq!(surface.u_local_to_global(2, 1, 2.0).unwrap(), 0.75);
    let (changed, u_fact, _) = surface.global_to_local_transformation(2, 1).unwrap();
    assert!(changed);
    assert_eq!(u_fact, 4.0);
}

#[test]
fn rejected_joints_fall_back_to_the_natural_ones() {
    let grid = row_grid();
    let mut u = Array1::with_bounds(1, 3).unwrap();
    [0.0, 0.0, 1.0]
        .iter()
        .enumerate()
        .for_each(|(i, &x)| u.set_value(i as i32 + 1, x).unwrap());
    let mut v = Array1::with_bounds(1, 2).unwrap();
    v.set_value(1, 0.0).unwrap();
    v.set_value(2, 1.0).unwrap();
    let surface = ShapeExtendCompositeSurface::new();
    assert!(!surface.init_with_joints(&grid, &u, &v).unwrap());
    assert_eq!(surface.u_joint_value(3).unwrap(), 3.0);
}

#[test]
fn copy_recomputes_natural_joints_and_an_empty_surface_reports_null_objects() {
    let surface =
        ShapeExtendCompositeSurface::with_grid(&row_grid(), ShapeExtendParametrisation::Uniform)
            .unwrap();
    surface.set_u_first_value(5.0).unwrap();
    let copy = surface
        .copy()
        .unwrap()
        .cast::<ShapeExtendCompositeSurface>();
    assert_eq!(copy.u_joint_value(1).unwrap(), 0.0);
    assert_eq!(surface.u_joint_value(1).unwrap(), 5.0);
    let empty = ShapeExtendCompositeSurface::new();
    assert_eq!(
        empty.nb_u_patches().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(empty.bounds(), (0.0, 0.0, 0.0, 0.0));
}

/// A test complex curve: segment i of n covers `[(i - 1) / n, i / n]`.
struct Segments {
    parent: StandardTransient,
    curves: Vec<Handle<dyn GeomCurve>>,
    closed: SharedState<bool>,
}
impl_transient!(Segments, "Segments", dyn ShapeExtendComplexCurve, parent, interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn ShapeExtendComplexCurve]);

impl GeomGeometry for Segments {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.complex_transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Err(Failure::new("not needed by the tests"))
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_curve(stream, depth)
    }
}

impl GeomCurve for Segments {
    fn reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    rustycad_tkshhealing::shape_extend_complex_curve_members!();
}

impl ShapeExtendComplexCurve for Segments {
    fn nb_curves(&self) -> i32 {
        self.curves.len() as i32
    }
    fn curve(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.curves[(index - 1) as usize].clone())
    }
    fn locate_parameter(&self, u: f64) -> Result<(i32, f64), Failure> {
        let n = self.nb_curves();
        let i = ((u * f64::from(n)).floor() as i32 + 1).clamp(1, n);
        Ok((i, u * f64::from(n) - f64::from(i - 1)))
    }
    fn local_to_global(&self, index: i32, u_local: f64) -> Result<f64, Failure> {
        Ok((f64::from(index - 1) + u_local) / f64::from(self.nb_curves()))
    }
    fn get_scale_factor(&self, _ind: i32) -> Result<f64, Failure> {
        Ok(f64::from(self.nb_curves()))
    }
    fn complex_closed(&self) -> bool {
        *self.closed.read()
    }
    fn set_complex_closed(&self, closed: bool) {
        *self.closed.write() = closed;
    }
}

fn bezier(points: &[(f64, f64)]) -> Handle<dyn GeomCurve> {
    let mut poles = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, &(x, y)) in points.iter().enumerate() {
        poles
            .set_value(i as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast()
}

fn segments(curves: Vec<Handle<dyn GeomCurve>>) -> Segments {
    Segments {
        parent: StandardTransient::default(),
        curves,
        closed: SharedState::new(false),
    }
}

#[test]
fn a_complex_curve_scales_its_derivatives_and_checks_its_joints() {
    let curve = segments(vec![
        bezier(&[(0.0, 0.0), (1.0, 0.0)]),
        bezier(&[(1.0, 0.0), (0.0, 0.0)]),
    ]);
    assert!(curve.check_connectivity(1e-7).unwrap());
    assert!(curve.is_closed().unwrap());
    let d1 = curve.eval_d1(0.25).unwrap();
    assert_eq!((d1.point.x(), d1.d1.x()), (0.5, 2.0));
    assert_eq!(curve.reversed_parameter(0.25).unwrap(), 0.75);
    assert!(!curve.is_cn(1).unwrap());
}

#[test]
fn one_segment_never_sets_the_closed_flag() {
    // The flag is written inside the loop over the joints, which one segment skips.
    let curve = segments(vec![bezier(&[(0.0, 0.0), (1.0, 1.0), (0.0, 0.0)])]);
    assert!(curve.check_connectivity(1e-7).unwrap());
    assert!(!curve.is_closed().unwrap());
    curve
        .transform({
            let mut t = Transform3::default();
            t.set_translation(Vector3::new(0.0, 0.0, 1.0));
            t
        })
        .unwrap();
    assert_eq!(curve.value(0.0).unwrap().z(), 1.0);
}
