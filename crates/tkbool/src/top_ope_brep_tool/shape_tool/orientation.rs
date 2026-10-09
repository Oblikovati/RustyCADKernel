// Rust translation of the orientation members of TopOpeBRepTool_ShapeTool
// (ShapesSameOriented, SurfacesSameOriented, FacesSameOriented, CurvesSameOriented,
// EdgesSameOriented and EdgeData in
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether shapes have the same orientation: solids always, faces by their normals
//! (planes and cylinders), edges by their tangents (lines), vertices by orientation.
use super::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::project::fun_tool_proj_pons;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_lprop::BRepLPropCLProps;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{epsilon, real_last};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

/// Whether an orientation is INTERNAL or EXTERNAL.
fn is_not_oriented(o: Orientation) -> bool {
    matches!(o, Orientation::Internal | Orientation::External)
}

/// The normal `d1u ^ d1v` of an adaptor at (u, v).
fn normal_at(s: &BRepAdaptorSurface, u: f64, v: f64) -> Result<Vector3, Failure> {
    let d1 = s.d1(u, v)?;
    Ok(d1.d1u.crossed(d1.d1v))
}

/// The normals of two cylinders at the origin of the first and its projection on the
/// second; `None` when the projection fails.
fn cylinder_normals(
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<Option<(Vector3, Vector3)>, Failure> {
    let n1 = normal_at(s1, 0., 0.)?;
    let p1 = s1.d1(0., 0.)?.point;
    let hs2 = s2.transformed_surface().geom_surface_transformed()?;
    let (mut p22d, mut dp2) = (Point2::default(), 0.);
    if !fun_tool_proj_pons(
        p1,
        &hs2,
        &mut p22d,
        &mut dp2,
        ExtFlag::MinMax,
        ExtAlgo::Grad,
    )? {
        return Ok(None);
    }
    Ok(Some((n1, normal_at(s2, p22d.x(), p22d.y())?)))
}

impl TopOpeBRepToolShapeTool {
    /// `ShapesSameOriented(S1, S2)`: the same oriented FORWARD/REVERSED shapes compare
    /// their orientations; else solids are, faces and edges compare their geometry and
    /// vertices their orientation (INTERNAL/EXTERNAL always); true for other types;
    /// `TopOpeBRepToolShapeTool::shapes_same_oriented(&f1, &f2)?`.
    pub fn shapes_same_oriented(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
        let (o1, o2) = (s1.orientation(), s2.orientation());
        if s1.is_same(s2) && !is_not_oriented(o1) && !is_not_oriented(o2) {
            return Ok(o1 == o2);
        }
        Ok(match (s1.shape_type()?, s2.shape_type()?) {
            (ShapeEnum::Solid, ShapeEnum::Solid) => true,
            (ShapeEnum::Face, ShapeEnum::Face) => Self::faces_same_oriented(s1, s2)?,
            (ShapeEnum::Edge, ShapeEnum::Edge) => Self::edges_same_oriented(s1, s2)?,
            (ShapeEnum::Vertex, ShapeEnum::Vertex) => {
                is_not_oriented(o1) || is_not_oriented(o2) || o1 == o2
            }
            _ => true,
        })
    }

    /// `SurfacesSameOriented(S1, Sref)`: two planes compare their normals at their first
    /// parameters, two cylinders at the origin of the first and its projection on the
    /// second (true when it fails); true for other surfaces (NYI natively);
    /// `TopOpeBRepToolShapeTool::surfaces_same_oriented(&s1, &s2)?`.
    pub fn surfaces_same_oriented(
        s1: &BRepAdaptorSurface,
        s2: &BRepAdaptorSurface,
    ) -> Result<bool, Failure> {
        let (st1, st2) = (s1.get_type()?, s2.get_type()?);
        if st1 == SurfaceType::Plane && st2 == SurfaceType::Plane {
            let n1 = normal_at(s1, s1.first_u_parameter()?, s1.first_v_parameter()?)?;
            let n2 = normal_at(s2, s2.first_u_parameter()?, s2.first_v_parameter()?)?;
            return Ok(n1.dot(n2) > 0.);
        }
        if st1 == SurfaceType::Cylinder && st2 == SurfaceType::Cylinder {
            return Ok(cylinder_normals(s1, s2)?.is_none_or(|(n1, n2)| n1.dot(n2) > 0.));
        }
        Ok(true)
    }

    /// `FacesSameOriented(S1, Sref)`: true for an INTERNAL or EXTERNAL face; else the
    /// same faces or same oriented surfaces, inverted when the orientations differ;
    /// `TopOpeBRepToolShapeTool::faces_same_oriented(&f1, &f2)?`.
    pub fn faces_same_oriented(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
        let (f1, f2) = (topods::face(s1)?, topods::face(s2)?);
        let (o1, o2) = (s1.orientation(), s2.orientation());
        if is_not_oriented(o1) || is_not_oriented(o2) {
            return Ok(true);
        }
        let bas1 = BRepAdaptorSurface::with_face(&f1, false)?;
        let bas2 = BRepAdaptorSurface::with_face(&f2, false)?;
        let so = s1.is_same(s2) || Self::surfaces_same_oriented(&bas1, &bas2)?;
        Ok(if o1 != o2 { !so } else { so })
    }

    /// `CurvesSameOriented(C1, Cref)`: two lines compare their tangents at their first
    /// parameters; true for other curves (NYI natively);
    /// `TopOpeBRepToolShapeTool::curves_same_oriented(&c1, &c2)?`.
    pub fn curves_same_oriented(
        c1: &BRepAdaptorCurve,
        c2: &BRepAdaptorCurve,
    ) -> Result<bool, Failure> {
        if c1.get_type()? != CurveType::Line || c2.get_type()? != CurveType::Line {
            return Ok(true);
        }
        let (mut t1, mut n1, mut k1) = (Direction3::Z, Direction3::Z, 0.);
        Self::edge_data(c1, c1.first_parameter()?, &mut t1, &mut n1, &mut k1)?;
        let (mut t2, mut n2, mut k2) = (Direction3::Z, Direction3::Z, 0.);
        Self::edge_data(c2, c2.first_parameter()?, &mut t2, &mut n2, &mut k2)?;
        Ok(t1.dot(t2) > 0.)
    }

    /// `EdgesSameOriented(E1, Eref)`: true for an INTERNAL or EXTERNAL edge; else the
    /// same oriented curves, inverted when the orientations differ;
    /// `TopOpeBRepToolShapeTool::edges_same_oriented(&e1, &e2)?`.
    pub fn edges_same_oriented(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
        let (e1, e2) = (topods::edge(s1)?, topods::edge(s2)?);
        let (o1, o2) = (s1.orientation(), s2.orientation());
        if is_not_oriented(o1) || is_not_oriented(o2) {
            return Ok(true);
        }
        let so = Self::curves_same_oriented(
            &BRepAdaptorCurve::with_edge(&e1)?,
            &BRepAdaptorCurve::with_edge(&e2)?,
        )?;
        Ok(if o1 != o2 { !so } else { so })
    }

    /// `EdgeData(BRAC, P, T, N, C)`: the tangent and curvature at `p` (LProp_NotDefined
    /// natively when the tangent is undefined); the normal threshold is RealLast, so
    /// `n` is never written, as natively; returns Precision::Angular();
    /// `TopOpeBRepToolShapeTool::edge_data(&bac, 0.5, &mut t, &mut n, &mut c)?`.
    pub fn edge_data(
        bac: &BRepAdaptorCurve,
        p: f64,
        t: &mut Direction3,
        n: &mut Direction3,
        c: &mut f64,
    ) -> Result<f64, Failure> {
        let tol: f64 = precision::angular();
        let mut bl = BRepLPropCLProps::with_parameter(bac, p, 2, tol)?;
        *t = bl.tangent()?;
        *c = bl.curvature()?;
        // xpu150399 cto900R4
        let tolm: f64 = tol.max(epsilon(0.).max(real_last()));
        if c.abs() > tolm {
            *n = bl.normal()?;
        }
        Ok(tol)
    }

    /// `EdgeData(E, P, T, N, C)`: on the adaptor of the edge;
    /// `TopOpeBRepToolShapeTool::edge_data_of_edge(&edge, 0.5, &mut t, &mut n, &mut c)?`.
    pub fn edge_data_of_edge(
        e: &Shape,
        p: f64,
        t: &mut Direction3,
        n: &mut Direction3,
        c: &mut f64,
    ) -> Result<f64, Failure> {
        let bac = BRepAdaptorCurve::with_edge(&topods::edge(e)?)?;
        Self::edge_data(&bac, p, t, n, c)
    }
}
