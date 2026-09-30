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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CurveAndTrihedron.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_CurveAndTrihedron`: the location law of a path and a trihedron law,
//! `M = (N, B, T)` and `V = C(t)`.
use super::location_helpers::{average_placement, columns, differs_from_identity};
use super::location_law::{
    GeomFillLocationLaw, LocationD0, LocationD1, LocationD2, LocationPoles2d,
};
use super::sweep_failures::live;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0, TrihedronD1, TrihedronD2};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Matrix3, Point2, Point3, Vector2, Vector3};

/// The path, its trimmed copy and the transformation (`myCurve`, `myTrimmed`,
/// `Trans`, `WithTrans`).
#[derive(Clone)]
struct CatFields {
    my_curve: Handle<dyn Adaptor3dCurve>,
    my_trimmed: Handle<dyn Adaptor3dCurve>,
    trans: Matrix3,
    with_trans: bool,
}

/// `GeomFill_CurveAndTrihedron`: `M(t)` from a trihedron law, `V(t)` the path point.
///
/// The native scratch members (`Point`, `V1`..`V3`) are kept local.
/// `let law = GeomFillCurveAndTrihedron::new(&frenet); law.set_curve(&path)?;`.
pub struct GeomFillCurveAndTrihedron {
    parent: StandardTransient,
    my_law: Handle<dyn GeomFillTrihedronLaw>,
    fields: SharedState<CatFields>,
}
impl_transient!(
    GeomFillCurveAndTrihedron,
    "GeomFill_CurveAndTrihedron",
    dyn GeomFillLocationLaw,
    parent,
    interfaces: [dyn GeomFillLocationLaw]
);

impl GeomFillCurveAndTrihedron {
    /// `GeomFill_CurveAndTrihedron(Trihedron)`: no curve, identity transformation.
    pub fn new(trihedron: &Handle<dyn GeomFillTrihedronLaw>) -> Self {
        let mut trans = Matrix3::default();
        trans.set_identity();
        Self {
            parent: StandardTransient::default(),
            my_law: trihedron.clone(),
            fields: SharedState::new(CatFields {
                my_curve: Handle::default(),
                my_trimmed: Handle::default(),
                trans,
                with_trans: false,
            }),
        }
    }

    fn law(&self) -> Result<&dyn GeomFillTrihedronLaw, Failure> {
        live(&self.my_law, "GeomFill_CurveAndTrihedron", "myLaw")
    }

    fn trimmed(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let trimmed = self.fields.read().my_trimmed.clone();
        live(&trimmed, member, "myTrimmed")?;
        Ok(trimmed)
    }

    fn my_curve(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let curve = self.fields.read().my_curve.clone();
        live(&curve, member, "myCurve")?;
        Ok(curve)
    }

    /// `M *= Trans` when the transformation is not the identity.
    fn transformed(&self, m: &mut [&mut Matrix3]) {
        let fields = self.fields.read();
        if fields.with_trans {
            for matrix in m.iter_mut() {
                **matrix *= fields.trans;
            }
        }
    }

    fn fused_intervals(
        &self,
        shape: Shape,
        nb_sec: i32,
        nb_law: i32,
    ) -> Result<LinearVector<f64>, Failure> {
        let mut int_c = one_based(nb_sec + 1)?;
        let mut int_l = one_based(nb_law + 1)?;
        self.trimmed("GeomFill_CurveAndTrihedron::Intervals")?
            .get()
            .expect("checked")
            .intervals(&mut int_c, shape)?;
        self.law()?.intervals(&mut int_l, shape)?;
        let mut inter = LinearVector::with_capacity(0);
        geom_lib::fuse_intervals(
            &int_c,
            &int_l,
            &mut inter,
            precision::p_confusion() * 0.99,
            false,
        );
        Ok(inter)
    }
}

impl GeomFillLocationLaw for GeomFillCurveAndTrihedron {
    /// Loads the path into the law and into the trihedron law.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        {
            let mut fields = self.fields.write();
            fields.my_curve = curve.clone();
            fields.my_trimmed = curve.clone();
        }
        self.law()?.set_curve(curve)
    }
    fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.fields.read().my_curve.clone()
    }
    fn set_trsf(&self, transfo: &Matrix3) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        fields.trans = *transfo;
        fields.with_trans = differs_from_identity(transfo);
        Ok(())
    }
    /// A law on a copy of the trihedron law (copied twice, as natively), with the
    /// same path and transformation.
    fn copy(&self) -> Result<Handle<dyn GeomFillLocationLaw>, Failure> {
        let _unused_copy = self.law()?.copy()?;
        let copy = GeomFillCurveAndTrihedron::new(&self.law()?.copy()?);
        let fields = self.fields.read().clone();
        copy.set_curve(&fields.my_curve)?;
        copy.set_trsf(&fields.trans)?;
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure> {
        let point = self
            .trimmed("GeomFill_CurveAndTrihedron::D0")?
            .get()
            .expect("checked")
            .d0(param)?;
        placement.v = Vector3::from(point.xyz());
        let mut frame = TrihedronD0::default();
        let ok = self.law()?.d0(param, &mut frame)?;
        placement.m = columns(frame.normal, frame.binormal, frame.tangent);
        self.transformed(&mut [&mut placement.m]);
        Ok(ok)
    }
    fn d0_poles2d(
        &self,
        param: f64,
        placement: &mut LocationD0,
        _poles2d: &mut Array1<'_, Point2>,
    ) -> Result<bool, Failure> {
        self.d0(param, placement)
    }
    fn d1(
        &self,
        param: f64,
        placement: &mut LocationD1,
        _poles2d: &mut Array1<'_, Point2>,
        _d_poles2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let (point, dv) = self
            .trimmed("GeomFill_CurveAndTrihedron::D1")?
            .get()
            .expect("checked")
            .d1(param)?;
        placement.dv = dv;
        placement.v = Vector3::from(point.xyz());
        let mut f = TrihedronD1::default();
        let ok = self.law()?.d1(param, &mut f)?;
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        self.transformed(&mut [&mut placement.m, &mut placement.dm]);
        Ok(ok)
    }
    fn d2(
        &self,
        param: f64,
        placement: &mut LocationD2,
        _poles: LocationPoles2d<'_, '_>,
    ) -> Result<bool, Failure> {
        let (point, dv, d2v) = self
            .trimmed("GeomFill_CurveAndTrihedron::D2")?
            .get()
            .expect("checked")
            .d2(param)?;
        (placement.dv, placement.d2v) = (dv, d2v);
        placement.v = Vector3::from(point.xyz());
        let mut f = TrihedronD2::default();
        let ok = self.law()?.d2(param, &mut f)?;
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        placement.d2m = columns(f.d2_normal, f.d2_binormal, f.d2_tangent);
        self.transformed(&mut [&mut placement.m, &mut placement.dm, &mut placement.d2m]);
        Ok(ok)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let trimmed = self.trimmed("GeomFill_CurveAndTrihedron::NbIntervals")?;
        let nb_sec = trimmed.get().expect("checked").nb_intervals(shape)?;
        let nb_law = self.law()?.nb_intervals(shape)?;
        if nb_sec == 1 {
            return Ok(nb_law);
        }
        if nb_law == 1 {
            return Ok(nb_sec);
        }
        Ok(self.fused_intervals(shape, nb_sec, nb_law)?.len() as i32 - 1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let trimmed = self.trimmed("GeomFill_CurveAndTrihedron::Intervals")?;
        let nb_sec = trimmed.get().expect("checked").nb_intervals(shape)?;
        let nb_law = self.law()?.nb_intervals(shape)?;
        if nb_sec == 1 {
            return self.law()?.intervals(bounds, shape);
        }
        if nb_law == 1 {
            return trimmed.get().expect("checked").intervals(bounds, shape);
        }
        super::frenet::copy_fusion(&self.fused_intervals(shape, nb_sec, nb_law)?, bounds)
    }
    /// The trihedron law interval and `myTrimmed = myCurve->Trim(First, Last, 0)`.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.law()?.set_interval(first, last)?;
        let curve = self.my_curve("GeomFill_CurveAndTrihedron::SetInterval")?;
        let trimmed = curve.get().expect("checked").trim(first, last, 0.0)?;
        self.fields.write().my_trimmed = trimmed;
        Ok(())
    }
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        let trimmed = self.trimmed("GeomFill_CurveAndTrihedron::GetInterval")?;
        let c = trimmed.get().expect("checked");
        Ok((c.first_parameter()?, c.last_parameter()?))
    }
    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        let curve = self.my_curve("GeomFill_CurveAndTrihedron::GetDomain")?;
        let c = curve.get().expect("checked");
        Ok((c.first_parameter()?, c.last_parameter()?))
    }
    /// Always 1.
    fn get_maximal_norm(&self) -> Result<f64, Failure> {
        Ok(1.0)
    }
    fn get_average_law(&self) -> Result<LocationD0, Failure> {
        let frame = self.law()?.get_average_law()?;
        let trimmed = self.trimmed("GeomFill_CurveAndTrihedron::GetAverageLaw")?;
        average_placement(frame, trimmed.get().expect("checked"))
    }
    /// A line path with a constant (or curve-only) trihedron law; `Error = 0`.
    fn is_translation(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        let curve = self.my_curve("GeomFill_CurveAndTrihedron::IsTranslation")?;
        if curve.get().expect("checked").get_type()? == CurveType::Line {
            return Ok(self.law()?.is_constant()? || self.law()?.is_only_by_3d_curve()?);
        }
        Ok(false)
    }
    /// A circle path with a curve-only trihedron law; `Error = 0`.
    fn is_rotation(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        let curve = self.my_curve("GeomFill_CurveAndTrihedron::IsRotation")?;
        if curve.get().expect("checked").get_type()? == CurveType::Circle {
            return self.law()?.is_only_by_3d_curve();
        }
        Ok(false)
    }
    /// The centre of the circle path.
    fn rotation(&self, center: &mut Point3) -> Result<(), Failure> {
        let curve = self.my_curve("GeomFill_CurveAndTrihedron::Rotation")?;
        *center = curve.get().expect("checked").circle()?.location();
        Ok(())
    }
}
