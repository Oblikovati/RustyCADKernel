// Created on: 1993-05-28
// Created by: Modelistation
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of BRepClass_FaceClassifier.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass_FaceClassifier`: the state of a 2d point, or of the projection of a 3d
//! point, against a face.
use super::FClassifier;
use super::edge::BRepClassEdge;
use super::face_explorer::BRepClassFaceExplorer;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeomalgo::int_res2d::Position;
use rustycad_tkgeombase::extrema::ExtPS;
use rustycad_tkmath::gp::{Point2, Point3};

/// `BRepClass_FaceClassifier` (docs/topalgo-design.md section 2): a
/// [`FClassifier`] that also builds its explorer from a face and classifies the
/// projection of a 3d point. When the 3d point has no projection the classifier is
/// rejected: `State` is OUT until the next 2d Perform.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
/// let classifier = BRepClassFaceClassifier::new();
/// assert_eq!(classifier.state(), State::In);
/// assert!(classifier.no_wires());
/// ```
#[derive(Default)]
pub struct BRepClassFaceClassifier {
    base: FClassifier,
    // The protected `rejected` of the base, as the 3d Perform writes it.
    rejected_by_projection: bool,
}

impl BRepClassFaceClassifier {
    /// `BRepClass_FaceClassifier()`: the undefined algorithm (IN, no wires);
    /// `BRepClassFaceClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepClass_FaceClassifier(F, P, Tol)`: classifies `point` on the face that
    /// `explorer` describes; `BRepClassFaceClassifier::with_explorer(&mut explorer, &p, 1e-7)?`.
    pub fn with_explorer(
        explorer: &mut BRepClassFaceExplorer,
        point: &Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            base: FClassifier::with_point(explorer, point, tolerance)?,
            rejected_by_projection: false,
        })
    }

    /// `BRepClass_FaceClassifier(theF, theP, theTol, theUseBndBox, theGapCheckTol)`
    /// with a 2d point ([`Self::perform`]); the native defaults are `false` and 0.1;
    /// `BRepClassFaceClassifier::with_face_point2d(&face, &p, 1e-7, false, 0.1)?`.
    pub fn with_face_point2d(
        face: &Face,
        point: &Point2,
        tolerance: f64,
        use_bnd_box: bool,
        gap_check_tol: f64,
    ) -> Result<Self, Failure> {
        let mut classifier = Self::new();
        classifier.perform(face, point, tolerance, use_bnd_box, gap_check_tol)?;
        Ok(classifier)
    }

    /// `BRepClass_FaceClassifier(theF, theP, theTol, theUseBndBox, theGapCheckTol)`
    /// with a 3d point ([`Self::perform_point3d`]);
    /// `BRepClassFaceClassifier::with_face_point3d(&face, &p, 1e-7, false, 0.1)?`.
    pub fn with_face_point3d(
        face: &Face,
        point: &Point3,
        tolerance: f64,
        use_bnd_box: bool,
        gap_check_tol: f64,
    ) -> Result<Self, Failure> {
        let mut classifier = Self::new();
        classifier.perform_point3d(face, point, tolerance, use_bnd_box, gap_check_tol)?;
        Ok(classifier)
    }

    /// `Perform(theF, theP, theTol, theUseBndBox, theGapCheckTol)`: classifies `point`
    /// with an explorer of `face` whose maximum tolerance is `gap_check_tol` and
    /// whose boxes follow `use_bnd_box` (boxes are recommended above 10 mostly spline
    /// edges); `classifier.perform(&face, &p, 1e-7, false, 0.1)?`.
    pub fn perform(
        &mut self,
        face: &Face,
        point: &Point2,
        tolerance: f64,
        use_bnd_box: bool,
        gap_check_tol: f64,
    ) -> Result<(), Failure> {
        let mut explorer = BRepClassFaceExplorer::new(face)?;
        explorer.set_max_tolerance(gap_check_tol);
        explorer.set_use_bnd_box(use_bnd_box);
        // The base Perform overwrites `rejected` before anything can fail.
        self.rejected_by_projection = false;
        self.base.perform(&mut explorer, point, tolerance)
    }

    /// `Perform(theF, theP, theTol, theUseBndBox, theGapCheckTol)` with a 3d point:
    /// classifies the UV parameters of the nearest projection of `point` on the face's
    /// surface (Extrema_ExtPS on the face's UV bounds with tolerance `tolerance`).
    /// Without a projection the classifier is rejected (OUT);
    /// `classifier.perform_point3d(&face, &p, 1e-7, false, 0.1)?`.
    pub fn perform_point3d(
        &mut self,
        face: &Face,
        point: &Point3,
        tolerance: f64,
        use_bnd_box: bool,
        gap_check_tol: f64,
    ) -> Result<(), Failure> {
        let surface = BRepAdaptorSurface::with_face(face, false)?;
        let (u1, u2, v1, v2) = brep_tools::uv_bounds(face)?;
        let mut extrema = ExtPS::new();
        extrema.initialize(&surface, (u1, u2), (v1, v2), (tolerance, tolerance))?;
        // modified by NIZNHY-PKV Wed Aug 13 11:28:47 2008
        self.rejected_by_projection = true;
        extrema.perform(*point)?;
        if !extrema.is_done() {
            return Ok(());
        }
        let Some(index) = nearest_extremum(&extrema)? else {
            return Ok(());
        };
        let (u, v) = extrema.point(index)?.parameter();
        self.perform(
            face,
            &Point2::new(u, v),
            tolerance,
            use_bnd_box,
            gap_check_tol,
        )
    }

    /// `State()` (of BRepClass_FClassifier): OUT when rejected, IN without wires,
    /// else the classification's state; `classifier.state()`.
    pub fn state(&self) -> State {
        if self.rejected_by_projection {
            return State::Out;
        }
        self.base.state()
    }

    /// `Rejected()`: the state was computed by a rejection (it is OUT);
    /// `classifier.rejected()`.
    pub fn rejected(&self) -> bool {
        self.rejected_by_projection || self.base.rejected()
    }

    /// `NoWires()`: the face has no wire (the state is IN); `classifier.no_wires()`.
    pub fn no_wires(&self) -> bool {
        self.base.no_wires()
    }

    /// `Edge()`: the edge of the closest intersection (the edge holding the point
    /// when the state is ON); Standard_DomainError when rejected; `classifier.edge()?`.
    pub fn edge(&self) -> Result<&BRepClassEdge, Failure> {
        self.check_not_rejected("TopClass_FaceClassifier::Edge")?;
        self.base.edge()
    }

    /// `EdgeParameter()`: the parameter of the closest intersection on
    /// [`Self::edge`]; Standard_DomainError when rejected;
    /// `classifier.edge_parameter()?`.
    pub fn edge_parameter(&self) -> Result<f64, Failure> {
        self.check_not_rejected("TopClass_FaceClassifier::EdgeParameter")?;
        self.base.edge_parameter()
    }

    /// `Position()`: the position of the closest intersection on [`Self::edge`];
    /// `classifier.position()`.
    pub fn position(&self) -> Position {
        self.base.position()
    }

    // The Standard_DomainError of the TopClass Edge and EdgeParameter when rejected.
    fn check_not_rejected(&self, member: &str) -> Result<(), Failure> {
        if !self.rejected_by_projection {
            return Ok(());
        }
        Err(Failure::with_message(
            "Standard_DomainError",
            &format!("{member} - rejected; expected a classification that was not rejected"),
        ))
    }
}

// The index of the smallest square distance (the first one of equal distances), None
// when no distance is below RealLast.
fn nearest_extremum(extrema: &ExtPS<'_>) -> Result<Option<i32>, Failure> {
    let mut max_dist: f64 = real_last();
    let mut nearest: Option<i32> = None;
    for index in 1..=extrema.nb_ext()? {
        let dist: f64 = extrema.square_distance(index)?;
        if dist < max_dist {
            max_dist = dist;
            nearest = Some(index);
        }
    }
    Ok(nearest)
}
