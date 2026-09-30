// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SameParameter.cxx (BuildInitialDistribution, IncreaseInitialNbSamples, CheckSameParameter, ComputeTangents)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The samples of the pcurve and their parameters on the 3D curve.
use super::fit::project_point_on_curve;
use super::{MAX_ARRAY_SIZE, NB_SAMPLES, SameParameter, SameParameterData, at, put};
use crate::extrema::{ExtPC, LocateExtPC};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `aSmallMagnitude`: the smallest pcurve speed of the end tangents.
const SMALL_MAGNITUDE: f64 = 1.0e-12;
/// `ProjectPointOnCurve` iterations.
const PROJECTION_ITERATIONS: i32 = 30;

impl SameParameter {
    /// `BuildInitialDistribution`: `NbSamples` equal steps on both curves, refined on
    /// the C1 breaks of a C0 pcurve; false when there are too many points.
    pub(super) fn build_initial_distribution(
        &self,
        data: &mut SameParameterData,
    ) -> Result<bool, Failure> {
        let deltacons: f64 = (data.c2d_pl - data.c2d_pf) / f64::from(NB_SAMPLES);
        let deltac3d: f64 = (data.c3d_pl - data.c3d_pf) / f64::from(NB_SAMPLES);
        let (mut wcons, mut wc3d) = (data.c2d_pf, data.c3d_pf);
        for ii in 0..NB_SAMPLES {
            put(&mut data.pc2d, ii, wcons)?;
            put(&mut data.pc3d, ii, wc3d)?;
            wcons += deltacons;
            wc3d += deltac3d;
        }
        data.nb_pnt = NB_SAMPLES;
        put(&mut data.pc2d, data.nb_pnt, data.c2d_pl)?;
        put(&mut data.pc3d, data.nb_pnt, data.c3d_pl)?;
        if self.hcurve_2d()?.continuity()? < Shape::C1 {
            return self.increase_initial_nb_samples(data);
        }
        Ok(true)
    }

    /// `IncreaseInitialNbSamples`: the C1 breaks of the pcurve (inside the 3D range)
    /// merged into the samples, which take the 3D parameters of the breaks.
    fn increase_initial_nb_samples(&self, data: &mut SameParameterData) -> Result<bool, Failure> {
        let hcurve = self.hcurve_2d()?;
        let mut nb_int: i32 = hcurve.nb_intervals(Shape::C1)? + 1;
        let mut breaks: Array1<'static, f64> =
            Array1::with_bounds(1, nb_int).map_err(|e| array_failure(e, "Approx_SameParameter"))?;
        hcurve.intervals(&mut breaks, Shape::C1)?;
        let brk = |i: i32| {
            breaks
                .value(i)
                .copied()
                .map_err(|e| array_failure(e, "Approx_SameParameter"))
        };
        let mut inter: i32 = 1;
        while inter <= nb_int && brk(inter)? <= data.c3d_pf + self.delta_min {
            inter += 1;
        }
        while nb_int > 0 && brk(nb_int)? >= data.c3d_pl - self.delta_min {
            nb_int -= 1;
        }
        let mut new_par: Vec<f64> = vec![data.c3d_pf];
        let mut ii: i32 = 1;
        while inter <= nb_int || (ii < NB_SAMPLES && inter <= breaks.length()) {
            let (b, p) = (brk(inter)?, at(&data.pc2d, ii)?);
            if b < p {
                new_par.push(b);
                if (p - b) <= self.delta_min {
                    ii = (ii + 1).min(NB_SAMPLES);
                }
                inter += 1;
            } else {
                if (b - p) > self.delta_min {
                    new_par.push(p);
                }
                ii += 1;
            }
        }
        data.nb_pnt = new_par.len() as i32;
        if data.nb_pnt > MAX_ARRAY_SIZE - 1 {
            return Ok(false);
        }
        for ii in 1..data.nb_pnt {
            let value: f64 = new_par[ii as usize];
            put(&mut data.pc2d, ii, value)?;
            put(&mut data.pc3d, ii, value)?;
        }
        put(&mut data.pc3d, data.nb_pnt, data.c3d_pl)?;
        put(&mut data.pc2d, data.nb_pnt, data.c2d_pl)?;
        Ok(true)
    }

    /// `CheckSameParameter(theData, theSqDist)`: keeps the samples whose points are
    /// within the tolerance at the same parameter, and projects the others on the 3D
    /// curve (local extremum, Newton iterations, then global extrema); returns
    /// whether every sample was within the tolerance, and the largest squared
    /// distance of the kept ones.
    pub(super) fn check_same_parameter(
        &self,
        data: &mut SameParameterData,
    ) -> Result<(bool, f64), Failure> {
        let tol2: f64 = data.tol * data.tol;
        let c3d = self.c3d()?;
        let mut dmax2: f64 = data
            .cons
            .value(data.c2d_pf)?
            .square_distance(&c3d.value(data.c3d_pf)?);
        let dist2: f64 = data
            .cons
            .value(data.c2d_pl)?
            .square_distance(&c3d.value(data.c3d_pl)?);
        dmax2 = dmax2.max_std(dist2);
        let mut projector: LocateExtPC<'_> = LocateExtPC::new();
        projector.initialize(c3d, (data.c3d_pf, data.c3d_pl), data.tol)?;
        let mut state = ProjectionState {
            count: 1,
            previousp: data.c3d_pf,
            initp: 0.0,
            is_proj_ok: false,
            is_same_param: true,
        };
        let bornesup: f64 = data.c3d_pl - self.delta_min;
        for ii in 1..data.nb_pnt {
            let pcons: Point3 = data.cons.value(at(&data.pc2d, ii)?)?;
            let pc3d: Point3 = c3d.value(at(&data.pc3d, ii)?)?;
            let dist2: f64 = pcons.square_distance(&pc3d);
            let separated: bool =
                at(&data.pc3d, ii)? > at(&data.pc3d, state.count - 1)? + self.delta_min;
            if dist2 <= tol2 && separated {
                if dmax2 < dist2 {
                    dmax2 = dist2;
                }
                let p3: f64 = at(&data.pc3d, ii)?;
                state.accept(data, ii, p3)?;
                continue;
            }
            self.project_sample(data, &mut projector, &mut state, (ii, pcons), bornesup)?;
        }
        data.nb_pnt = state.count;
        put(&mut data.pc2d, data.nb_pnt, data.c2d_pl)?;
        put(&mut data.pc3d, data.nb_pnt, data.c3d_pl)?;
        Ok((state.is_same_param, dmax2))
    }

    // The projection of the pcurve point of sample `ii` on the 3D curve.
    fn project_sample(
        &self,
        data: &mut SameParameterData,
        projector: &mut LocateExtPC<'_>,
        state: &mut ProjectionState,
        (ii, pcons): (i32, Point3),
        bornesup: f64,
    ) -> Result<(), Failure> {
        if !state.is_proj_ok {
            state.initp = at(&data.pc3d, ii)?;
        }
        state.is_same_param = false;
        projector.perform(pcons, state.initp)?;
        let (mut found, curp): (bool, f64) = if projector.is_done() {
            (true, projector.point()?.parameter())
        } else {
            project_point_on_curve(
                state.initp,
                pcons,
                data.tol,
                PROJECTION_ITERATIONS,
                self.c3d()?,
            )?
        };
        found = found && curp > state.previousp + self.delta_min && curp < bornesup;
        state.is_proj_ok = found;
        if found {
            return state.accept(data, ii, curp);
        }
        self.global_projection(data, state, (ii, pcons), bornesup)
    }

    // "Whole parameter space search using general extrema": the nearest extremum.
    fn global_projection(
        &self,
        data: &mut SameParameterData,
        state: &mut ProjectionState,
        (ii, pcons): (i32, Point3),
        bornesup: f64,
    ) -> Result<(), Failure> {
        let pr = ExtPC::with_range(pcons, self.c3d()?, data.c3d_pf, data.c3d_pl, data.tol)?;
        if !pr.is_done() || pr.nb_ext()? == 0 {
            return Ok(());
        }
        let (mut ind_min, mut cur_dist_min): (i32, f64) = (0, real_last());
        for i in 1..=pr.nb_ext()? {
            let dist2: f64 = pr.point(i)?.value().square_distance(&pcons);
            if dist2 < cur_dist_min {
                (cur_dist_min, ind_min) = (dist2, i);
            }
        }
        if ind_min == 0 {
            return Ok(());
        }
        let curp: f64 = pr.point(ind_min)?.parameter();
        if curp > state.previousp + self.delta_min && curp < bornesup {
            state.accept(data, ii, curp)?;
            state.is_proj_ok = true;
        }
        Ok(())
    }

    /// `ComputeTangents(theCOnS, theFirstTangent, theLastTangent)`: the ratios of the
    /// 3D speed to the curve on surface speed at both ends; None when a curve on
    /// surface speed is under 1e-12.
    pub(super) fn compute_tangents(
        &self,
        data: &SameParameterData,
    ) -> Result<Option<[f64; 2]>, Failure> {
        let c3d = self.c3d()?;
        let mut tangents: [f64; 2] = [0.0; 2];
        for (slot, param) in [c3d.first_parameter()?, c3d.last_parameter()?]
            .into_iter()
            .enumerate()
        {
            let (_, vec_cons): (Point3, Vector3) = data.cons.d1(param)?;
            let (_, vec): (Point3, Vector3) = c3d.d1(param)?;
            let magnitude: f64 = vec_cons.magnitude();
            if magnitude <= SMALL_MAGNITUDE {
                return Ok(None);
            }
            tangents[slot] = vec.magnitude() / magnitude;
        }
        Ok(Some(tangents))
    }
}

/// `std::max` on reals: the first unless the second is greater.
trait StdMax {
    fn max_std(self, other: f64) -> f64;
}

impl StdMax for f64 {
    fn max_std(self, other: f64) -> f64 {
        if self < other { other } else { self }
    }
}

/// The locals of the CheckSameParameter loop.
struct ProjectionState {
    count: i32,
    previousp: f64,
    initp: f64,
    is_proj_ok: bool,
    is_same_param: bool,
}

impl ProjectionState {
    // Keeps sample `ii` with the 3D parameter `p3`.
    fn accept(&mut self, data: &mut SameParameterData, ii: i32, p3: f64) -> Result<(), Failure> {
        (self.initp, self.previousp) = (p3, p3);
        put(&mut data.pc3d, self.count, p3)?;
        let p2: f64 = at(&data.pc2d, ii)?;
        put(&mut data.pc2d, self.count, p2)?;
        self.count += 1;
        Ok(())
    }
}
