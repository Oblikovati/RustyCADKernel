// Created on: 1995-03-22
// Created by: Laurent BUCHARD
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of BRepTopAdaptor_FClass2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTopAdaptor_FClass2d`: the fast classification of a UV point against a face,
//! through one polygon classifier per wire, with the exact BRepClass classifier for
//! the points near a polygon and for the faces whose wires are malformed.
use crate::brep_class::{BRepClassFaceClassifier, BRepClassFaceExplorer};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::cslib::{Class2d, Class2dResult};
use rustycad_tkmath::gp::Point2;
use std::sync::Mutex;

/// `WireRole`: how a wire polygon classifies (`Invalid` for a malformed wire).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WireRole {
    Invalid,
    Inner,
    Outer,
}

/// `ClassificationMode`: `Perform` or `TestOnRestriction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClassificationMode {
    Perform,
    OnRestriction,
}

/// `BRepTopAdaptor_FClass2d` (docs/topalgo-design.md section 4): one `CSLib_Class2d`
/// polygon per wire of the face (sampled from its pcurves, refined while the polygon
/// deflection exceeds the expected thickness of the wire), the role of each wire, and
/// the exact `BRepClass_FaceClassifier` for the uncertain points, on one explorer
/// built on first use.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
/// # use rustycad_tkbrep::brep::BRepBuilder;
/// # use rustycad_tkg3d::geom::GeomPlane;
/// # use rustycad_tkernel::standard::Handle;
/// # use rustycad_tkmath::gp::{Direction3, Point3};
/// # let plane = GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
/// # let mut face = Face::default();
/// # BRepBuilder::default().make_face_on_surface(&mut face, &Handle::new(plane).cast(), 1e-7).unwrap();
/// // A face without wires is the whole surface.
/// let classifier = BRepTopAdaptorFClass2d::new(&face, 1e-9).unwrap();
/// assert_eq!(classifier.perform(Point2::new(3.0, 4.0), true).unwrap(), State::In);
/// ```
pub struct BRepTopAdaptorFClass2d {
    pub(super) tab_class: Vec<Class2d>,
    pub(super) tab_orien: Vec<WireRole>,
    pub(super) toluv: f64,
    pub(super) face: Face,
    exact_explorer: Mutex<Option<BRepClassFaceExplorer>>,
    pub(super) is_u_periodic: bool,
    pub(super) is_v_periodic: bool,
    pub(super) u_period: f64,
    pub(super) v_period: f64,
    pub(super) u1: f64,
    pub(super) v1: f64,
    pub(super) u2: f64,
    pub(super) v2: f64,
    pub(super) umin: f64,
    pub(super) umax: f64,
    pub(super) vmin: f64,
    pub(super) vmax: f64,
}

/// The reframed coordinates of a point and the iteration over its periodic copies.
struct PeriodicScan {
    u: f64,
    v: f64,
    reframed_u: f64,
    reframed_v: f64,
    is_u_reframed: bool,
    is_v_reframed: bool,
}

impl BRepTopAdaptorFClass2d {
    /// The fields before the wires are sampled (the native member initializers).
    pub(super) fn unsampled(face: &Face, tol_uv: f64) -> Self {
        Self {
            tab_class: Vec::new(),
            tab_orien: Vec::new(),
            toluv: tol_uv,
            face: face.clone(),
            exact_explorer: Mutex::new(None),
            is_u_periodic: false,
            is_v_periodic: false,
            u_period: 0.0,
            v_period: 0.0,
            u1: 0.0,
            v1: 0.0,
            u2: 0.0,
            v2: 0.0,
            umin: 0.0,
            umax: 0.0,
            vmin: 0.0,
            vmax: 0.0,
        }
    }

    /// `PerformInfinitePoint()`: the state of a point beyond the lower corner of the
    /// sampled bounds (IN when a bound is RealLast); `classifier.perform_infinite_point()?`.
    pub fn perform_infinite_point(&self) -> Result<State, Failure> {
        let last: f64 = real_last();
        if self.umax == -last || self.vmax == -last || self.umin == last || self.vmin == last {
            return Ok(State::In);
        }
        let point = Point2::new(
            self.umin - (self.umax - self.umin),
            self.vmin - (self.vmax - self.vmin),
        );
        self.perform(point, false)
    }

    /// `Perform(Puv, RecadreOnPeriodic)`: IN, OUT, or the exact state (tolerance
    /// `min(Toluv, 4)`) of a point near a wire polygon; on a periodic surface the
    /// periodic copies of an OUT point inside the sampled bounds are tried too;
    /// `classifier.perform(Point2::new(0.5, 0.5), true)?`.
    pub fn perform(&self, point: Point2, recadre_on_periodic: bool) -> Result<State, Failure> {
        self.classify(
            point,
            self.toluv,
            recadre_on_periodic,
            ClassificationMode::Perform,
        )
    }

    /// `TestOnRestriction(Puv, Tol, RecadreOnPeriodic)`: as [`Self::perform`] with the
    /// ON tolerance `tolerance`, a point near a wire polygon being ON;
    /// `classifier.test_on_restriction(Point2::new(0.5, 0.5), 1e-7, true)?`.
    pub fn test_on_restriction(
        &self,
        point: Point2,
        tolerance: f64,
        recadre_on_periodic: bool,
    ) -> Result<State, Failure> {
        self.classify(
            point,
            tolerance,
            recadre_on_periodic,
            ClassificationMode::OnRestriction,
        )
    }

    /// `Destroy()`: drops the polygons, the wire roles and the exact explorer; the
    /// classifier then answers IN everywhere; `classifier.destroy()`.
    pub fn destroy(&mut self) {
        self.tab_class.clear();
        self.tab_orien.clear();
        *self
            .exact_explorer
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    /// `exactState`: the BRepClass state of the point, on the explorer of the face
    /// built (with its recommended box switch) on the first call.
    fn exact_state(&self, point: Point2, tolerance: f64) -> Result<State, Failure> {
        let mut guard = self
            .exact_explorer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.is_none() {
            let mut explorer = BRepClassFaceExplorer::new(&self.face)?;
            let should_use: bool = explorer.should_use_bnd_box()?;
            explorer.set_use_bnd_box(should_use);
            *guard = Some(explorer);
        }
        let explorer: &mut BRepClassFaceExplorer = guard.as_mut().expect("explorer built above");
        let classifier = BRepClassFaceClassifier::with_explorer(explorer, &point, tolerance)?;
        Ok(classifier.state())
    }

    /// `classify`: the polygon state of each periodic copy until IN or ON.
    fn classify(
        &self,
        point: Point2,
        tolerance: f64,
        recadre_on_periodic: bool,
        mode: ClassificationMode,
    ) -> Result<State, Failure> {
        if self.tab_class.is_empty() {
            return Ok(State::In);
        }
        //-- U1 is the First Param and U2 in this case is U1+Period
        let mut scan = PeriodicScan {
            u: point.x(),
            v: point.y(),
            reframed_u: point.x(),
            reframed_v: point.y(),
            is_u_reframed: false,
            is_v_reframed: false,
        };
        if recadre_on_periodic {
            self.reframe(&mut scan);
        }
        loop {
            let state: State = self.state_at(Point2::new(scan.u, scan.v), tolerance, mode)?;
            if !recadre_on_periodic || (!self.is_u_periodic && !self.is_v_periodic) {
                return Ok(state);
            }
            if state == State::In || state == State::On || !self.advance(&mut scan) {
                return Ok(state);
            }
        }
    }

    /// The reframed coordinates: the periodic copy of the point at or just above the
    /// lower sampled bound in each periodic direction.
    fn reframe(&self, scan: &mut PeriodicScan) {
        if self.is_u_periodic {
            scan.reframed_u = reframed(scan.reframed_u, self.umin, self.u_period);
        }
        if self.is_v_periodic {
            scan.reframed_v = reframed(scan.reframed_v, self.vmin, self.v_period);
        }
    }

    /// The next periodic copy of the point; false when the copies are exhausted.
    fn advance(&self, scan: &mut PeriodicScan) -> bool {
        if !scan.is_u_reframed {
            scan.u = scan.reframed_u;
            scan.is_u_reframed = true;
        } else if self.is_u_periodic {
            scan.u += self.u_period;
        }
        if scan.u > self.umax || !self.is_u_periodic {
            if !scan.is_v_reframed {
                scan.v = scan.reframed_v;
                scan.is_v_reframed = true;
            } else if self.is_v_periodic {
                scan.v += self.v_period;
            }
            scan.u = scan.reframed_u;
            if scan.v > self.vmax || !self.is_v_periodic {
                return false;
            }
        }
        true
    }

    /// The state of one point: the wire polygons when every wire is valid, else (and
    /// for an uncertain polygon answer of Perform) the exact classifier.
    fn state_at(
        &self,
        point: Point2,
        tolerance: f64,
        mode: ClassificationMode,
    ) -> Result<State, Failure> {
        if self.tab_orien[0] == WireRole::Invalid {
            // A malformed wire cannot be classified reliably by its polygon.
            let exact_tolerance: f64 = if mode == ClassificationMode::Perform {
                self.toluv
            } else {
                tolerance
            };
            return self.exact_state(point, exact_tolerance);
        }
        match self.polygon_result(point, tolerance, mode) {
            Class2dResult::Uncertain if mode == ClassificationMode::Perform => {
                self.exact_state(point, std_min(self.toluv, 4.0))
            }
            Class2dResult::Uncertain => Ok(State::On),
            Class2dResult::Inside => Ok(State::In),
            Class2dResult::Outside => Ok(State::Out),
        }
    }

    /// The polygon answer: OUT inside an inner wire or outside an outer one,
    /// uncertain near a wire, else inside.
    fn polygon_result(
        &self,
        point: Point2,
        tolerance: f64,
        mode: ClassificationMode,
    ) -> Class2dResult {
        for (polygon, role) in self.tab_class.iter().zip(&self.tab_orien) {
            let wire_result: Class2dResult = match mode {
                ClassificationMode::Perform => polygon.si_dans(point),
                ClassificationMode::OnRestriction => polygon.si_dans_on_mode(point, tolerance),
            };
            match (wire_result, role) {
                (Class2dResult::Inside, WireRole::Inner)
                | (Class2dResult::Outside, WireRole::Outer) => return Class2dResult::Outside,
                (Class2dResult::Uncertain, _) => return Class2dResult::Uncertain,
                _ => {}
            }
        }
        Class2dResult::Inside
    }
}

// The periodic copy of `value` in `[min, min + period)`: shifted up while below
// `min`, else down until below `min` and one period back up.
fn reframed(value: f64, min: f64, period: f64) -> f64 {
    let mut result: f64 = value;
    if result < min {
        while result < min {
            result += period;
        }
        return result;
    }
    while result >= min {
        result -= period;
    }
    result + period
}
