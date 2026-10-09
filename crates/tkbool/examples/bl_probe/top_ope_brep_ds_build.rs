//! TopOpeBRepDS_DataStructure, TopOpeBRepDS_HDataStructure and TopOpeBRepDS_BuildTool probe
//! operations, the Rust twin of tools/oracle/tkbool_probe_topopebrepds_build.cxx (the
//! records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::brep_fill_multi_line::{curve2d, curve3d};
use super::common::{Reader, attempt, end, flag, int, real};
use super::shapes::tp_shapes;
use rustycad_tkbool::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSBuildTool, TopOpeBRepDSConfig, TopOpeBRepDSCurve,
    TopOpeBRepDSCurvePointInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSPoint,
    TopOpeBRepDSSurfaceCurveInterference, TopOpeBRepDSTransition,
};
use rustycad_tkbool::top_ope_brep_tool::{TopOpeBRepToolCurveTool, TopOpeBRepToolOutCurveType};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Direction3, Point2, Point3};

/// The curve types in their native order.
const TYPES: [TopOpeBRepToolOutCurveType; 3] = [
    TopOpeBRepToolOutCurveType::BSpline1,
    TopOpeBRepToolOutCurveType::Approx,
    TopOpeBRepToolOutCurveType::Interpol,
];

/// `bl_ds_edge`.
fn edge(e: &Edge, f1: &Face, f2: &Face) {
    attempt(|| BRepTool::edge_tolerance(e).map(real));
    attempt(|| {
        let (a, b) = BRepTool::range(e)?;
        real(a);
        real(b);
        Ok(())
    });
    attempt(|| curve3d(&BRepTool::curve(e)?.0));
    for f in [f1, f2] {
        attempt(|| {
            let pc = BRepTool::curve_on_face(e, f)?;
            curve2d(&pc.pcurve)?;
            real(pc.first);
            real(pc.last);
            Ok(())
        });
    }
    let vertices: Vec<Shape> = Explorer::new(e.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)
        .map(|x| x.collect())
        .unwrap_or_default();
    for v in vertices {
        attempt(|| {
            let vertex = topods::vertex(&v)?;
            int(v.orientation() as i32);
            real(BRepTool::parameter(&vertex, e)?);
            real(BRepTool::vertex_tolerance(&vertex)?);
            Ok(())
        });
    }
}

/// `bl_ds_curve`.
fn ds_curve(c: &TopOpeBRepDSCurve) {
    attempt(|| curve3d(c.curve()));
    real(c.tolerance());
    flag(c.range().is_some());
    let (a, b) = c.range().unwrap_or((-1., -1.));
    real(a);
    real(b);
    attempt(|| curve2d(&c.curve1()?));
    attempt(|| curve2d(&c.curve2()?));
    int(c.mother());
    int(c.ds_index());
}

/// The 3d points and the UV on both faces of a polyline.
type Polyline = (
    Array1<'static, Point3>,
    Array1<'static, Point2>,
    Array1<'static, Point2>,
);

/// The polyline points of a request.
fn points(r: &mut Reader<'_>) -> Result<Polyline, Failure> {
    let n = r.integer();
    let err = |_| Failure::out_of_range("bl_probe: points");
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, n).map_err(err)?;
    let mut q1: Array1<'static, Point2> = Array1::with_bounds(1, n).map_err(err)?;
    let mut q2: Array1<'static, Point2> = Array1::with_bounds(1, n).map_err(err)?;
    for i in 1..=n {
        (p[i], q1[i], q2[i]) = (r.pnt(), r.pnt2d(), r.pnt2d());
    }
    Ok((p, q1, q2))
}

/// The data structure curve of the request, stored with its interferences; its index.
fn store_curve(
    h: &TopOpeBRepDSHDataStructure,
    (f1, f2): (&Face, &Face),
    walk: bool,
    curves: (
        Handle<dyn GeomCurve>,
        Handle<dyn Geom2dCurve>,
        Handle<dyn Geom2dCurve>,
    ),
) -> i32 {
    let mut ds = h.change_ds();
    let (i1, i2) = (
        ds.add_shape_with_rank(f1.as_ref(), 1),
        ds.add_shape_with_rank(f2.as_ref(), 2),
    );
    let mut dc = TopOpeBRepDSCurve::new(&curves.0, 1e-7, walk);
    dc.set_shapes(f1.as_ref(), f2.as_ref());
    let t = TopOpeBRepDSTransition::default();
    let s1: Handle<TopOpeBRepDSInterference> =
        Handle::new(TopOpeBRepDSSurfaceCurveInterference::new(
            &t,
            TopOpeBRepDSKind::Face,
            i1,
            TopOpeBRepDSKind::Curve,
            0,
            &curves.1,
        ))
        .cast();
    let s2: Handle<TopOpeBRepDSInterference> =
        Handle::new(TopOpeBRepDSSurfaceCurveInterference::new(
            &t,
            TopOpeBRepDSKind::Face,
            i2,
            TopOpeBRepDSKind::Curve,
            0,
            &curves.2,
        ))
        .cast();
    dc.set_sci(&s1, &s2);
    let ic = ds.add_curve(&mut dc);
    for s in [&s1, &s2] {
        if let Some(s) = s.get() {
            s.set_geometry(ic);
        }
    }
    ds.add_shape_interference(f1.as_ref(), &s1);
    ds.add_shape_interference(f2.as_ref(), &s2);
    ic
}

/// The edge of the curve with its vertices at the parameters.
fn make_edge(
    bt: &TopOpeBRepDSBuildTool,
    dc: &TopOpeBRepDSCurve,
    (p1, p2): (f64, f64),
) -> Result<Shape, Failure> {
    let mut e = Shape::default();
    bt.make_edge(&mut e, dc)?;
    let c = dc.curve().get().ok_or_else(|| Failure::new("null curve"))?;
    for (k, par) in [p1, p2].into_iter().enumerate() {
        let mut v = Shape::default();
        bt.make_vertex(&mut v, &TopOpeBRepDSPoint::new(c.value(par)?, 1e-7))?;
        v.set_orientation(if k == 0 {
            Orientation::Forward
        } else {
            Orientation::Reversed
        });
        bt.add_edge_vertex(&mut e, &v)?;
        bt.parameter(&e, &v, par)?;
    }
    Ok(e)
}

/// `ds_recompute program f1 f2 walk type c3d pc1 pc2 p1 p2 n (x y z u1 v1 u2 v2)...`.
pub fn recompute_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (fi1, fi2) = (r.integer(), r.integer());
    let (walk, ty) = (r.flag(), TYPES[r.integer() as usize]);
    let (c3d, pc1, pc2) = (r.flag(), r.flag(), r.flag());
    let pars = (r.real(), r.real());
    let setup = (|| {
        let (f1, f2) = (topods::face(&at(&t, fi1)?)?, topods::face(&at(&t, fi2)?)?);
        let (p, q1, q2) = points(&mut r)?;
        let curves = if walk {
            (
                TopOpeBRepToolCurveTool::make_bspline1_from_pnt(&p)?,
                TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&q1)?,
                TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&q2)?,
            )
        } else {
            let circle = GeomCircle::new(Axis2::new(Point3::new(0., 0., 1.), Direction3::Z)?, 1.)?;
            (
                Handle::new(circle).cast(),
                Handle::default(),
                Handle::default(),
            )
        };
        Ok::<_, Failure>((f1, f2, curves))
    })();
    let (f1, f2, curves) = match setup {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let Some(h) = hds.get() else { return };
    let ic = store_curve(h, (&f1, &f2), walk, curves);
    let mut bt = TopOpeBRepDSBuildTool::with_type(ty);
    bt.change_geom_tool().define(ty, c3d, pc1, pc2);
    let e = match make_edge(&bt, &h.curve(ic), pars).and_then(|e| topods::edge(&e)) {
        Ok(e) => e,
        Err(error) => return failed(error),
    };
    let mut ee = e.clone();
    let inew = match bt.recompute_curves(&h.curve(ic), &e, &mut ee, &hds) {
        Ok(i) => i,
        Err(error) => {
            print!("{} ", error.exception_type());
            -1
        }
    };
    report(h, (inew, ic), (&ee, &f1, &f2));
}

/// The records after RecomputeCurves.
fn report(
    h: &TopOpeBRepDSHDataStructure,
    (inew, ic): (i32, i32),
    (ee, f1, f2): (&Edge, &Face, &Face),
) {
    // A native RecomputeCurves without new curve leaves inewC untouched (-1).
    int(if inew == 0 { -1 } else { inew });
    int(h.nb_curves());
    if inew > 0 {
        ds_curve(&h.curve(inew));
    }
    ds_curve(&h.curve(ic));
    edge(ee, f1, f2);
    let ds = h.ds();
    int(ds.shape_interferences(f1.as_ref(), true).len() as i32);
    int(ds.shape_interferences(f2.as_ref(), true).len() as i32);
    end();
}

/// The configurations in their native order.
const CONFIGS: [TopOpeBRepDSConfig; 3] = [
    TopOpeBRepDSConfig::UnshGeometry,
    TopOpeBRepDSConfig::SameOriented,
    TopOpeBRepDSConfig::DiffOriented,
];

/// `ds_same_domain program s1 s2 refFirst c1 c2 mode`.
pub fn same_domain_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let shapes = (|| Ok::<_, Failure>((at(&t, r.integer())?, at(&t, r.integer())?)))();
    let (s1, s2) = match shapes {
        Ok(v) => v,
        Err(error) => return failed(error),
    };
    let reff = r.flag();
    let (c1, c2) = (CONFIGS[r.integer() as usize], CONFIGS[r.integer() as usize]);
    let mode = r.integer();
    let mut ds = TopOpeBRepDSDataStructure::new();
    for k in 0..2 {
        let (a, b) = if k == 0 { (&s1, &s2) } else { (&s2, &s1) };
        attempt(|| {
            if mode == 1 {
                ds.fill_shapes_same_domain_with_configs(a, b, c1, c2, reff)
            } else {
                ds.fill_shapes_same_domain(a, b, reff)
            }
        });
        for s in [&s1, &s2] {
            int(ds.shape_index(s, true));
            int(ds.same_domain_ref(s));
            int(ds.same_domain_ori(s) as i32);
            int(ds.same_domain_ind(s));
            int(ds.ancestor_rank(s));
            int(ds.shape_same_domain(s).len() as i32);
        }
    }
    attempt(|| {
        ds.unfill_shapes_same_domain(&s1, &s2)?;
        int(ds.shape_same_domain(&s1).len() as i32);
        int(ds.shape_same_domain(&s2).len() as i32);
        Ok(())
    });
    end();
}

/// `ds_sort n (param orientation)...`.
pub fn sort_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut l = ListOfInterference::default();
    for i in 0..n {
        let par = r.real();
        let o = Orientation::ALL[r.integer() as usize];
        let t = TopOpeBRepDSTransition::from_orientation(o);
        let it = TopOpeBRepDSCurvePointInterference::new(
            &t,
            TopOpeBRepDSKind::Curve,
            1,
            TopOpeBRepDSKind::Point,
            i + 1,
            par,
        );
        l.append(Handle::new(it).cast());
    }
    let hds = TopOpeBRepDSHDataStructure::new();
    attempt(|| {
        let mut s = ListOfInterference::default();
        hds.sort_on_parameter(&l, &mut s)?;
        int(s.len() as i32);
        for i in s.iter() {
            int(i.get().map_or(0, |i| i.geometry()));
        }
        Ok(())
    });
    attempt(|| {
        let (a, b) = hds.min_max_on_parameter(&l)?.unwrap_or((-1., -1.));
        real(a);
        real(b);
        Ok(())
    });
    end();
}
