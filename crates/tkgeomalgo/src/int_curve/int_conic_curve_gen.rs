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
// Rust translation of IntCurve_IntConicCurveGen.gxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntConicCurveGen`: a gp conic against a parametric curve through the
//! IntImpParGen intersector, the conic being the implicit curve.
use super::{GpConic, IConicTool};
use crate::int_imp_par_gen::{Intersector, IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, Intersection};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};
use std::f64::consts::PI;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// The parametric curve of an instantiation.
type PCurveOf<T> = <<T as IntersectorTypes>::Par as ParTool>::Curve;

/// `IntCurve_IntConicCurveGen` over an IntImpParGen instantiation whose implicit
/// tool is [`IConicTool`] (`Geom2dInt_TheIntConicCurveOfGInter`): an
/// [`Intersection`] reached through `Deref`. The domain of a circle or an ellipse is
/// closed on `[First, First + 2PI]` when it is not; the parametric domain must be
/// bounded (Standard_ConstructionError). `Geom2dInt_TheIntConicCurveOfGInter::line(l, &d1, &adaptor, &d2, 1e-9, 1e-9)?`.
pub struct IntConicCurveGen<T: IntersectorTypes<Imp = IConicTool>> {
    base: Intersection,
    types: PhantomData<T>,
}

impl<T: IntersectorTypes<Imp = IConicTool>> Default for IntConicCurveGen<T> {
    fn default() -> Self {
        Self {
            base: Intersection::new(),
            types: PhantomData,
        }
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> Clone for IntConicCurveGen<T> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            types: PhantomData,
        }
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> Deref for IntConicCurveGen<T> {
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> DerefMut for IntConicCurveGen<T> {
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

// One named constructor per native constructor over a conic.
macro_rules! conic_curve_constructor {
    ($(#[$doc:meta])* $name:ident, $perform:ident, $conic:ty) => {
        $(#[$doc])*
        pub fn $name(
            conic: $conic,
            d1: &Domain,
            pcurve: &PCurveOf<T>,
            d2: &Domain,
            tol_conf: f64,
            tol: f64,
        ) -> Result<Self, Failure> {
            let mut inter: Self = Self::new();
            inter.$perform(conic, d1, pcurve, d2, tol_conf, tol)?;
            Ok(inter)
        }
    };
}

/// A circle or ellipse domain closed on `[First, First + 2PI]` unless it is closed.
fn closed(d1: &Domain) -> Result<Domain, Failure> {
    if d1.is_closed() {
        return Ok(*d1);
    }
    let mut d: Domain = *d1;
    let first: f64 = d1.first_parameter()?;
    d.set_equivalent_parameters(first, first + PI + PI)?;
    Ok(d)
}

impl<T: IntersectorTypes<Imp = IConicTool>> IntConicCurveGen<T> {
    /// `IntCurve_IntConicCurveGen()`: not done. `Geom2dInt_TheIntConicCurveOfGInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    conic_curve_constructor!(
        /// `IntCurve_IntConicCurveGen(gp_Lin2d, D1, PCurve, D2, TolConf, Tol)`.
        line, perform_line, Line2
    );
    conic_curve_constructor!(
        /// `IntCurve_IntConicCurveGen(gp_Circ2d, D1, PCurve, D2, TolConf, Tol)`.
        circle, perform_circle, Circle2
    );
    conic_curve_constructor!(
        /// `IntCurve_IntConicCurveGen(gp_Elips2d, D1, PCurve, D2, TolConf, Tol)`.
        ellipse, perform_ellipse, Ellipse2
    );
    conic_curve_constructor!(
        /// `IntCurve_IntConicCurveGen(gp_Parab2d, D1, PCurve, D2, TolConf, Tol)`.
        parabola, perform_parabola, Parabola2
    );
    conic_curve_constructor!(
        /// `IntCurve_IntConicCurveGen(gp_Hypr2d, D1, PCurve, D2, TolConf, Tol)`.
        hyperbola, perform_hyperbola, Hyperbola2
    );

    /// `Perform(gp_Lin2d, D1, PCurve, D2, TolConf, Tol)`.
    /// `inter.perform_line(l, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub fn perform_line(
        &mut self,
        l: Line2,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.perform_conic(&IConicTool::from_line(l)?, d1, pcurve, d2, (tol_conf, tol))
    }
    /// `Perform(gp_Circ2d, D1, PCurve, D2, TolConf, Tol)`.
    /// `inter.perform_circle(c, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub fn perform_circle(
        &mut self,
        c: Circle2,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let d: Domain = closed(d1)?;
        self.perform_conic(&IConicTool::from_circle(c), &d, pcurve, d2, (tol_conf, tol))
    }
    /// `Perform(gp_Elips2d, D1, PCurve, D2, TolConf, Tol)`.
    /// `inter.perform_ellipse(e, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub fn perform_ellipse(
        &mut self,
        e: Ellipse2,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let d: Domain = closed(d1)?;
        self.perform_conic(
            &IConicTool::from_ellipse(e),
            &d,
            pcurve,
            d2,
            (tol_conf, tol),
        )
    }
    /// `Perform(gp_Parab2d, D1, PCurve, D2, TolConf, Tol)`.
    /// `inter.perform_parabola(p, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub fn perform_parabola(
        &mut self,
        p: Parabola2,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.perform_conic(
            &IConicTool::from_parabola(p),
            d1,
            pcurve,
            d2,
            (tol_conf, tol),
        )
    }
    /// `Perform(gp_Hypr2d, D1, PCurve, D2, TolConf, Tol)`.
    /// `inter.perform_hyperbola(h, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub fn perform_hyperbola(
        &mut self,
        h: Hyperbola2,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.perform_conic(
            &IConicTool::from_hyperbola(h),
            d1,
            pcurve,
            d2,
            (tol_conf, tol),
        )
    }

    /// The Perform overload of the kind of `conic`, as the curve-curve templates
    /// select it from the curve type. `inter.perform_any_conic(&conic, &d1, &pcurve, &d2, 1e-9, 1e-9)?`.
    pub(crate) fn perform_any_conic(
        &mut self,
        conic: &GpConic,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        match *conic {
            GpConic::Line(l) => self.perform_line(l, d1, pcurve, d2, tol_conf, tol),
            GpConic::Circle(c) => self.perform_circle(c, d1, pcurve, d2, tol_conf, tol),
            GpConic::Ellipse(e) => self.perform_ellipse(e, d1, pcurve, d2, tol_conf, tol),
            GpConic::Parabola(p) => self.perform_parabola(p, d1, pcurve, d2, tol_conf, tol),
            GpConic::Hyperbola(h) => self.perform_hyperbola(h, d1, pcurve, d2, tol_conf, tol),
        }
    }

    // The private `Perform(IntCurve_IConicTool, ...)`: a fresh intersector with the
    // reversed flag of this one, whose result is copied.
    fn perform_conic(
        &mut self,
        icurve: &IConicTool,
        d1: &Domain,
        pcurve: &PCurveOf<T>,
        d2: &Domain,
        (tol_conf, tol): (f64, f64),
    ) -> Result<(), Failure> {
        let mut myintersection: Intersector<T> = Intersector::new();
        myintersection.set_reversed_parameters(self.base.reversed_parameters());
        myintersection.perform(icurve, d1, pcurve, d2, tol_conf, tol)?;
        self.base.set_values(&myintersection);
        Ok(())
    }
}
