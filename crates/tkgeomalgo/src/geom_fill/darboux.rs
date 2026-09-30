// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Darboux.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Darboux`: the Darboux frame of a curve on a surface (tangent, surface
//! normal as binormal, and their cross product).
use super::darboux_normal::{normal_d0, normal_d1, normal_d2};
use super::frame_math::{dderiv, fderiv};
use super::frenet::average_frame;
use super::sweep_failures::{live, sweep_failure};
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Vector2, Vector3};

/// `GeomFill_Darboux`: the frame of a curve lying on a surface. The curve must be
/// an `Adaptor3d_CurveOnSurface`; the native code casts it without a check, the
/// port raises Standard_TypeMismatch for another adaptor.
/// `let law = GeomFillDarboux::new(); law.set_curve(&curve_on_surface)?;`.
#[derive(Default)]
pub struct GeomFillDarboux {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
}
impl_transient!(
    GeomFillDarboux,
    "GeomFill_Darboux",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

/// The 2d curve and the support of `myTrimmed`.
fn curve_on_surface(
    law: &GeomFillDarboux,
    member: &str,
) -> Result<(Handle<dyn Adaptor2dCurve2d>, Handle<dyn Adaptor3dSurface>), Failure> {
    let trimmed = law.base.trimmed();
    live(&trimmed, member, "myTrimmed")?;
    let cos: Handle<Adaptor3dCurveOnSurface> = trimmed.cast();
    let Some(c) = cos.get() else {
        return Err(sweep_failure(
            "Standard_TypeMismatch",
            &format!(
                "{member}: myTrimmed is not an Adaptor3d_CurveOnSurface; expected a curve on a surface"
            ),
        ));
    };
    Ok((c.get_curve(), c.get_surface()))
}

/// `a.X() * U + a.Y() * V`.
fn combine(a: Vector2, u: Vector3, v: Vector3) -> Vector3 {
    a.x() * u + a.y() * v
}

impl GeomFillDarboux {
    /// `GeomFill_Darboux()`; `GeomFillDarboux::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl GeomFillTrihedronLaw for GeomFillDarboux {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillDarboux::new();
        let curve = self.base.curve();
        if curve.is_some() {
            copy.set_curve(&curve)?;
        }
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        let (c2d, support) = curve_on_surface(self, "GeomFill_Darboux::D0")?;
        let (c2d, support) = (
            live(&c2d, "GeomFill_Darboux::D0", "myCurve2d")?,
            live(&support, "GeomFill_Darboux::D0", "mySupport")?,
        );
        let (p2d, d2d) = c2d.d1(param)?;
        let normal_dir = normal_d0(p2d.x(), p2d.y(), support)?;
        frame.binormal = Vector3::from(normal_dir.xyz());
        let s = support.d1(p2d.x(), p2d.y())?;
        frame.tangent = combine(d2d, s.d1u, s.d1v);
        frame.tangent.normalize()?;
        frame.normal = frame.binormal;
        frame.normal ^= frame.tangent;
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let (c2d, support) = curve_on_surface(self, "GeomFill_Darboux::D1")?;
        let (c2d, support) = (
            live(&c2d, "GeomFill_Darboux::D1", "myCurve2d")?,
            live(&support, "GeomFill_Darboux::D1", "mySupport")?,
        );
        let (p2d, d2d, d2_2d) = c2d.d2(param)?;
        let s = support.d2(p2d.x(), p2d.y())?;
        let f = combine(d2d, s.d1u, s.d1v);
        frame.tangent = f.normalized()?;
        let df = d2_2d.x() * s.d1u
            + d2_2d.y() * s.d1v
            + s.d2u * d2d.x() * d2d.x()
            + 2.0 * s.d2uv * d2d.x() * d2d.y()
            + s.d2v * d2d.y() * d2d.y();
        frame.d_tangent = fderiv(f, df);
        let (normal_dir, d1u_normal, d1v_normal) = normal_d1(p2d.x(), p2d.y(), support)?;
        frame.binormal = Vector3::from(normal_dir.xyz());
        frame.d_binormal = d1u_normal * d2d.x() + d1v_normal * d2d.y();
        frame.normal = frame.binormal;
        frame.normal ^= frame.tangent;
        frame.d_normal =
            frame.binormal.crossed(frame.d_tangent) + frame.d_binormal.crossed(frame.tangent);
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let (c2d, support) = curve_on_surface(self, "GeomFill_Darboux::D2")?;
        let (c2d, support) = (
            live(&c2d, "GeomFill_Darboux::D2", "myCurve2d")?,
            live(&support, "GeomFill_Darboux::D2", "mySupport")?,
        );
        let (p2d, d2d, d2_2d, d3_2d) = c2d.d3(param)?;
        let s = support.d3(p2d.x(), p2d.y())?;
        let f = combine(d2d, s.d1u, s.d1v);
        frame.tangent = f.normalized()?;
        let (dx, dy) = (d2d.x(), d2d.y());
        let df = d2_2d.x() * s.d1u
            + d2_2d.y() * s.d1v
            + s.d2u * dx * dx
            + 2.0 * s.d2uv * dx * dy
            + s.d2v * dy * dy;
        let d2f = d3_2d.x() * s.d1u
            + d3_2d.y() * s.d1v
            + d2_2d.x() * (dx * s.d2u + dy * s.d2uv)
            + d2_2d.y() * (dx * s.d2uv + dy * s.d2v)
            + dx * dx * (dx * s.d3u + dy * s.d3uuv)
            + dy * dy * (dx * s.d3uvv + dy * s.d3v)
            + 2.0
                * (dx * d2_2d.x() * s.d2u
                    + dy * d2_2d.y() * s.d2v
                    + dx * dy * (dx * s.d3uuv + dy * s.d3uvv)
                    + s.d2uv * (d2_2d.x() * dy + dx * d2_2d.y()));
        frame.d_tangent = fderiv(f, df);
        frame.d2_tangent = dderiv(f, df, d2f);
        let n = normal_d2(p2d.x(), p2d.y(), support)?;
        frame.binormal = Vector3::from(n.normal.xyz());
        frame.d_binormal = n.d1u * dx + n.d1v * dy;
        frame.d2_binormal = n.d1u * d2_2d.x()
            + n.d1v * d2_2d.y()
            + n.d2u * dx * dx
            + 2.0 * n.d2uv * dx * dy
            + n.d2v * dy * dy;
        frame.normal = frame.binormal;
        frame.normal ^= frame.tangent;
        frame.d_normal =
            frame.binormal.crossed(frame.d_tangent) + frame.d_binormal.crossed(frame.tangent);
        frame.d2_normal = frame.binormal.crossed(frame.d2_tangent)
            + 2.0 * frame.d_binormal.crossed(frame.d_tangent)
            + frame.d2_binormal.crossed(frame.tangent);
        Ok(true)
    }
    /// The intervals of the curve itself.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let curve = self.base.curve();
        live(&curve, "GeomFill_Darboux::NbIntervals", "myCurve")?.nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let curve = self.base.curve();
        live(&curve, "GeomFill_Darboux::Intervals", "myCurve")?.intervals(bounds, shape)
    }
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let mut average = average_frame(self, &self.base.trimmed())?;
        average.tangent /= 21.0;
        average.normal /= 21.0;
        average.tangent.normalize()?;
        average.binormal = average.tangent.crossed(average.normal).normalized()?;
        average.normal = average.binormal.crossed(average.tangent);
        Ok(average)
    }
    /// Whether the curve is a line.
    fn is_constant(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        Ok(live(&curve, "GeomFill_Darboux::IsConstant", "myCurve")?.get_type()? == CurveType::Line)
    }
}
