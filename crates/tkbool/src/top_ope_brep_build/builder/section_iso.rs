//! The iso section edges on periodic faces (TopOpeBRepBuild_Section.cxx): the static
//! periodicity and bound tests, and the splits ON moved onto the side of their edge.
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_tool::{
    TopOpeBRepToolShapeTool, TopOpeBRepToolTOOL, fc2d_curve_on_surface, fun_tool_bounds,
    fun_tool_par_f, uviso,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// `FUN_periodicS(F)` (static): a face on a cylinder, cone or sphere basis surface.
pub(crate) fn fun_periodic_s(f: &Shape) -> Result<bool, Failure> {
    if f.is_null() || f.shape_type()? != ShapeEnum::Face {
        return Ok(false);
    }
    let sss = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    let gas = GeomAdaptorSurface::with_surface(&sss)?;
    Ok(matches!(
        gas.get_type()?,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere
    ))
}

/// `FUN_periodic(F, uper, vper)` (static): the periodicities of the surface of the
/// face when one is periodic.
pub(crate) fn fun_periodic(f: &Face) -> Result<Option<(bool, bool)>, Failure> {
    let su = BRepTool::surface(f)?;
    let s = su.get().ok_or_else(null_hds)?;
    let (uper, vper) = (s.is_u_periodic(), s.is_v_periodic());
    Ok((uper || vper).then_some((uper, vper)))
}

/// `FUN_onboundsper(uv, F)` (static): the point is on a bound of the periodic direction
/// (U first) of the surface of the face.
fn fun_onboundsper_uv(uv: Point2, f: &Face) -> Result<bool, Failure> {
    let su = BRepTool::surface(f)?;
    let s = su.get().ok_or_else(null_hds)?;
    let (uclo, vclo) = (s.is_u_periodic(), s.is_v_periodic());
    let (u1, u2, v1, v2) = s.bounds();
    let toluv = 1.0e-8 * 1.0e-2;
    if uclo {
        return Ok((u1 - uv.x()).abs() < toluv || (u2 - uv.x()).abs() < toluv);
    }
    if vclo {
        return Ok((v1 - uv.y()).abs() < toluv || (v2 - uv.y()).abs() < toluv);
    }
    Ok(false)
}

/// `FUN_onboundsper(E, F, uv)` (static): the point of the edge at a fixed inner
/// parameter is on a periodic bound of the face; the uv point (None when the point
/// is not found on the face).
fn fun_onboundsper_edge(e: &Edge, f: &Face) -> Result<(bool, Point2), Failure> {
    let (first, last) = fun_tool_bounds(e)?;
    let x = 0.45678;
    let par = first * x + (1. - x) * last;
    let mut uv = Point2::default();
    if !fun_tool_par_f(e, par, f, &mut uv)? {
        return Ok((false, uv));
    }
    Ok((fun_onboundsper_uv(uv, f)?, uv))
}

impl TopOpeBRepBuildBuilder {
    /// The splits ON of an iso section edge of a periodic face moved by the period
    /// side of their 3d point (None when the edge p-curve is not an iso);
    /// Standard_ProgramError when a split is not the same iso.
    pub(crate) fn iso_moved_splits(
        &self,
        eg: &Edge,
        ff: &Face,
        per: (bool, bool),
    ) -> Result<Option<List<Shape>>, Failure> {
        let (uper, vper) = per;
        let (pc, _, _, _) = fc2d_curve_on_surface(eg, ff, true)?;
        let (mut d2d, mut o2d) = (Direction2::default(), Point2::default());
        let (iso, iso_u, iso_v) = uviso(&pc, &mut d2d, &mut o2d)?;
        if !iso {
            return Ok(None);
        }
        let mut newlon = List::default();
        for e in self.splits(eg.as_ref(), State::On).iter() {
            let eon = topods::edge(e)?;
            let (pcon, _, _, _) = fc2d_curve_on_surface(&eon, ff, true)?;
            let (mut d2don, mut o2don) = (Direction2::default(), Point2::default());
            let (iso_on, isouon, isovon) = uviso(&pcon, &mut d2don, &mut o2don)?;
            if !iso_on || (iso_u && !isouon) || (iso_v && !isovon) {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    "TopOpeBRepBuild_Builder::splitON",
                    "TopOpeBRepBuild_Builder",
                ));
            }
            if (uper && iso_u) || (vper && iso_v) {
                let (isonclo, uvok) = fun_onboundsper_edge(&eon, ff)?;
                if !isonclo && fun_onboundsper_uv(o2don, ff)? {
                    let tr = if iso_u {
                        Vector2::new(uvok.x() - o2don.x(), 0.)
                    } else {
                        Vector2::new(0., uvok.y() - o2don.y())
                    };
                    if tr.magnitude() > 1.0e-8 * 1.0e2 {
                        TopOpeBRepToolTOOL::trsl_uv_modif_e(tr, ff, &eon)?;
                    }
                }
            }
            newlon.append(eon.into_shape());
        }
        Ok(Some(newlon))
    }
}
