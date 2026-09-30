// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of the Init members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Pipe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The set-up of a pipe: the location law on the path and the section law of the
//! section placed at the start of the path.
use super::constant_bi_normal::GeomFillConstantBiNormal;
use super::corrected_frenet::GeomFillCorrectedFrenet;
use super::curve_and_trihedron::GeomFillCurveAndTrihedron;
use super::darboux::GeomFillDarboux;
use super::fixed::GeomFillFixed;
use super::frenet::GeomFillFrenet;
use super::guide_trihedron_ac::GeomFillGuideTrihedronAC;
use super::guide_trihedron_plan::GeomFillGuideTrihedronPlan;
use super::location_guide::GeomFillLocationGuide;
use super::location_law::GeomFillLocationLaw;
use super::pipe::GeomFillPipe;
use super::pipe_error::GeomFillTrihedron;
use super::section_placement::GeomFillSectionPlacement;
use super::sweep_failures::{construction_error, live};
use super::trihedron_law::GeomFillTrihedronLaw;
use super::trihedron_with_guide::GeomFillTrihedronWithGuide;
use super::uniform_section::GeomFillUniformSection;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomGeometry, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_lprop::CLProps;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, StandardDirection3, Vector3};
use std::f64::consts::PI;

/// An adaptor on a copy of `path`.
fn copied_path(path: &Handle<dyn GeomCurve>) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let copy: Handle<dyn GeomCurve> = live(path, "GeomFill_Pipe::Init", "Path")?.copy()?.cast();
    Ok(Handle::new(GeomAdaptorCurve::with_curve(&copy)?).cast())
}

/// `(First, Last)` of a geometric curve.
fn curve_range(path: &Handle<dyn GeomCurve>) -> Result<(f64, f64), Failure> {
    let c = live(path, "GeomFill_Pipe::Init", "Path")?;
    Ok((c.first_parameter(), c.last_parameter()))
}

/// The unit tangent and normal of `curve` at `param` (1e-9 resolution) when the
/// tangent is defined: the normal on a curved point, else the X direction of the
/// frame of the tangent.
fn tangent_and_normal(
    curve: &Handle<dyn GeomCurve>,
    param: f64,
) -> Result<Option<(Vector3, Vector3)>, Failure> {
    let eps = 1.0e-9;
    let mut cp = CLProps::with_parameter(curve.clone(), param, 2, eps)?;
    if !cp.is_tangent_defined()? {
        return Ok(None);
    }
    let d = cp.tangent()?;
    let mut tangent = Vector3::from(d.xyz());
    tangent.normalize()?;
    let normal_dir = if cp.curvature()? > eps {
        cp.normal()?
    } else {
        Axis2::new(Point3::new(0.0, 0.0, 0.0), d)?.x_direction()
    };
    let mut normal = Vector3::from(normal_dir.xyz());
    normal.normalize()?;
    Ok(Some((tangent, normal)))
}

impl GeomFillPipe {
    /// The curve-and-trihedron law of `t_law` on the path.
    fn set_curve_law(&mut self, t_law: Handle<dyn GeomFillTrihedronLaw>) -> Result<bool, Failure> {
        self.my_loc = Handle::new(GeomFillCurveAndTrihedron::new(&t_law)).cast();
        live(&self.my_loc, "GeomFill_Pipe::Init", "myLoc")?.set_curve(&self.my_adp_path)
    }

    /// The section placed on the path by the location law, untranslated.
    fn placed_section(
        &self,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<GeomFillSectionPlacement, Failure> {
        let mut place = GeomFillSectionPlacement::new(&self.my_loc, &first_sect.clone().cast())?;
        place.perform(precision::confusion())?;
        Ok(place)
    }

    /// `mySec`: the uniform law of `sect` over `range`.
    fn set_uniform_section(
        &mut self,
        sect: &Handle<dyn GeomCurve>,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        self.my_sec = Handle::new(GeomFillUniformSection::new(sect, range.0, range.1)?).cast();
        Ok(())
    }

    /// `Init(Path, Radius)`: a circle of `radius` in the plane normal to the path,
    /// moved by the corrected Frenet trihedron; `pipe.init_radius(&path, 1.0)?`.
    pub fn init_radius(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        radius: f64,
    ) -> Result<(), Failure> {
        (self.my_type, self.my_error, self.my_radius) = (1, 0.0, radius);
        self.my_adp_path = Handle::new(GeomAdaptorCurve::with_curve(path)?).cast();
        let circle = GeomCircle::new(Axis2::from_standard(StandardDirection3::Z), radius)?;
        circle.rotate(Axis1::from_standard(StandardDirection3::Z), PI / 2.0)?;
        let c: Handle<dyn GeomCurve> = Handle::new(circle).cast();
        self.set_uniform_section(&c, curve_range(path)?)?;
        let t_law: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillCorrectedFrenet::new()).cast();
        self.set_curve_law(t_law)?;
        Ok(())
    }

    /// `Init(Path, FirstSect, Option)`: the section placed by the trihedron law of
    /// `option` (Darboux falls back to Frenet; a guided or discrete option raises
    /// Standard_ConstructionError); `pipe.init_section(&path, &sect, GeomFillTrihedron::IsFrenet)?`.
    pub fn init_section(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        option: GeomFillTrihedron,
    ) -> Result<(), Failure> {
        self.my_adp_path = copied_path(path)?;
        let (first, last) = curve_range(path)?;
        let t_law: Handle<dyn GeomFillTrihedronLaw> = match option {
            GeomFillTrihedron::IsCorrectedFrenet => {
                Handle::new(GeomFillCorrectedFrenet::new()).cast()
            }
            GeomFillTrihedron::IsDarboux | GeomFillTrihedron::IsFrenet => {
                Handle::new(GeomFillFrenet::new()).cast()
            }
            GeomFillTrihedron::IsFixed => self.fixed_law(path, first)?,
            GeomFillTrihedron::IsConstantNormal => self.constant_normal_law(first_sect)?,
            _ => return Err(construction_error("GeomFill::Init : Unknown Option")),
        };
        self.set_curve_law(t_law)?;
        let sect = self.placed_section(first_sect)?.section(false)?;
        self.set_uniform_section(&sect, (first, last))
    }

    /// The fixed trihedron of the path start (Z and Y when its tangent is undefined).
    fn fixed_law(
        &self,
        path: &Handle<dyn GeomCurve>,
        param: f64,
    ) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let (v1, v2) = tangent_and_normal(path, param)?
            .unwrap_or((Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0)));
        Ok(Handle::new(GeomFillFixed::new(v1, v2)?).cast())
    }

    /// The constant binormal of the section normal at its placement on the Frenet
    /// law (Y when the section tangent is undefined).
    fn constant_normal_law(
        &mut self,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        self.set_curve_law(Handle::new(GeomFillFrenet::new()).cast())?;
        let ponsec = self.placed_section(first_sect)?.parameter_on_section();
        let v =
            tangent_and_normal(first_sect, ponsec)?.map_or(Vector3::new(0.0, 1.0, 0.0), |(_, n)| n);
        Ok(Handle::new(GeomFillConstantBiNormal::new(Direction3::from_vector(v)?)).cast())
    }

    /// `Init(Path, Support, FirstSect)`: a 2d path on a surface with the Darboux
    /// trihedron; `pipe.init_on_surface(&path2d, &support, &sect)?`.
    pub fn init_on_surface(
        &mut self,
        path: &Handle<dyn Geom2dCurve>,
        support: &Handle<dyn GeomSurface>,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        let t_law: Handle<dyn GeomFillTrihedronLaw> = Handle::new(GeomFillDarboux::new()).cast();
        let curve: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_curve(path)?).cast();
        let surface: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(support)?).cast();
        let cos = Adaptor3dCurveOnSurface::with_curve(&curve, &surface)?;
        self.my_adp_path = Handle::new(Adaptor3dCurveOnSurface::copy_of(&cos)).cast();
        self.set_curve_law(t_law)?;
        let mut place = GeomFillSectionPlacement::new(&self.my_loc, &first_sect.clone().cast())?;
        place.perform_on_path(&self.my_adp_path, precision::confusion())?;
        let sect = place.section(false)?;
        let adp = live(&self.my_adp_path, "GeomFill_Pipe::Init", "myAdpPath")?;
        let range = (adp.first_parameter()?, adp.last_parameter()?);
        self.set_uniform_section(&sect, range)
    }

    /// `Init(Path, FirstSect, Direction)`: the section with a constant binormal;
    /// `pipe.init_direction(&path, &sect, Direction3::Z)?`.
    pub fn init_direction(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        direction: Direction3,
    ) -> Result<(), Failure> {
        self.reset();
        self.my_adp_path = copied_path(path)?;
        let v = Vector3::from(direction.xyz());
        let t_law: Handle<dyn GeomFillTrihedronLaw> =
            Handle::new(GeomFillConstantBiNormal::new(Direction3::from_vector(v)?)).cast();
        self.set_curve_law(t_law)?;
        let sect = self.placed_section(first_sect)?.section(false)?;
        self.set_uniform_section(&sect, curve_range(path)?)
    }

    /// `Init(Path, Curve1, Curve2, Radius)`: the circular blend between two rails;
    /// `pipe.init_rails(&path, &c1, &c2, 1.0)`.
    pub fn init_rails(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        curve1: &Handle<dyn Adaptor3dCurve>,
        curve2: &Handle<dyn Adaptor3dCurve>,
        radius: f64,
    ) {
        (self.my_type, self.my_error, self.my_radius) = (4, 0.0, radius);
        self.my_adp_path = path.clone();
        self.my_adp_first_sect = curve1.clone();
        self.my_adp_last_sect = curve2.clone();
    }

    /// `Init(Path, Guide, FirstSect, byACR, rotat)`: the section placed by the
    /// guided law and, with `rotat`, turned to follow the guide;
    /// `pipe.init_guide(&path, &guide, &sect, true, false)?`.
    pub fn init_guide(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        guide: &Handle<dyn Adaptor3dCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        by_acr: bool,
        rotat: bool,
    ) -> Result<(), Failure> {
        self.my_adp_path = copied_path(path)?;
        let t_law: Handle<dyn GeomFillTrihedronWithGuide> = if by_acr {
            Handle::new(GeomFillGuideTrihedronAC::new(guide)?).cast()
        } else {
            Handle::new(GeomFillGuideTrihedronPlan::new(guide)?).cast()
        };
        live(&t_law, "GeomFill_Pipe::Init", "TLaw")?.set_curve(&self.my_adp_path)?;
        let the_loc = GeomFillLocationGuide::new(&t_law)?;
        the_loc.set_curve(&self.my_adp_path)?;
        let the_loc: Handle<GeomFillLocationGuide> = Handle::new(the_loc);
        let loc: Handle<dyn GeomFillLocationLaw> = the_loc.clone().cast();
        let mut place = GeomFillSectionPlacement::new(&loc, &first_sect.clone().cast())?;
        place.perform(precision::confusion())?;
        let adp = live(&self.my_adp_path, "GeomFill_Pipe::Init", "myAdpPath")?;
        let (first, last) = (adp.first_parameter()?, adp.last_parameter()?);
        self.set_uniform_section(&place.section(false)?, (first, last))?;
        if rotat {
            live(&the_loc, "GeomFill_Pipe::Init", "TheLoc")?.set(
                &self.my_sec,
                rotat,
                first,
                last,
                0.0,
            )?;
        }
        self.my_loc = loc;
        Ok(())
    }
}
