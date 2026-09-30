// Rust translation of EcartContraintesMil and G0Error/G1Error/G2Error(Index) of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gaps between the plate surface and a curve constraint at the middles of its
//! discretisation intervals.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::live;
use crate::local_analysis::{LocalAnalysisSurfaceContinuity, SurfaceContinuityTolerances};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// The distance, angle and curvature gaps at the middles of the intervals, 1-based
/// (`d`, `an`, `courb` of `EcartContraintesMil`).
pub(super) struct ConstraintGaps {
    pub(super) d: Array1<'static, f64>,
    pub(super) an: Array1<'static, f64>,
    pub(super) courb: Array1<'static, f64>,
}

impl ConstraintGaps {
    /// Three zeroed arrays of `n` values (`new NCollection_HArray1<double>(1, n)`,
    /// cleared by the default memory manager).
    pub(super) fn zeroed(n: i32) -> Result<Self, Failure> {
        let make = || Array1::<f64>::with_bounds(1, n).map_err(errors::array_bounds);
        Ok(Self {
            d: make()?,
            an: make()?,
            courb: make()?,
        })
    }
}

/// `array->ChangeValue(i) = value`, Standard_OutOfRange outside it.
fn set(array: &mut Array1<'_, f64>, i: i32, value: f64) -> Result<(), Failure> {
    array.set_value(i, value).map_err(errors::array_bounds)
}

/// The angle between the lines of the two normals, in `[0, PI/2]`.
fn line_angle(
    v3f: rustycad_tkmath::gp::Vector3,
    v3i: rustycad_tkmath::gp::Vector3,
) -> Result<f64, Failure> {
    let angle: f64 = v3f.angle(v3i)?;
    Ok(if angle > PI / 2.0 { PI - angle } else { angle })
}

impl GeomPlateBuildPlateSurface {
    /// The parameters of the discretisation of curve `c` (`myParCont->Value(c)`).
    pub(super) fn par_cont_of(&self, c: i32) -> Result<&[f64], Failure> {
        let par_cont = self.par_cont.as_deref().ok_or_else(|| {
            errors::null_object(
                "GeomPlate_BuildPlateSurface: myParCont is null; expected a performed builder",
            )
        })?;
        let index: usize = super::sequence_index(c, par_cont.len(), "ParCont")?;
        Ok(&par_cont[index])
    }

    /// The middle of interval `i` of `par` (1-based), Standard_OutOfRange past it.
    fn middle(par: &[f64], i: i32) -> Result<f64, Failure> {
        let at = |k: i32| -> Result<f64, Failure> {
            par.get((k - 1) as usize).copied().ok_or_else(|| {
                errors::out_of_range(&format!(
                    "GeomPlate: parameter {k} outside 1..={}",
                    par.len()
                ))
            })
        };
        Ok((at(i)? + at(i + 1)?) / 2.0)
    }

    /// Fills the gaps of curve `c` at the middles of its intervals (at least three
    /// intervals are read, Standard_OutOfRange past the parameters, as natively): the
    /// distance for G0, the distance and the normal angle for G1, and the C0, G1 and
    /// G2 gaps of `LocalAnalysis_SurfaceContinuity` for G2 (`EcartContraintesMil`).
    pub(super) fn ecart_contraintes_mil(
        &mut self,
        c: i32,
        gaps: &mut ConstraintGaps,
    ) -> Result<(), Failure> {
        let par: Vec<f64> = self.par_cont_of(c)?.to_vec();
        let nb_pt: i32 = if par.len() < 3 { 4 } else { par.len() as i32 };
        let lin = self.lin_cont[super::sequence_index(c, self.lin_cont.len(), "LinCont")?].clone();
        let lin_cont = live(&lin, "a curve constraint")?;
        match lin_cont.order() {
            0 => self.g0_gaps(lin_cont, &par, nb_pt, gaps),
            1 => self.g1_gaps(lin_cont, &par, nb_pt, gaps),
            2 => self.g2_gaps(lin_cont, &par, nb_pt, gaps),
            _ => Ok(()),
        }
    }

    fn plate_surface(&self) -> Result<&dyn GeomSurface, Failure> {
        Ok(live(&self.geom_plate_surface, "the plate surface")?)
    }

    fn g0_gaps(
        &mut self,
        lin: &crate::geom_plate::GeomPlateCurveConstraint,
        par: &[f64],
        nb_pt: i32,
        gaps: &mut ConstraintGaps,
    ) -> Result<(), Failure> {
        for i in 1..nb_pt {
            let u: f64 = Self::middle(par, i)?;
            let pi: Point3 = lin.d0(u)?;
            let p2d: Point2 = self.uv_on_init(lin, u, || Ok(pi))?;
            let pf: Point3 = self.plate_surface()?.d0(p2d.x(), p2d.y())?;
            gaps.an.init(0.0);
            gaps.courb.init(0.0);
            set(&mut gaps.d, i, pf.distance(&pi))?;
        }
        Ok(())
    }

    fn g1_gaps(
        &mut self,
        lin: &crate::geom_plate::GeomPlateCurveConstraint,
        par: &[f64],
        nb_pt: i32,
        gaps: &mut ConstraintGaps,
    ) -> Result<(), Failure> {
        for i in 1..nb_pt {
            let u: f64 = Self::middle(par, i)?;
            let di = lin.d1(u)?;
            let p2d: Point2 = self.uv_on_init(lin, u, || Ok(di.point))?;
            let df = self.plate_surface()?.d1(p2d.x(), p2d.y())?;
            set(&mut gaps.d, i, df.point.distance(&di.point))?;
            let angle: f64 = line_angle(df.d1u ^ df.d1v, di.d1u ^ di.d1v)?;
            set(&mut gaps.an, i, angle)?;
            gaps.courb.init(0.0);
        }
        Ok(())
    }

    fn g2_gaps(
        &mut self,
        lin: &crate::geom_plate::GeomPlateCurveConstraint,
        par: &[f64],
        nb_pt: i32,
        gaps: &mut ConstraintGaps,
    ) -> Result<(), Failure> {
        let splate: Handle<dyn GeomSurface> = self.geom_plate_surface.cast();
        let mut cg2 = LocalAnalysisSurfaceContinuity::with_tolerances(
            &SurfaceContinuityTolerances::default(),
        );
        for i in 1..nb_pt {
            let u: f64 = Self::middle(par, i)?;
            let pi: Point3 = lin.d0(u)?;
            let p2d: Point2 = self.uv_on_init(lin, u, || Ok(pi))?;
            let mut prop = SLProps::with_parameters(splate.clone(), p2d.x(), p2d.y(), 2, 0.001)?;
            cg2.compute_analysis(&mut prop, &mut *lin.lprop_surf(u)?, Shape::G2)?;
            set(&mut gaps.d, i, cg2.c0_value()?)?;
            set(&mut gaps.an, i, cg2.g1_angle()?)?;
            set(&mut gaps.courb, i, cg2.g2_curvature_gap()?)?;
        }
        Ok(())
    }

    /// The greatest of the first `myNbPtsOnCur` values of one gap of curve `index`.
    fn max_gap(
        &mut self,
        index: i32,
        pick: fn(&ConstraintGaps) -> &Array1<'static, f64>,
    ) -> Result<f64, Failure> {
        let mut gaps = ConstraintGaps::zeroed(self.nb_pts_on_cur)?;
        self.ecart_contraintes_mil(index, &mut gaps)?;
        let mut max: f64 = 0.0;
        for i in 1..=self.nb_pts_on_cur {
            let value: f64 = *pick(&gaps).value(i).map_err(errors::array_bounds)?;
            if value > max {
                max = value;
            }
        }
        Ok(max)
    }

    /// The greatest distance between the plate and the curve constraint `index` at
    /// the middles of its intervals (`G0Error(Index)`). The gaps go to arrays of
    /// `NbPtsOnCur` values, so a curve with more intervals raises Standard_OutOfRange
    /// as natively. `builder.g0_error_of(1)?`.
    pub fn g0_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.max_gap(index, |g| &g.d)
    }

    /// The greatest normal angle to the curve constraint `index` (`G1Error(Index)`);
    /// `builder.g1_error_of(1)?`.
    pub fn g1_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.max_gap(index, |g| &g.an)
    }

    /// The greatest curvature gap to the curve constraint `index`
    /// (`G2Error(Index)`); `builder.g2_error_of(1)?`.
    pub fn g2_error_of(&mut self, index: i32) -> Result<f64, Failure> {
        self.max_gap(index, |g| &g.courb)
    }
}
