//! Split from faulty.rs.
use super::{
    CORRISO_DECREASE, CORRISO_INCREASE, CORRISO_SPLITEDGE, EdgeFlagMap, TopOpeBRepToolCORRISO,
};
use crate::top_ope_brep_tool::tool_iso::uviso;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `FUN_tool_recadre(minx, maxx, xfirst, xlast, tolx, maxsup)` (static): INCREASE when
/// the range is before the bounds, DECREASE after them, SPLITEDGE when it crosses one
/// of them, 10 otherwise; with whether the max is beyond the last bound.
fn fun_tool_recadre(minx: f64, maxx: f64, xfirst: f64, xlast: f64, tolx: f64) -> (i32, bool) {
    let maxinf = maxx < xfirst + tolx;
    let mininf = minx < xfirst - tolx;
    let maxsup = maxx > xlast + tolx;
    let minsup = minx > xlast - tolx;
    let maxok = xfirst - tolx < maxx && maxx < xlast + tolx;
    let minok = xfirst - tolx < minx && minx < xlast + tolx;
    let recadre = if maxinf {
        CORRISO_INCREASE
    } else if minsup {
        CORRISO_DECREASE
    } else if (mininf && maxok) || (minok && maxsup) {
        CORRISO_SPLITEDGE
    } else {
        10
    };
    (recadre, maxsup)
}

impl TopOpeBRepToolCORRISO {
    /// `EdgeOUTofBoundsUV(E, onU, tolx, parspE)`: INCREASE or DECREASE when the UV
    /// representation of the edge is out of the bounds of the reference surface in the
    /// direction (with the split parameter for SPLITEDGE of an iso across), 10 when it
    /// is inside or crosses a bound (non-iso), 0 for an unknown edge;
    /// `let (code, par) = c.edge_out_of_bounds_uv(&e, true, 1e-9)?;`.
    pub fn edge_out_of_bounds_uv(
        &self,
        e: &Edge,
        on_u: bool,
        tolx: f64,
    ) -> Result<(i32, f64), Failure> {
        let parsp_e = -1.0e7;
        let Some(c2df) = self.my_e_rep2d.seek(e) else {
            return Ok((0, parsp_e));
        };
        let (pc, f, l, tol) = c2df.pc();
        let gas = &self.my_gas;
        let xfirst = if on_u {
            gas.first_u_parameter()?
        } else {
            gas.first_v_parameter()?
        };
        let xlast = if on_u {
            gas.last_u_parameter()?
        } else {
            gas.last_v_parameter()?
        };
        let xperiod = if on_u { self.my_uper } else { self.my_vper };
        let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
        let (iso, isou, isov) = uviso(&pc, &mut d2d, &mut o2d)?;
        if !iso {
            let mut bn2d = BoundingBox2::default();
            let gc2d = Geom2dAdaptorCurve::with_range(&pc, f, l)?;
            let toladd = std_max(BRepTool::edge_tolerance(e)?, tol);
            Add2dCurve::add_adaptor(&gc2d, toladd, &mut bn2d)?;
            let [umin, vmin, umax, vmax] = bn2d.get()?;
            let (xmin, xmax) = if on_u { (umin, umax) } else { (vmin, vmax) };
            let (recadre, _) = fun_tool_recadre(xmin, xmax, xfirst, xlast, tolx);
            return Ok((
                if recadre == CORRISO_SPLITEDGE {
                    10
                } else {
                    recadre
                },
                parsp_e,
            ));
        }
        if (on_u && isou) || (!on_u && isov) {
            let xpar = if on_u { o2d.x() } else { o2d.y() };
            let mut recadre = 10;
            if xpar < xfirst - tolx {
                recadre = CORRISO_INCREASE;
            }
            if xpar > xfirst + xperiod + tolx {
                recadre = CORRISO_DECREASE;
            }
            return Ok((recadre, parsp_e));
        }
        let reverse = (if on_u { d2d.y() } else { d2d.x() }) < 0.;
        let xfactor = if reverse { -1. } else { 1. };
        let (max, min) = if reverse { (f, l) } else { (l, f) };
        let curve = pc.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "TopOpeBRepTool_CORRISO::EdgeOUTofBoundsUV: null p-curve; expected a 2d curve",
                "TopOpeBRepTool_CORRISO",
            )
        })?;
        let (maxuv, minuv) = (curve.value(max)?, curve.value(min)?);
        let maxx = if on_u { maxuv.x() } else { maxuv.y() };
        let minx = if on_u { minuv.x() } else { minuv.y() };
        let (recadre, maxsup) = fun_tool_recadre(minx, maxx, xfirst, xlast, tolx);
        if recadre == CORRISO_SPLITEDGE {
            let xbound = if maxsup { xperiod } else { 0. };
            return Ok((recadre, max - xfactor * (maxx - xbound)));
        }
        Ok((recadre, parsp_e))
    }

    /// `EdgesOUTofBoundsUV(EdsToCheck, onU, tolx, FyEds)`: the edges to translate
    /// forwards (1) or backwards (-1); `c.edges_out_of_bounds_uv(&eds, true, 1e-9, &mut fy)?`.
    pub fn edges_out_of_bounds_uv(
        &self,
        eds_to_check: &List<Shape>,
        on_u: bool,
        tolx: f64,
        fy_eds: &mut EdgeFlagMap,
    ) -> Result<bool, Failure> {
        fy_eds.clear(false);
        for es in eds_to_check.iter() {
            let (recadre, _) = self.edge_out_of_bounds_uv(&topods::edge(es)?, on_u, tolx)?;
            if recadre == CORRISO_INCREASE {
                fy_eds.bind(es.clone(), 1);
            }
            if recadre == CORRISO_DECREASE {
                fy_eds.bind(es.clone(), -1);
            }
        }
        Ok(!fy_eds.is_empty())
    }
}
