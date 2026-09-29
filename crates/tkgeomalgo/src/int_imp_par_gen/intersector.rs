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
// Rust translation of IntImpParGen_Intersector.gxx (constructors, Perform and the
// isolated roots) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImpParGen_Intersector`: the intersection of an implicit curve with a bounded
//! parametric curve, from the roots of the signed distance along the latter.
use super::MyImpParTool;
use super::package::{
    TransitionSide, determine_position, determine_transition, normalize_on_domain,
};
use super::traits::{ImpTool, IntersectorTypes, ParTool};
use crate::int_res2d::{
    Domain, Intersection, IntersectionPoint, Position, Transition, construction_error,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{FunctionAllRoots, FunctionSample};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// The parametric curve of one instantiation.
pub(super) type ParCurveOf<T> = <<T as IntersectorTypes>::Par as ParTool>::Curve;

/// `IntImpParGen_Intersector` over one [`IntersectorTypes`]
/// (`IntCurve_IntImpConicParConic`, `Geom2dInt_TheIntersectorOfTheIntConicCurveOfGInter`):
/// an [`Intersection`] (reached through `Deref`) computed by Perform. The domain of
/// the parametric curve must be bounded (Standard_ConstructionError otherwise); the
/// exceptions of the curves and domains are returned as `Err`.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve::{IConicTool, IntImpConicParConic, PConic};
/// use rustycad_tkgeomalgo::int_res2d::Domain;
/// use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Axis22d};
/// let line = Line2::new(Point2::new(-2.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
/// let circle = Circle2::new(Axis22d::default(), 1.0).unwrap();
/// let pconic = PConic::from_circle(circle);
/// let dom2 = Domain::bounded(Point2::new(1.0, 0.0), 0.0, 1e-9, Point2::new(1.0, 0.0), 6.0, 1e-9);
/// let inter = IntImpConicParConic::with_curves(&IConicTool::from_line(line).unwrap(), &Domain::new(),
///     &pconic, &dom2, 1e-9, 1e-9).unwrap();
/// assert_eq!(inter.nb_points().unwrap(), 2);
/// ```
pub struct Intersector<T: IntersectorTypes> {
    pub(super) base: Intersection,
    types: PhantomData<T>,
}

impl<T: IntersectorTypes> Clone for Intersector<T> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            types: PhantomData,
        }
    }
}

impl<T: IntersectorTypes> Default for Intersector<T> {
    fn default() -> Self {
        Self {
            base: Intersection::new(),
            types: PhantomData,
        }
    }
}

impl<T: IntersectorTypes> Deref for Intersector<T> {
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: IntersectorTypes> DerefMut for Intersector<T> {
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

/// The variables of the native Perform that live across its steps: the D2 outputs
/// keep their values from one evaluation to the next (a line's tool does not write
/// every output), and the end flags record which domain ends already hold a point.
pub(super) struct PerformState {
    pub(super) pt1: Point2,
    pub(super) pt2: Point2,
    pub(super) tan1: Vector2,
    pub(super) tan2: Vector2,
    pub(super) norm1: Vector2,
    pub(super) norm2: Vector2,
    pub(super) trans1: Transition,
    pub(super) trans2: Transition,
    pub(super) head_on_imp: bool,
    pub(super) head_on_par: bool,
    pub(super) end_on_imp: bool,
    pub(super) end_on_par: bool,
}

impl PerformState {
    fn new() -> Self {
        Self {
            pt1: Point2::default(),
            pt2: Point2::default(),
            tan1: Vector2::default(),
            tan2: Vector2::default(),
            norm1: Vector2::default(),
            norm2: Vector2::default(),
            trans1: Transition::default(),
            trans2: Transition::default(),
            head_on_imp: false,
            head_on_par: false,
            end_on_imp: false,
            end_on_par: false,
        }
    }
    // Records a position of a solution on the implicit (`imp`) and parametric curves.
    pub(super) fn record(&mut self, pos1: Position, pos2: Position) {
        match pos1 {
            Position::End => self.end_on_imp = true,
            Position::Head => self.head_on_imp = true,
            Position::Middle => {}
        }
        match pos2 {
            Position::End => self.end_on_par = true,
            Position::Head => self.head_on_par = true,
            Position::Middle => {}
        }
    }
    // `DetermineTransition` on the current derivatives and transitions.
    pub(super) fn transition(&mut self, pos1: Position, pos2: Position, tolerance: f64) {
        let side1 = TransitionSide {
            pos: pos1,
            tan: &mut self.tan1,
            norm: self.norm1,
            trans: &mut self.trans1,
        };
        let side2 = TransitionSide {
            pos: pos2,
            tan: &mut self.tan2,
            norm: self.norm2,
            trans: &mut self.trans2,
        };
        determine_transition(side1, side2, tolerance);
    }
}

/// The inputs and tolerances of one Perform.
pub(super) struct PerformInput<'a, T: IntersectorTypes> {
    pub(super) imp_tool: &'a T::Imp,
    pub(super) imp_domain: &'a Domain,
    pub(super) par_curve: &'a ParCurveOf<T>,
    pub(super) par_domain: &'a Domain,
    pub(super) eps_nul: f64,
    pub(super) tolerance_angulaire: f64,
}

impl<T: IntersectorTypes> Intersector<T> {
    /// `IntImpParGen_Intersector()`: not done. `IntImpConicParConic::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IntImpParGen_Intersector(ITool, Dom1, PCurve, Dom2, TolConf, Tol)`: performs
    /// the intersection; `IntImpConicParConic::with_curves(&itool, &d1, &pconic, &d2, 1e-9, 1e-9)?`.
    pub fn with_curves(
        imp_tool: &T::Imp,
        imp_domain: &Domain,
        par_curve: &ParCurveOf<T>,
        par_domain: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.perform(imp_tool, imp_domain, par_curve, par_domain, tol_conf, tol)?;
        Ok(inter)
    }
    /// `Perform(ITool, Dom1, PCurve, Dom2, TolConf, Tol)`: the isolated roots of the
    /// signed distance become points, its null intervals segments (with their parts
    /// shifted by a period on a closed implicit curve); bounds of both domains that
    /// meet within tolerance are added last. Standard_ConstructionError when the
    /// parametric domain is not bounded;
    /// `inter.perform(&itool, &d1, &pconic, &d2, 1e-9, 1e-9)?`.
    pub fn perform(
        &mut self,
        imp_tool: &T::Imp,
        imp_domain: &Domain,
        par_curve: &ParCurveOf<T>,
        par_domain: &Domain,
        tol_conf: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        let mut function: MyImpParTool<'_, T> = MyImpParTool::new(imp_tool, par_curve);
        if !(par_domain.has_first_point() && par_domain.has_last_point()) {
            return Err(construction_error("Domaine sur courbe incorrect"));
        }
        let (first, last) = (par_domain.first_parameter()?, par_domain.last_parameter()?);
        let nb_echantillons: i32 = T::Par::nb_samples(par_curve, first, last)?;
        let mut eps_x: f64 = T::Par::eps_x(par_curve);
        if eps_x > 1.0e-10 {
            eps_x = 1.0e-10;
        }
        let eps_nul: f64 = if tol_conf <= 1.0e-10 {
            1.0e-10
        } else {
            tol_conf
        };
        let eps_dist: f64 = if tol <= 1.0e-10 { 1.0e-10 } else { tol };
        if (last - first) < 100.0 * eps_x {
            eps_x = (last - first) * 0.01;
        }
        let sample = FunctionSample::new(first, last, nb_echantillons);
        let roots = FunctionAllRoots::new(&mut function, &sample, [eps_x, eps_dist, eps_nul]);
        if let Some(failure) = function.take_failure() {
            return Err(failure);
        }
        let sol: FunctionAllRoots = roots?;
        if !sol.is_done() {
            self.base.done = false;
            return Ok(());
        }
        let input = PerformInput::<T> {
            imp_tool,
            imp_domain,
            par_curve,
            par_domain,
            eps_nul,
            tolerance_angulaire: eps_dist,
        };
        let mut state: PerformState = PerformState::new();
        self.insert_isolated_roots(&input, &sol, &mut state)?;
        self.add_null_intervals(&input, &sol, &mut state)?;
        self.insert_domain_ends(&input, &mut state)?;
        self.base.done = true;
        Ok(())
    }

    // The points: the isolated roots whose point is on the implicit domain.
    fn insert_isolated_roots(
        &mut self,
        input: &PerformInput<'_, T>,
        sol: &FunctionAllRoots,
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        for i in 1..=sol.number_of_points()? {
            let param2: f64 = sol.point(i)?;
            let mut pt: Point2 = Point2::default();
            let mut param1: f64 = self.find_u(param2, &mut pt, input.par_curve, input.imp_tool)?;
            if input.imp_domain.is_closed() {
                param1 = normalize_on_domain(param1, input.imp_domain)?;
            }
            if is_on_domain(input.imp_domain, param1, pt)? {
                self.insert_root(input, (param1, param2), state)?;
            }
        }
        Ok(())
    }

    fn insert_root(
        &mut self,
        input: &PerformInput<'_, T>,
        (param1, param2): (f64, f64),
        state: &mut PerformState,
    ) -> Result<(), Failure> {
        input
            .imp_tool
            .d2(param1, &mut state.pt1, &mut state.tan1, &mut state.norm1);
        T::Par::d2(
            input.par_curve,
            param2,
            &mut state.pt2,
            &mut state.tan2,
            &mut state.norm2,
        )?;
        let pos1: Position = determine_position(input.imp_domain, state.pt1, param1)?;
        let pos2: Position = determine_position(input.par_domain, state.pt2, param2)?;
        state.record(pos1, pos2);
        state.transition(pos1, pos2, input.tolerance_angulaire);
        let reversed: bool = self.base.reversed_parameters();
        let ip = IntersectionPoint::new(
            state.pt1,
            param1,
            param2,
            state.trans1,
            state.trans2,
            reversed,
        );
        self.base.insert(ip)
    }

    /// `FindU(parameter, point, ParCurve, ImpTool)`: the point of the parametric
    /// curve at `parameter` (written to `point`) and its parameter on the implicit
    /// curve; `inter.find_u(0.5, &mut point, &pconic, &itool)?`.
    pub fn find_u(
        &self,
        parameter: f64,
        point: &mut Point2,
        par_curve: &ParCurveOf<T>,
        imp_tool: &T::Imp,
    ) -> Result<f64, Failure> {
        *point = T::Par::value(par_curve, parameter)?;
        imp_tool.find_parameter(*point)
    }
}

// Whether a point of parameter `param1` on the implicit curve lies on its domain:
// beyond a bound it must be within that bound's tolerance.
pub(super) fn is_on_domain(domain: &Domain, param1: f64, pt: Point2) -> Result<bool, Failure> {
    if domain.has_first_point()
        && param1 < domain.first_parameter()?
        && pt.distance(domain.first_point()?) > domain.first_tolerance()?
    {
        return Ok(false);
    }
    if domain.has_last_point()
        && param1 > domain.last_parameter()?
        && pt.distance(domain.last_point()?) > domain.last_tolerance()?
    {
        return Ok(false);
    }
    Ok(true)
}
