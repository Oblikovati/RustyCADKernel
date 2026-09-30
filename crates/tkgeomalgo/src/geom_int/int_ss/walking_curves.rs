//! The curves of a walking line: polygonal B-splines of its parts, or their
//! approximation by `GeomInt_WLApprox` (on the decomposed lines when the line crosses
//! the seams), in the plane when one surface is a plane.
use super::{GeomIntIntSS, GeomIntIntSSApprox};
use crate::approx_int::{ApproxIntKnotTools, ApproxIntRange};
use crate::geom_int::instances::GeomIntWLApprox;
use crate::geom_int::line_tool::{GeomIntLineTool, line_as};
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLine};
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::SurfaceType;

/// `myTolApprox`: the 3d and 2d tolerances of the approximation.
const TOL_APPROX: f64 = 0.0000001;
/// `aTolSS`: the tolerance of the decomposition of the walking lines.
const TOL_SS: f64 = 2.0e-7;

/// The walking line and the range of one part (`WL`, `ifprm`, `ilprm`).
pub(super) struct WalkingPart {
    pub(super) wl: Handle<IntPatchWLine>,
    pub(super) ifprm: i32,
    pub(super) ilprm: i32,
}

impl GeomIntIntSS {
    /// The polygonal B-splines of a part (3d, and 2d when asked); the 3d one is made
    /// first after a failed approximation, last otherwise, as natively.
    pub(super) fn polygonal_part(
        &mut self,
        part: &WalkingPart,
        approx: GeomIntIntSSApprox,
        bsp_first: bool,
    ) -> Result<(), Failure> {
        let (wl, f, l) = (&part.wl, part.ifprm, part.ilprm);
        let early = if bsp_first {
            Some(Self::make_bspline(wl, f, l)?)
        } else {
            None
        };
        let h1: Handle<dyn Geom2dCurve> = if approx.approx_s1 {
            Self::make_bspline2d(wl, f, l, true)?.cast()
        } else {
            Handle::default()
        };
        let h2: Handle<dyn Geom2dCurve> = if approx.approx_s2 {
            Self::make_bspline2d(wl, f, l, false)?.cast()
        } else {
            Handle::default()
        };
        let bsp = match early {
            Some(bsp) => bsp,
            None => Self::make_bspline(wl, f, l)?,
        };
        self.sline.push(bsp);
        self.sline_s1.push(h1);
        self.sline_s2.push(h2);
        Ok(())
    }

    /// The curves of a walking line.
    pub(super) fn walking_curves(
        &mut self,
        wl: Handle<IntPatchWLine>,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        if !approx.approx {
            for i in 1..=self.my_lconstruct.nb_parts()? {
                let (fprm, lprm) = self.my_lconstruct.part(i)?;
                let part = WalkingPart {
                    wl: wl.clone(),
                    ifprm: native_int_cast(fprm),
                    ilprm: native_int_cast(lprm),
                };
                self.polygonal_part(&part, approx, false)?;
            }
            return Ok(());
        }
        self.approximated_curves(wl, approx)
    }

    /// The approximated curves of the parts of a walking line (of its decomposed lines
    /// when it crosses the seams).
    fn approximated_curves(
        &mut self,
        wl: Handle<IntPatchWLine>,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        let mut theapp3d = GeomIntWLApprox::new();
        let tol2d: f64 = TOL_APPROX;
        let two_surfaces: bool = self.my_hs1 != self.my_hs2;
        theapp3d.set_parameters(
            TOL_APPROX,
            tol2d,
            4,
            8,
            0,
            30,
            two_surfaces,
            ParametrizationType::ChordLength,
        );
        let mut seq_of_l: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        let decomposed: bool = GeomIntLineTool::decomposition_of_wline(
            &wl,
            &self.my_hs1,
            &self.my_hs2,
            TOL_SS,
            &self.my_lconstruct,
            &mut seq_of_l,
        )?;
        let nb_parts: i32 = self.my_lconstruct.nb_parts()?;
        let nbiter: i32 = if decomposed {
            seq_of_l.len() as i32
        } else {
            nb_parts
        };
        let mut current: Handle<IntPatchWLine> = wl;
        for i in 1..=nbiter {
            let part = if decomposed {
                current = line_as::<IntPatchWLine>(
                    &seq_of_l[(i - 1) as usize],
                    "GeomInt_IntSS::MakeCurve",
                )?;
                let ilprm: i32 = current.nb_pnts()?;
                WalkingPart {
                    wl: current.clone(),
                    ifprm: 1,
                    ilprm,
                }
            } else {
                let (fprm, lprm) = self.my_lconstruct.part(i)?;
                WalkingPart {
                    wl: current.clone(),
                    ifprm: native_int_cast(fprm),
                    ilprm: native_int_cast(lprm),
                }
            };
            self.approximate_part(&mut theapp3d, &part, approx)?;
        }
        Ok(())
    }

    /// The approximation of one part: in the plane (2d) when one surface is a plane.
    fn approximate_part(
        &mut self,
        theapp3d: &mut GeomIntWLApprox,
        part: &WalkingPart,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        let (typs1, typs2) = (self.my_hs1.get_type()?, self.my_hs2.get_type()?);
        let (mut an_approx, mut an_approx1, mut an_approx2) =
            (approx.approx, approx.approx_s1, approx.approx_s2);
        if typs1 == SurfaceType::Plane {
            an_approx = false;
            an_approx1 = true;
        } else if typs2 == SurfaceType::Plane {
            an_approx = false;
            an_approx2 = true;
        }
        let par_type: ParametrizationType = ApproxIntKnotTools::define_par_type(
            &part.wl,
            part.ifprm,
            part.ilprm,
            (an_approx, an_approx1, an_approx2),
        )?;
        let two_surfaces: bool = self.my_hs1 != self.my_hs2;
        theapp3d.set_parameters(TOL_APPROX, TOL_APPROX, 4, 8, 0, 30, two_surfaces, par_type);
        // lbr: when a surface is a plane, the approximation is made in 2d on it and the
        // 2d points are brought back into 3d.
        let range = |xyz: bool, u1v1: bool, u2v2: bool| ApproxIntRange {
            approx_xyz: xyz,
            approx_u1v1: u1v1,
            approx_u2v2: u2v2,
            indicemin: part.ifprm,
            indicemax: part.ilprm,
        };
        let hs1: Adaptor3dSurfaceHandle = self.my_hs1.cast();
        let hs2: Adaptor3dSurfaceHandle = self.my_hs2.cast();
        let r: ApproxIntRange = if typs1 == SurfaceType::Plane {
            range(false, true, approx.approx_s2)
        } else if typs2 == SurfaceType::Plane {
            range(false, approx.approx_s1, true)
        } else {
            let spline = |t: SurfaceType| {
                matches!(t, SurfaceType::BezierSurface | SurfaceType::BSplineSurface)
            };
            if two_surfaces && spline(typs1) && spline(typs2) {
                theapp3d.set_parameters(TOL_APPROX, TOL_APPROX, 4, 8, 0, 30, true, par_type);
            }
            range(true, approx.approx_s1, approx.approx_s2)
        };
        theapp3d.perform_on_surfaces(&hs1, &hs2, &part.wl, r)?;
        if !theapp3d.is_done() {
            return self.polygonal_part(part, approx, true);
        }
        self.update_tolerances(theapp3d, (typs1, typs2), approx);
        self.approximated_part_curves(theapp3d, (typs1, typs2), approx)
    }

    /// The reached tolerances after a successful approximation.
    fn update_tolerances(
        &mut self,
        theapp3d: &GeomIntWLApprox,
        (typs1, typs2): (SurfaceType, SurfaceType),
        approx: GeomIntIntSSApprox,
    ) {
        let plane: bool = typs1 == SurfaceType::Plane || typs2 == SurfaceType::Plane;
        if (approx.approx_s1 || approx.approx_s2 || plane)
            && (theapp3d.tol_reached2d() > self.my_tol_reached2d || self.my_tol_reached2d == 0.0)
        {
            self.my_tol_reached2d = theapp3d.tol_reached2d();
        }
        if plane {
            self.my_tol_reached3d = self.my_tol_reached2d;
        } else if theapp3d.tol_reached3d() > self.my_tol_reached3d || self.my_tol_reached3d == 0.0 {
            self.my_tol_reached3d = theapp3d.tol_reached3d();
        }
    }
}
