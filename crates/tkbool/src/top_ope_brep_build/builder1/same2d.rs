//! IsSame2d of TopOpeBRepBuild_Builder1: the seam pair of p-curves of a tool piece on a periodic object face.
use super::pieces_on::TwoPiecesSeq;
use crate::top_ope_brep_build::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::tools::PAR_T;
use crate::top_ope_brep_tool::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Vector2;

impl TopOpeBRepBuildBuilder {
    /// `IsSame2d(aSeq, aListOfPiecesOut2d)` (Builder1): on a periodic face with closed
    /// edges, the tool piece given a seam pair of p-curves on the object face (its own
    /// and one translated onto the object piece) and listed; returns 0 then, else 1;
    /// `TopOpeBRepBuildBuilder::is_same2d(&seq, &mut lo)?`.
    pub fn is_same2d(seq: &TwoPiecesSeq, out2d: &mut List<Shape>) -> Result<i32, Failure> {
        let f_obj = topods::face(&seq.face_obj)?;
        let bas = BRepAdaptorSurface::with_face(&f_obj, true)?;
        if !(bas.is_u_periodic()? || bas.is_v_periodic()?) {
            return Ok(1);
        }
        if !BRepTool::is_closed(&seq.edge_obj)? || !BRepTool::is_closed(&seq.edge_tool)? {
            return Ok(1);
        }
        let (e_obj, e_tool) = (topods::edge(&seq.edge_obj)?, topods::edge(&seq.edge_tool)?);
        let (p_obj, p_tool) = (
            topods::edge(&seq.piece_obj)?,
            topods::edge(&seq.piece_tool)?,
        );
        let null = || {
            Failure::with_message(
                "Standard_NullObject",
                "TopOpeBRepBuild_Builder1::IsSame2d: no p-curve",
            )
        };
        let (c2d_piece_tool, f1, l1, _) = fc2d_curve_on_surface(&p_tool, &f_obj, true)?;
        let p_piece = c2d_piece_tool
            .get()
            .ok_or_else(null)?
            .value(f1 * PAR_T + (1. - PAR_T) * l1)?;
        let (c2d, f, l, _) = fc2d_curve_on_surface(&e_tool, &f_obj, true)?;
        let mut par_ori =
            ProjectPointOnCurve::with_curve(p_piece, &c2d)?.lower_distance_parameter()?;
        if par_ori < f {
            par_ori += 2. * std::f64::consts::PI;
        }
        if par_ori > l {
            par_ori -= 2. * std::f64::consts::PI;
        }
        let uv2 = c2d.get().ok_or_else(null)?.value(par_ori)?;
        let (c2d_piece_obj, f, l, _) = fc2d_curve_on_surface(&p_obj, &f_obj, true)?;
        let uv1 = c2d_piece_obj
            .get()
            .ok_or_else(null)?
            .value(f * PAR_T + (1. - PAR_T) * l)?;
        let translate = Vector2::between(uv1, uv2);
        if translate.magnitude() < 1.0e-10 {
            return Ok(1);
        }
        let tr: Handle<dyn Geom2dCurve> = c2d_piece_tool.get().ok_or_else(null)?.copy()?.cast();
        let t = tr.get().expect("copied p-curve");
        t.translate(translate)?;
        let tr_vec = Vector2::between(t.value(f1)?, t.value(l1)?);
        let (c2d_edge_obj, fo, lo, _) = fc2d_curve_on_surface(&e_obj, &f_obj, true)?;
        let ce = c2d_edge_obj.get().ok_or_else(null)?;
        let mut o_vec = Vector2::between(ce.value(fo)?, ce.value(lo)?);
        if e_obj.orientation() == Orientation::Reversed {
            o_vec.reverse();
        }
        let tol_e = BRepTool::edge_tolerance(&p_tool)?;
        let pair = if tr_vec.dot(o_vec) <= 0. {
            (&tr, &c2d_piece_tool)
        } else {
            (&c2d_piece_tool, &tr)
        };
        BRepBuilder::default().update_edge_pcurves_on_face(&p_tool, pair, &f_obj, tol_e)?;
        out2d.append(p_tool.into());
        Ok(0)
    }
}
