//! The private steps of the approximation: the offsets, the knots and the curve.
use super::perform::ApproxIntRange;
use super::{ApproxIntApprox, ApproxIntApproxTypes};
use crate::approx_int::knot_tools::{ApproxIntKnotPointSets, ApproxIntKnotTools};
use crate::approx_int::line::ApproxIntLine;
use crate::approx_int::multi_line::{ApproxIntMultiLine, ApproxIntMultiLineLayout};
use crate::approx_int::sv_surfaces::SvSurfacesRef;
use crate::approx_int::test_points::TestPoints;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::math::Vector;

/// `ComputeTrsf3d(theline, theXo, theYo, theZo)`: the opposite of the minimal
/// coordinates of the points.
fn compute_trsf3d<L: ApproxIntLine>(line: &L) -> Result<(f64, f64, f64), Failure> {
    let (mut xmin, mut ymin, mut zmin) = (real_last(), real_last(), real_last());
    for i in 1..=line.nb_pnts()? {
        let p = line.point(i)?.value();
        xmin = std_min(p.x(), xmin);
        ymin = std_min(p.y(), ymin);
        zmin = std_min(p.z(), zmin);
    }
    Ok((-xmin, -ymin, -zmin))
}

/// `ComputeTrsf2d(theline, onFirst, theUo, theVo)`: the opposite of the minimal
/// parameters of the points on one surface.
fn compute_trsf2d<L: ApproxIntLine>(line: &L, on_first: bool) -> Result<(f64, f64), Failure> {
    let (mut umin, mut vmin) = (real_last(), real_last());
    for i in 1..=line.nb_pnts()? {
        let (u, v) = line.point(i)?.parameters_on_surface(on_first);
        umin = std_min(u, umin);
        vmin = std_min(v, vmin);
    }
    Ok((-umin, -vmin))
}

impl<B: ApproxIntApproxTypes> ApproxIntApprox<B> {
    /// `prepareDS(theApproxXYZ, theApproxU1V1, theApproxU2V2, theIndicemin,
    /// theIndicemax)`.
    pub(super) fn prepare_ds(&mut self, range: ApproxIntRange) {
        self.my_tol_reached3d = 0.0;
        self.my_tol_reached2d = 0.0;
        self.my_data.approx_u1v1 = range.approx_u1v1;
        self.my_data.approx_u2v2 = range.approx_u2v2;
        self.my_data.approx_xyz = range.approx_xyz;
        self.my_data.indicemin = range.indicemin;
        self.my_data.indicemax = range.indicemax;
        self.my_data.parametrization = self.my_compute_line_bezier.parametrization();
    }

    /// `fillData(theline)`: the offsets of the approximated spaces (0 for the others).
    pub(super) fn fill_data(&mut self, line: &Handle<B::Line>) -> Result<(), Failure> {
        let l: &B::Line = line_ref::<B::Line>(line)?;
        let o = &mut self.my_data.offsets;
        (o.xo, o.yo, o.zo) = if self.my_data.approx_xyz {
            compute_trsf3d(l)?
        } else {
            (0.0, 0.0, 0.0)
        };
        (o.u1o, o.v1o) = if self.my_data.approx_u1v1 {
            compute_trsf2d(l, true)?
        } else {
            (0.0, 0.0)
        };
        (o.u2o, o.v2o) = if self.my_data.approx_u2v2 {
            compute_trsf2d(l, false)?
        } else {
            (0.0, 0.0)
        };
        Ok(())
    }

    /// The multi-line of the points `imin..=imax` with the current data.
    fn multi_line(
        &self,
        line: &Handle<B::Line>,
        sv: Option<SvSurfacesRef>,
        (imin, imax): (i32, i32),
    ) -> ApproxIntMultiLine<B::Line> {
        let d = &self.my_data;
        let layout = ApproxIntMultiLineLayout {
            nb_p3d: i32::from(d.approx_xyz),
            nb_p2d: i32::from(d.approx_u1v1) + i32::from(d.approx_u2v2),
            approx_u1v1: d.approx_u1v1,
            approx_u2v2: d.approx_u2v2,
            offsets: d.offsets,
            p2d_on_first: d.approx_u1v1,
            ind_min: imin,
            ind_max: imax,
        };
        ApproxIntMultiLine::with_sv_surfaces(line.clone(), sv, layout)
    }

    /// `buildKnots(theline, thePtrSVSurf)`: the ends of the range for one B-spline,
    /// else the knots of `ApproxInt_KnotTools` on the expected parameters.
    pub(super) fn build_knots(
        &mut self,
        line: &Handle<B::Line>,
        sv: Option<SvSurfacesRef>,
    ) -> Result<(), Failure> {
        self.my_knots.clear();
        let (imin, imax) = (self.my_data.indicemin, self.my_data.indicemax);
        if !self.my_data.bezier_approx {
            self.my_knots.push(imin);
            self.my_knots.push(imax);
            return Ok(());
        }
        let test_line = self.multi_line(line, sv, (imin, imax));
        let points = TestPoints::read(&test_line, (imin, imax), self.my_data.approx_u1v1)?;
        // Expected parametrization.
        let mut pars = Vector::filled(imin, imax, 0.0)?;
        Self::parameters(
            &test_line,
            imin,
            imax,
            self.my_data.parametrization,
            &mut pars,
        )?;
        let sets = ApproxIntKnotPointSets {
            xyz: &points.xyz,
            u1v1: &points.u1v1,
            u2v2: &points.u2v2,
            approx_xyz: self.my_data.approx_xyz,
            approx_u1v1: self.my_data.approx_u1v1,
            approx_u2v2: self.my_data.approx_u2v2,
        };
        ApproxIntKnotTools::build_knots(sets, &pars, self.my_data.nb_pnt_max, &mut self.my_knots)
    }

    /// `UpdateTolReached()`: the largest error of the Bezier pieces, or the error of the
    /// B-spline.
    fn update_tol_reached(&mut self) -> Result<(), Failure> {
        if !self.my_data.bezier_approx {
            (self.my_tol_reached3d, self.my_tol_reached2d) = self.my_compute_line.error();
            return Ok(());
        }
        for i_cur in 1..=self.my_compute_line_bezier.nb_multi_curves() {
            let (tol3d, tol2d) = self.my_compute_line_bezier.error(i_cur)?;
            self.my_tol_reached3d = std_max(self.my_tol_reached3d, tol3d);
            self.my_tol_reached2d = std_max(self.my_tol_reached2d, tol2d);
        }
        Ok(())
    }

    /// `buildCurve(theline, thePtrSVSurf)`: the approximation of each knot interval,
    /// its curves moved back by the offsets and, for Bezier pieces, joined into a
    /// B-spline at the end. A knot interval without a Bezier piece stops everything.
    pub(super) fn build_curve(
        &mut self,
        line: &Handle<B::Line>,
        sv: Option<SvSurfacesRef>,
    ) -> Result<(), Failure> {
        if self.my_data.bezier_approx {
            self.my_bez_to_bspl.reset();
        }
        let mut kind: usize = 0;
        loop {
            let imin: i32 = knot_at(&self.my_knots, kind)?;
            let imax: i32 = knot_at(&self.my_knots, kind + 1)?;
            let multi_line = self.multi_line(line, sv.clone(), (imin, imax));
            if self.my_data.bezier_approx {
                self.my_compute_line_bezier.perform(&multi_line)?;
                if self.my_compute_line_bezier.nb_multi_curves() == 0 {
                    return Ok(());
                }
            } else {
                self.my_compute_line.perform(&multi_line)?;
            }
            self.update_tol_reached()?;
            self.translate_back()?;
            if !self.my_data.bezier_approx {
                break;
            }
            for nbmc in 1..=self.my_compute_line_bezier.nb_multi_curves() {
                let curve = self.my_compute_line_bezier.value(nbmc)?.clone();
                self.my_bez_to_bspl.append(&curve);
            }
            kind += 1;
            if kind >= self.my_knots.len() - 1 {
                break;
            }
        }
        if self.my_data.bezier_approx {
            self.my_bez_to_bspl.perform()?;
        }
        Ok(())
    }

    /// The transformations that move the curves of the last knot interval back by the
    /// offsets (the curves are numbered 3d, then first then second parameter curve).
    fn translate_back(&mut self) -> Result<(), Failure> {
        let d = self.my_data;
        let o = d.offsets;
        let indice3d: i32 = 1;
        let mut indice2d1: i32 = 2;
        let mut indice2d2: i32 = 3;
        if !d.approx_xyz {
            indice2d1 -= 1;
            indice2d2 -= 1;
        }
        if !d.approx_u1v1 {
            indice2d2 -= 1;
        }
        if d.approx_xyz {
            self.transform(|c| c.transform(indice3d, [-o.xo, 1.0, -o.yo, 1.0, -o.zo, 1.0]))?;
        }
        if d.approx_u1v1 {
            self.transform(|c| c.transform2d(indice2d1, [-o.u1o, 1.0, -o.v1o, 1.0]))?;
        }
        if d.approx_u2v2 {
            self.transform(|c| c.transform2d(indice2d2, [-o.u2o, 1.0, -o.v2o, 1.0]))?;
        }
        Ok(())
    }

    /// Applies `edit` to the Bezier pieces (from the last one) or to the B-spline.
    fn transform(
        &mut self,
        edit: impl Fn(&rustycad_tkgeombase::app_par_curves::MultiCurve) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        if !self.my_data.bezier_approx {
            return edit(self.my_compute_line.change_value());
        }
        for nbmc in (1..=self.my_compute_line_bezier.nb_multi_curves()).rev() {
            edit(self.my_compute_line_bezier.change_value(nbmc)?)?;
        }
        Ok(())
    }
}

/// The knot of 0-based rank `rank` (`myKnots(kind)`, Standard_OutOfRange past the
/// end).
fn knot_at(knots: &[i32], rank: usize) -> Result<i32, Failure> {
    knots.get(rank).copied().ok_or_else(|| {
        Failure::with_message(
            "Standard_OutOfRange",
            &format!(
                "NCollection_DynamicArray::Value: index {rank} outside 0..{}",
                knots.len()
            ),
        )
    })
}

/// The line of a handle, or Standard_NullObject where the native code dereferences a
/// null one.
fn line_ref<L: ApproxIntLine>(line: &Handle<L>) -> Result<&L, Failure> {
    line.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ApproxInt_Approx::fillData() - the line handle is null; expected a line",
        )
    })
}
