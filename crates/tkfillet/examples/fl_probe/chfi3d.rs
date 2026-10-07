//! The ChFi3d operations that run without the TKBool data structure
//! (`fl_hatching_domains`, `fl_concave_side`, `fl_search_sing`); the twin of
//! tools/oracle/tkfillet_probe_chfi3d.cxx.
use super::blend_func::geom_guide;
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi3d::{ChFi3dSearchSing, compute_hatching_domains, concave_side};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tkmath::gp::{Axis2d, Point2};
use rustycad_tkmath::math::{Function, FunctionRoot, FunctionWithDerivative};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeRevol, BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

/// The hatcher with the closed polygon of the points as elements.
fn polygon_hatcher(points: &[Point2], tol: f64, keep: bool) -> Result<Hatcher, Failure> {
    let mut hatcher = Hatcher::new(&Intersector::new(tol, tol), tol, tol, false, keep);
    for (i, p) in points.iter().enumerate() {
        let next = points[(i + 1) % points.len()];
        let segment: Handle<dyn Geom2dCurve> =
            MakeSegment2d::from_points(*p, next)?.value()?.cast();
        hatcher.add_element_curve(&segment, Orientation::Forward)?;
    }
    Ok(hatcher)
}

/// The points of the hatching (parameter, segment flags), then its domains.
fn hatching_state(hatcher: &Hatcher, ih: i32) -> Result<(), Failure> {
    int(hatcher.nb_points(ih)?);
    for i in 1..=hatcher.nb_points(ih)? {
        let point = hatcher.point(ih, i)?;
        real(point.parameter());
        flag(point.segment_beginning());
        flag(point.segment_end());
    }
    attempt(|| {
        int(hatcher.nb_domains(ih)?);
        for i in 1..=hatcher.nb_domains(ih)? {
            let domain = hatcher.domain(ih, i)?;
            flag(domain.has_first_point());
            if domain.has_first_point() {
                real(domain.first_point()?.parameter());
            }
            flag(domain.has_second_point());
            if domain.has_second_point() {
                real(domain.second_point()?.parameter());
            }
        }
        Ok(())
    });
    Ok(())
}

/// `fl_hatching_domains`: see tools/oracle/tkfillet_probe_chfi3d.cxx.
pub fn hatching_domains_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let n = r.integer();
    let points: Vec<Point2> = (0..n).map(|_| r.pnt2d()).collect();
    let (p, d, tol, keep) = (r.pnt2d(), r.dir2d(), r.real(), r.flag());
    let mut hatcher = polygon_hatcher(&points, tol, keep).expect("polygon elements");
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(p, d))).cast();
    let adaptor = Geom2dAdaptorCurve::with_curve(&line).expect("hatching adaptor");
    let ih = hatcher.trim_curve(adaptor).expect("trimmed hatching");
    attempt(|| {
        flag(compute_hatching_domains(&mut hatcher, ih)?);
        Ok(())
    });
    hatching_state(&hatcher, ih).expect("hatching state");
    end();
}

/// `fl_polygon_face`.
fn polygon_face(points: &[(f64, f64)]) -> Result<Face, Failure> {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for &(x, y) in points {
        poly.add_point(Point3::new(x, y, 0.0))?;
    }
    poly.close()?;
    BRepBuilderAPIMakeFace::from_wire(&poly.wire()?, false)?.face()
}

/// The shape of a prim maker.
fn built(mut maker: impl BRepBuilderAPIMakeShape) -> Result<Shape, Failure> {
    Ok(maker.shape()?.clone())
}

/// `fl_solid`: see tools/oracle/tkfillet_probe_chfi3d.cxx.
pub(super) fn solid(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let kind = r.integer();
    let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
    match kind {
        0 => built(BRepPrimAPIMakeBox::from_sizes(a, b, c)?),
        1 => {
            let f = polygon_face(&[(0.0, 0.0), (a, 0.0), (a, d), (d, d), (d, b), (0.0, b)])?;
            built(BRepPrimAPIMakePrism::new(
                &f.into_shape(),
                Vector3::new(0.0, 0.0, c),
                false,
                true,
            )?)
        }
        2 => built(BRepPrimAPIMakeCylinder::new(a, b)),
        3 => built(BRepPrimAPIMakeCone::new(a, b, c)?),
        4 => built(BRepPrimAPIMakeSphere::new(a)?),
        5 => built(BRepPrimAPIMakeTorus::new(a, b)?),
        _ => {
            let f = polygon_face(&[(a, 0.0), (a + b, 0.0), (a + b, c), (a, c)])?;
            let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 1.0, 0.0)?);
            built(BRepPrimAPIMakeRevol::new(&f.into_shape(), axis, d, false)?)
        }
    }
}

/// `fl_edge_faces`.
pub(super) fn edge_faces(s: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut m)?;
    Ok(m)
}

/// `fl_concave`.
fn concave(f1: &Face, f2: &Face, e: &Edge) {
    attempt(|| {
        let (s1, s2) = (
            BRepAdaptorSurface::with_face(f1, true)?,
            BRepAdaptorSurface::with_face(f2, true)?,
        );
        let (choice, o1, o2) = concave_side(&s1, &s2, e)?;
        int(choice);
        int(o1 as i32);
        int(o2 as i32);
        Ok(())
    });
}

/// The faces of the edge `i` of the map, and its edge.
fn edge_and_faces(
    m: &IndexedDataMapOfShapeListOfShape,
    i: usize,
) -> Result<Option<(Edge, Face, Face)>, Failure> {
    let faces = m.find_from_index(i)?;
    int(faces.len() as i32);
    let mut it = faces.iter();
    let (Some(first), Some(second)) = (it.next(), it.next()) else {
        return Ok(None);
    };
    Ok(Some((
        topods::edge(m.find_key(i)?)?,
        topods::face(first)?,
        topods::face(second)?,
    )))
}

/// `fl_concave_side`: see tools/oracle/tkfillet_probe_chfi3d.cxx.
pub fn concave_side_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let s = solid(r).expect("probe solid");
    let m = edge_faces(&s).expect("edge faces");
    int(m.extent() as i32);
    for i in 1..=m.extent() {
        let Some((e, f1, f2)) = edge_and_faces(&m, i).expect("edge") else {
            continue;
        };
        concave(&f1, &f2, &e);
        concave(&f2, &f1, &e);
        concave(&f1, &f2, &topods::edge(&e.reversed()).expect("edge"));
    }
    end();
}

/// The SearchSing values at one parameter.
fn search_sing_values(f: &mut ChFi3dSearchSing, x: f64) {
    let (mut v, mut d, mut v2, mut d2) = (0.0, 0.0, 0.0, 0.0);
    attempt(|| {
        let ok = f.value(x, &mut v);
        failure_of(f)?;
        flag(ok);
        real(v);
        let ok = f.derivative(x, &mut d);
        failure_of(f)?;
        flag(ok);
        real(d);
        let ok = f.values(x, &mut v2, &mut d2);
        failure_of(f)?;
        flag(ok);
        real(v2);
        real(d2);
        Ok(())
    });
}

/// The failure kept by the last evaluation (natively thrown by it).
fn failure_of(f: &mut ChFi3dSearchSing) -> Result<(), Failure> {
    f.take_failure().map_or(Ok(()), Err)
}

/// `fl_search_sing`: see tools/oracle/tkfillet_probe_chfi3d.cxx.
pub fn search_sing_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (c1, c2) = (geom_guide(r), geom_guide(r));
    let mut f = ChFi3dSearchSing::new(&c1, &c2);
    for _ in 0..r.integer() {
        let x = r.real();
        search_sing_values(&mut f, x);
    }
    let (x0, tol, a, b) = (r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let root = FunctionRoot::bounded(&mut f, x0, tol, a, b, 100)?;
        flag(root.is_done());
        if root.is_done() {
            real(root.root()?);
            real(root.value()?);
            real(root.derivative()?);
            int(root.iterations()?);
        }
        Ok(())
    });
    end();
}
