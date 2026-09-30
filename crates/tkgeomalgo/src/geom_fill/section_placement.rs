// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionPlacement.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SectionPlacement`: where a section (a curve or a point) best sits on
//! the path of a location law, and the transformation that carries it there
//! (docs/geomalgo-design.md, section 14).
use super::location_law::GeomFillLocationLaw;
use super::section_placement_axis::{SectionAxisInit, section_size};
use super::sweep_failures::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCurve, GeomGeometry, GeomPoint};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Axis1, Point3};

/// `GeomFill_SectionPlacement`: the placement of a section on a path.
///
/// The native member `myExt` (an `Extrema_ExtPC` initialized once on the section)
/// is rebuilt on the section at each projection: every `Perform` of it starts from
/// its initialized state, so the results are the same.
/// `let mut sp = GeomFillSectionPlacement::new(&law, &section)?; sp.perform(1e-4)?;`.
pub struct GeomFillSectionPlacement {
    pub(crate) done: bool,
    pub(crate) isplan: bool,
    pub(crate) the_axe: Axis1,
    pub(crate) gabarit: f64,
    pub(crate) my_law: Handle<dyn GeomFillLocationLaw>,
    pub(crate) my_adp_section: GeomAdaptorCurve,
    pub(crate) my_section: Handle<dyn GeomCurve>,
    pub(crate) sec_param: f64,
    pub(crate) path_param: f64,
    pub(crate) dist: f64,
    pub(crate) angle_max: f64,
    pub(crate) my_is_point: bool,
    pub(crate) my_point: Point3,
}

impl GeomFillSectionPlacement {
    /// `GeomFill_SectionPlacement(L, Section)`: a point section, or a curve section
    /// with its size (`Gabarit`) and its axis (the plane normal when the curve is
    /// planar); `GeomFillSectionPlacement::new(&law, &section)?`.
    pub fn new(
        law: &Handle<dyn GeomFillLocationLaw>,
        section: &Handle<dyn GeomGeometry>,
    ) -> Result<Self, Failure> {
        live(section, "GeomFill_SectionPlacement", "Section")?;
        let mut placement = Self::empty(law);
        let point: Handle<GeomCartesianPoint> = section.cast();
        if let Some(p) = point.get() {
            placement.my_is_point = true;
            placement.my_point = p.pnt();
            placement.isplan = true;
        } else {
            let curve: Handle<dyn GeomCurve> = section.cast();
            placement.my_adp_section.load(&curve)?;
            placement.my_section = curve;
        }
        placement.gabarit = section_size(&placement)?;
        if !placement.my_is_point {
            let init = SectionAxisInit::of(&placement.my_adp_section)?;
            placement.the_axe = init.axe;
            placement.isplan = init.isplan;
        }
        Ok(placement)
    }

    fn empty(law: &Handle<dyn GeomFillLocationLaw>) -> Self {
        Self {
            done: false,
            isplan: false,
            the_axe: Axis1::default(),
            gabarit: 0.0,
            my_law: law.clone(),
            my_adp_section: GeomAdaptorCurve::default(),
            my_section: Handle::default(),
            sec_param: 0.0,
            path_param: 0.0,
            dist: f64::MAX,
            angle_max: 0.0,
            my_is_point: false,
            my_point: Point3::default(),
        }
    }

    /// `SetLocation(L)`: replaces the location law; `sp.set_location(&law)`.
    pub fn set_location(&mut self, law: &Handle<dyn GeomFillLocationLaw>) {
        self.my_law = law.clone();
    }

    /// `Perform(Tol)`: the placement on the path of the law; `sp.perform(1e-4)?`.
    pub fn perform(&mut self, tol: f64) -> Result<(), Failure> {
        let path = self.law("GeomFill_SectionPlacement::Perform")?.get_curve();
        self.perform_on_path(&path, tol)
    }

    pub(crate) fn law(&self, member: &str) -> Result<&dyn GeomFillLocationLaw, Failure> {
        live(&self.my_law, member, "myLaw")
    }

    pub(crate) fn section_adaptor(&self) -> &dyn Adaptor3dCurve {
        &self.my_adp_section
    }

    /// `IsDone()`; `sp.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ParameterOnPath()`; `sp.parameter_on_path()`.
    pub fn parameter_on_path(&self) -> f64 {
        self.path_param
    }

    /// `ParameterOnSection()`; `sp.parameter_on_section()`.
    pub fn parameter_on_section(&self) -> f64 {
        self.sec_param
    }

    /// `Distance()`: from the path point to the section point; `sp.distance()`.
    pub fn distance(&self) -> f64 {
        self.dist
    }

    /// `Angle()`: between the path tangent and the section; `sp.angle()`.
    pub fn angle(&self) -> f64 {
        self.angle_max
    }
}
