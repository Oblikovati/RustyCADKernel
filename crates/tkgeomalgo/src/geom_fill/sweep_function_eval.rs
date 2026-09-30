// Rust translation of the Approx_SweepFunction members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SweepFunction.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluators and the queries of `GeomFill_SweepFunction`. The native file
//! disables the index checks of its arrays; here an index outside an array raises
//! Standard_OutOfRange.
use super::location_law::{LocationD0, LocationD1, LocationD2, LocationPoles2d};
use super::section_law::{SectionD1, SectionD2};
use super::sweep_function::{GeomFillSweepFunction, index_error, place_pole, place_poles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2, SweepFunction};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector3, Xyz};

fn xyz_at(array: &Array1<'_, Vector3>, name: &str, ii: i32) -> Result<Xyz, Failure> {
    Ok(*array.value(ii).map_err(|e| index_error(name, ii, e))?.xyz())
}

fn point_at(poles: &Array1<'_, Point3>, ii: i32) -> Result<Xyz, Failure> {
    Ok(poles
        .value(ii)
        .map_err(|e| index_error("Poles", ii, e))?
        .xyz())
}

fn scale_at(array: &mut Array1<'_, f64>, name: &str, ii: i32, factor: f64) -> Result<(), Failure> {
    *array
        .change_value(ii)
        .map_err(|e| index_error(name, ii, e))? *= factor;
    Ok(())
}

fn store_vec(array: &mut Array1<'_, Vector3>, name: &str, ii: i32, v: Xyz) -> Result<(), Failure> {
    *array
        .change_value(ii)
        .map_err(|e| index_error(name, ii, e))? = Vector3::from(v);
    Ok(())
}

impl GeomFillSweepFunction {
    /// The first derivative of pole `ii`: `(DP * ratio) * M + DM * P + DV`.
    fn d1_pole(
        &self,
        p: &LocationD1,
        values: &mut SweepD1<'_, '_>,
        ii: i32,
    ) -> Result<(), Failure> {
        let mut pprim = xyz_at(values.d_poles, "DPoles", ii)?;
        let pole = point_at(values.poles, ii)?;
        pprim *= self.my_ratio;
        scale_at(values.d_weights, "DWeigths", ii, self.my_ratio)?;
        pprim *= p.m;
        pprim += p.dm * pole;
        pprim += *p.dv.xyz();
        store_vec(values.d_poles, "DPoles", ii, pprim)
    }

    /// The first and second derivatives of pole `ii`, in the native order.
    fn d2_pole(
        &self,
        p: &LocationD2,
        values: &mut SweepD2<'_, '_>,
        ii: i32,
    ) -> Result<(), Failure> {
        let squareratio = self.my_ratio * self.my_ratio;
        let mut psecn = xyz_at(values.d2_poles, "D2Poles", ii)?;
        let mut pprim = xyz_at(values.d1.d_poles, "DPoles", ii)?;
        pprim *= self.my_ratio;
        scale_at(values.d1.d_weights, "DWeigths", ii, self.my_ratio)?;
        psecn *= squareratio;
        scale_at(values.d2_weights, "D2Weigths", ii, squareratio)?;
        let pole = point_at(values.d1.poles, ii)?;
        psecn *= p.m;
        psecn += 2.0 * (p.dm * pprim);
        psecn += p.d2m * pole;
        psecn += *p.d2v.xyz();
        store_vec(values.d2_poles, "D2Poles", ii, psecn)?;
        pprim *= p.m;
        pprim += p.dm * pole;
        pprim += *p.dv.xyz();
        store_vec(values.d1.d_poles, "DPoles", ii, pprim)
    }

    fn stored_d1(&self) -> LocationD1 {
        let s = self.placement.read();
        LocationD1 {
            m: s.m,
            v: s.v,
            dm: s.dm,
            dv: s.dv,
        }
    }

    fn store_d1(&self, p: &LocationD1) {
        let mut s = self.placement.write();
        (s.m, s.v, s.dm, s.dv) = (p.m, p.v, p.dm, p.dv);
    }
}

impl SweepFunction for GeomFillSweepFunction {
    /// `D0`: the placed section poles; false when a law fails.
    fn d0(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let t = self.section_parameter(param);
        let l = poles.len() as i32;
        let mut p = {
            let s = self.placement.read();
            LocationD0 { m: s.m, v: s.v }
        };
        let ok = self.loc()?.d0_poles2d(param, &mut p, poles2d)?;
        {
            let mut s = self.placement.write();
            (s.m, s.v) = (p.m, p.v);
        }
        if !ok || !self.sec()?.d0(t, poles, weights)? {
            return Ok(false);
        }
        place_poles(poles, l, &p)?;
        Ok(true)
    }

    /// `D1`: with the first derivatives (the section ones scaled by the ratio).
    fn d1(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        mut values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let t = self.section_parameter(param);
        let l = values.poles.len() as i32;
        let mut p = self.stored_d1();
        let ok = self
            .loc()?
            .d1(param, &mut p, values.poles2d, values.d_poles2d)?;
        self.store_d1(&p);
        if !ok || !self.sec()?.d1(t, self.section_d1(&mut values))? {
            return Ok(false);
        }
        for ii in 1..=l {
            self.d1_pole(&p, &mut values, ii)?;
            place_pole(values.poles, ii, &p.m, p.v)?;
        }
        Ok(true)
    }

    /// `D2`: with the second derivatives (scaled by the squared ratio).
    fn d2(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        mut values: SweepD2<'_, '_>,
    ) -> Result<bool, Failure> {
        let t = self.section_parameter(param);
        let l = values.d1.poles.len() as i32;
        let mut p = *self.placement.read();
        let poles2d = LocationPoles2d {
            poles2d: values.d1.poles2d,
            d_poles2d: values.d1.d_poles2d,
            d2_poles2d: values.d2_poles2d,
        };
        let ok = self.loc()?.d2(param, &mut p, poles2d)?;
        *self.placement.write() = p;
        if !ok || !self.sec()?.d2(t, self.section_d2(&mut values))? {
            return Ok(false);
        }
        for ii in 1..=l {
            self.d2_pole(&p, &mut values, ii)?;
            place_pole(values.d1.poles, ii, &p.m, p.v)?;
        }
        Ok(true)
    }

    fn nb_2d_curves(&self) -> Result<i32, Failure> {
        Ok(self.loc()?.nb_2d_curves())
    }

    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        self.sec()?.section_shape()
    }

    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.sec()?.knots(knots)
    }

    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.sec()?.mults(mults)
    }

    fn is_rational(&self) -> Result<bool, Failure> {
        self.sec()?.is_rational()
    }

    /// `NbIntervals(S)`: the path intervals fused with the section ones.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let nb_sec = self.sec()?.nb_intervals(shape)?;
        let nb_loc = self.loc()?.nb_intervals(shape)?;
        if nb_sec == 1 {
            return Ok(nb_loc);
        }
        if nb_loc == 1 {
            return Ok(nb_sec);
        }
        Ok(self.fused_intervals(nb_sec, nb_loc, shape)?.len() as i32 - 1)
    }

    /// `Intervals(T, S)`: their bounds, from the first index of `t` on.
    fn intervals(&self, t: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let nb_sec = self.sec()?.nb_intervals(shape)?;
        let nb_loc = self.loc()?.nb_intervals(shape)?;
        if nb_sec == 1 {
            return self.loc()?.intervals(t, shape);
        }
        if nb_loc == 1 {
            self.sec()?.intervals(t, shape)?;
            for ii in 1..=nb_sec + 1 {
                let bound = t.change_value(ii).map_err(|e| index_error("T", ii, e))?;
                *bound = self.path_parameter(*bound);
            }
            return Ok(());
        }
        let inter = self.fused_intervals(nb_sec, nb_loc, shape)?;
        for (index, value) in inter.as_slice().iter().enumerate() {
            *t.change_at(index)
                .map_err(|e| index_error("T", index as i32, e))? = *value;
        }
        Ok(())
    }

    /// `SetInterval(First, Last)`: the path interval and its section image (from
    /// `myf`, as natively, not from `myfOnS`).
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.loc()?.set_interval(first, last)?;
        let uf = self.myf + (first - self.myf) * self.my_ratio;
        let ul = self.myf + (last - self.myf) * self.my_ratio;
        self.sec()?.set_interval(uf, ul)
    }

    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        self.loc()?.resolution(index, tol)
    }

    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        self.sec()?
            .get_tolerance(bound_tol, surf_tol, angle_tol, tol3d)
    }

    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
        self.sec()?.set_tolerance(tol3d, tol2d)?;
        self.loc()?.set_tolerance(tol3d, tol2d)
    }

    /// The section barycentre moved by the mean placement of the path.
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        let mut bary = self.sec()?.barycentre_of_surf()?.xyz();
        let average = self.loc()?.get_average_law()?;
        bary *= average.m;
        bary += *average.v.xyz();
        Ok(Point3::from(bary))
    }

    fn maximal_section(&self) -> Result<f64, Failure> {
        let l = self.sec()?.maximal_section()?;
        Ok(l * self.loc()?.get_maximal_norm()?)
    }

    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.sec()?.get_minimal_weight(weights)
    }
}

impl GeomFillSweepFunction {
    fn section_d1<'a, 'b>(&self, values: &'a mut SweepD1<'_, 'b>) -> SectionD1<'a, 'b> {
        SectionD1 {
            poles: values.poles,
            d_poles: values.d_poles,
            weights: values.weights,
            d_weights: values.d_weights,
        }
    }

    fn section_d2<'a, 'b>(&self, values: &'a mut SweepD2<'_, 'b>) -> SectionD2<'a, 'b> {
        SectionD2 {
            d1: self.section_d1(&mut values.d1),
            d2_poles: values.d2_poles,
            d2_weights: values.d2_weights,
        }
    }
}
