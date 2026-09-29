// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntCurve_UserIntConicCurveGen.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_UserIntConicCurveGen`: a gp conic against a parametric curve, handled
//! as a conic pair when the curve is a conic and per continuity interval when it is
//! composite.
use super::{GpConic, IConicTool, IntConicConic, IntConicCurveGen, PCurveTool};
use crate::int_imp_par_gen::{IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, Intersection};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};
use std::ops::{Deref, DerefMut};

/// The parametric curve of an instantiation.
type PCurveOf<T> = <<T as IntersectorTypes>::Par as ParTool>::Curve;

/// `IntCurve_UserIntConicCurveGen` (`Geom2dInt_IntConicCurveOfGInter`): an
/// [`Intersection`] reached through `Deref`. A conic second curve goes to
/// [`IntConicConic`] (its parameters swapped when its kind comes first in the pair
/// order); any other curve to [`IntConicCurveGen`], per C1 interval (merged by
/// `Append`) when it has several.
/// `Geom2dInt_IntConicCurveOfGInter::line(l, &d1, &adaptor, &d2, 1e-9, 1e-9)?`.
pub struct UserIntConicCurveGen<T: IntersectorTypes<Imp = IConicTool>>
where
    T::Par: PCurveTool,
{
    base: Intersection,
    param1inf: f64,
    param1sup: f64,
    param2inf: f64,
    param2sup: f64,
    intconiconi: IntConicConic,
    intconicurv: IntConicCurveGen<T>,
}

impl<T: IntersectorTypes<Imp = IConicTool>> Default for UserIntConicCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_UserIntConicCurveGen()`: not done; the composite bounds start at 0
    /// (uninitialised natively, and only read after they are set).
    fn default() -> Self {
        Self {
            base: Intersection::new(),
            param1inf: 0.0,
            param1sup: 0.0,
            param2inf: 0.0,
            param2sup: 0.0,
            intconiconi: IntConicConic::new(),
            intconicurv: IntConicCurveGen::new(),
        }
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> Deref for UserIntConicCurveGen<T>
where
    T::Par: PCurveTool,
{
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> DerefMut for UserIntConicCurveGen<T>
where
    T::Par: PCurveTool,
{
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

// One named constructor and one Perform per conic kind.
macro_rules! user_conic_members {
    ($(#[$doc:meta])* $name:ident, $perform:ident, $conic:ty, $variant:ident) => {
        $(#[$doc])*
        pub fn $name(
            conic: $conic,
            d1: &Domain,
            c2: &PCurveOf<T>,
            d2: &Domain,
            tol_conf: f64,
            tol: f64,
        ) -> Result<Self, Failure> {
            let mut inter: Self = Self::new();
            inter.$perform(conic, d1, c2, d2, tol_conf, tol)?;
            Ok(inter)
        }
        /// The Perform overload of the same conic kind.
        pub fn $perform(
            &mut self,
            conic: $conic,
            d1: &Domain,
            c2: &PCurveOf<T>,
            d2: &Domain,
            tol_conf: f64,
            tol: f64,
        ) -> Result<(), Failure> {
            self.perform_conic(&GpConic::$variant(conic), d1, c2, d2, (tol_conf, tol))
        }
    };
}

impl<T: IntersectorTypes<Imp = IConicTool>> UserIntConicCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_UserIntConicCurveGen()`. `Geom2dInt_IntConicCurveOfGInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    user_conic_members!(
        /// `IntCurve_UserIntConicCurveGen(gp_Lin2d, D1, C2, D2, TolConf, Tol)`.
        line, perform_line, Line2, Line
    );
    user_conic_members!(
        /// `IntCurve_UserIntConicCurveGen(gp_Circ2d, D1, C2, D2, TolConf, Tol)`.
        circle, perform_circle, Circle2, Circle
    );
    user_conic_members!(
        /// `IntCurve_UserIntConicCurveGen(gp_Elips2d, D1, C2, D2, TolConf, Tol)`.
        ellipse, perform_ellipse, Ellipse2, Ellipse
    );
    user_conic_members!(
        /// `IntCurve_UserIntConicCurveGen(gp_Parab2d, D1, C2, D2, TolConf, Tol)`.
        parabola, perform_parabola, Parabola2, Parabola
    );
    user_conic_members!(
        /// `IntCurve_UserIntConicCurveGen(gp_Hypr2d, D1, C2, D2, TolConf, Tol)`: the
        /// main branch of the hyperbola.
        hyperbola, perform_hyperbola, Hyperbola2, Hyperbola
    );

    // The Perform of every conic kind: the C1 intervals of the curve clipped by its
    // domain (Standard_DomainError when unbounded) when it has several, the whole
    // domain otherwise.
    fn perform_conic(
        &mut self,
        conic1: &GpConic,
        d1: &Domain,
        c2: &PCurveOf<T>,
        d2: &Domain,
        tols: (f64, f64),
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let nb_inter_c2: i32 = T::Par::nb_intervals(c2)?;
        if nb_inter_c2 <= 1 {
            return self.internal_perform(conic1, d1, c2, d2, tols, false);
        }
        let (d2_first_param, d2_last_param) = (d2.first_parameter()?, d2.last_parameter()?);
        self.param1inf = if d1.has_first_point() {
            d1.first_parameter()?
        } else {
            -precision::infinite()
        };
        self.param1sup = if d1.has_last_point() {
            d1.last_parameter()?
        } else {
            precision::infinite()
        };
        self.param2inf = T::Par::first_parameter(c2)?;
        self.param2sup = T::Par::last_parameter(c2)?;
        let mut tab2: Array1<'static, f64> = Array1::with_bounds(1, nb_inter_c2 + 1)
            .map_err(|e| e.into_failure("IntCurve_UserIntConicCurveGen::Perform"))?;
        T::Par::intervals(c2, &mut tab2)?;
        for num_inter_c2 in 1..=nb_inter_c2 {
            let (mut param_inf, mut param_sup) = T::Par::get_interval(c2, num_inter_c2, &tab2)?;
            if (param_inf > d2_last_param) || (param_sup < d2_first_param) {
                break;
            }
            if param_inf < d2_first_param {
                param_inf = d2_first_param;
            }
            if param_sup > d2_last_param {
                param_sup = d2_last_param;
            }
            if (param_sup - param_inf) > f64::EPSILON {
                let domain_c2_num_inter = Domain::bounded(
                    T::Par::value(c2, param_inf)?,
                    param_inf,
                    d2.first_tolerance()?,
                    T::Par::value(c2, param_sup)?,
                    param_sup,
                    d2.last_tolerance()?,
                );
                self.internal_perform(conic1, d1, c2, &domain_c2_num_inter, tols, true)?;
            }
        }
        Ok(())
    }

    // `InternalPerform(Conic1, D1, C2, D2, TolConf, Tol, Composite)`: a conic curve
    // through IntConicConic in pair order, any other through IntConicCurveGen; the
    // result appended (composite) or copied.
    fn internal_perform(
        &mut self,
        conic1: &GpConic,
        d1: &Domain,
        c2: &PCurveOf<T>,
        d2: &Domain,
        (tol_conf, tol): (f64, f64),
        composite: bool,
    ) -> Result<(), Failure> {
        let Some(conic2) = GpConic::of_curve::<T::Par>(c2)? else {
            return self.internal_perform_curve(conic1, d1, c2, d2, (tol_conf, tol), composite);
        };
        if conic2.rank() < conic1.rank() {
            self.intconiconi.set_reversed_parameters(true);
            self.intconiconi
                .perform_ordered(&conic2, d2, conic1, d1, tol_conf, tol)?;
        } else {
            self.intconiconi.set_reversed_parameters(false);
            self.intconiconi
                .perform_ordered(conic1, d1, &conic2, d2, tol_conf, tol)?;
        }
        let bounds = [
            self.param1inf,
            self.param1sup,
            self.param2inf,
            self.param2sup,
        ];
        merge_result(&mut self.base, &self.intconiconi, bounds, composite)
    }

    fn internal_perform_curve(
        &mut self,
        conic1: &GpConic,
        d1: &Domain,
        c2: &PCurveOf<T>,
        d2: &Domain,
        (tol_conf, tol): (f64, f64),
        composite: bool,
    ) -> Result<(), Failure> {
        self.intconicurv.set_reversed_parameters(false);
        self.intconicurv
            .perform_any_conic(conic1, d1, c2, d2, tol_conf, tol)?;
        let bounds = [
            self.param1inf,
            self.param1sup,
            self.param2inf,
            self.param2sup,
        ];
        merge_result(&mut self.base, &self.intconicurv, bounds, composite)
    }
}

// `Append(other, param1inf, param1sup, param2inf, param2sup)` for a composite curve,
// `SetValues(other)` otherwise.
pub(super) fn merge_result(
    base: &mut Intersection,
    other: &Intersection,
    [f1, l1, f2, l2]: [f64; 4],
    composite: bool,
) -> Result<(), Failure> {
    if composite {
        return base.append_composite(other, f1, l1, f2, l2);
    }
    base.set_values(other);
    Ok(())
}
