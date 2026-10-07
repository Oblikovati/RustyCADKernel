//! HLRBRep_EdgeFaceTool.hxx/.cxx and HLRBRep_TypeOfResultingEdge.hxx, OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Curve, Surface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{self, Direction3, Vector3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPF;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;
use std::cmp::Ordering;

/// HLRBRep_TypeOfResultingEdge: the kinds of edges HLRBRep_HLRToShape extracts.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum TypeOfResultingEdge {
    /// `HLRBRep_Undefined`: no type.
    #[default]
    Undefined,
    /// `HLRBRep_IsoLine`: an isoparametric line of a face.
    IsoLine,
    /// `HLRBRep_OutLine`: an apparent contour of a face.
    OutLine,
    /// `HLRBRep_Rg1Line`: a smooth (G1) edge.
    Rg1Line,
    /// `HLRBRep_RgNLine`: a sewn edge of higher continuity.
    RgNLine,
    /// `HLRBRep_Sharp`: a sharp edge.
    Sharp,
}

/// HLRBRep_EdgeFaceTool: the normal curvature of a face along a direction and the
/// (u, v) of an edge point on a face; `EdgeFaceTool::curvature_value(&s, 0.5, 0.5, d)?`.
pub struct EdgeFaceTool;

impl EdgeFaceTool {
    /// CurvatureValue(F, U, V, Tg): the normal curvature of the surface at (u, v)
    /// along the tangent `tg` projected on the tangent plane; 0 when the first
    /// derivatives are parallel; `EdgeFaceTool::curvature_value(&s, 0.5, 0.5, d)?`.
    pub fn curvature_value(f: &Surface, u: f64, v: f64, tg: Direction3) -> Result<f64, Failure> {
        let d2 = f.d2(u, v)?;
        let tgv = Vector3::from(tg);
        let (d1u, d1v) = (d2.d1u, d2.d1v);
        let d1ut = d1u.dot(tgv);
        let d1vt = d1v.dot(tgv);
        let d1ud1v = d1u.dot(d1v);
        let nmu2 = d1u.dot(d1u);
        let nmv2 = d1v.dot(d1v);
        let det = nmu2 * nmv2 - d1ud1v * d1ud1v;
        // Not above the resolution (NaN included) gives 0, as the native else branch.
        if det.partial_cmp(&gp::resolution()) != Some(Ordering::Greater) {
            return Ok(0.);
        }
        let alfa = (d1ut * nmv2 - d1vt * d1ud1v) / det;
        let beta = (d1vt * nmu2 - d1ut * d1ud1v) / det;
        let (alfa2, beta2, alfabeta) = (alfa * alfa, beta * beta, alfa * beta);
        let mut nm = d1u.crossed(d1v);
        nm.normalize()?;
        let n = nm.dot(d2.d2u) * alfa2 + 2. * nm.dot(d2.d2uv) * alfabeta + nm.dot(d2.d2v) * beta2;
        let d = nmu2 * alfa2 + 2. * d1ud1v * alfabeta + nmv2 * beta2;
        Ok(n / d)
    }

    /// UVPoint(Par, E, F, U, V): the (u, v) of the edge point of parameter `par` on
    /// the face: from the edge's p-curve when it has one, else from the nearest
    /// extremum of the 3D point on the face (`None` when there is none);
    /// `EdgeFaceTool::uv_point(0.5, &curve, &surface)?`.
    pub fn uv_point(par: f64, e: &Curve, f: &Surface) -> Result<Option<(f64, f64)>, Failure> {
        let edge = e.curve().edge();
        let face = f.surface().face();
        if !BRepTool::curve_on_face(&edge, &face)?.pcurve.is_null() {
            let pc = BRepAdaptorCurve2d::with_edge_on_face(&edge, &face)?;
            let p2d = pc.d0(par)?;
            return Ok(Some((p2d.x(), p2d.y())));
        }
        let vertex = BRepLibMakeVertex::new(e.value_3d(par)?)?.vertex()?;
        let proj =
            BRepExtremaExtPF::with_vertex_face(&vertex, &face, ExtFlag::MinMax, ExtAlgo::Grad)?;
        let (mut index, mut dist2) = (0, real_last());
        for i in 1..=proj.nb_ext() {
            let newdist2 = proj.square_distance(i)?;
            if newdist2 < dist2 {
                dist2 = newdist2;
                index = i;
            }
        }
        if index == 0 {
            return Ok(None);
        }
        Ok(Some(proj.parameter(index)?))
    }
}
