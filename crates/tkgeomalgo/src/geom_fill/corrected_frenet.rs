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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CorrectedFrenet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_CorrectedFrenet`: the Frenet frame turned around the tangent by the
//! angle law that removes its twist. `Init` and the angle law are in
//! `corrected_frenet_init.rs`, the angle evaluation and `EvaluateBestMode` in
//! `corrected_frenet_mode.rs`.
use super::frenet::{GeomFillFrenet, copy_fusion};
use super::pipe_error::GeomFillTrihedron;
use super::sweep_failures::live;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use crate::law::LawFunction;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::Vector3;

/// The frames and angles `Init` records at the start of each step
/// (`HArrPoles`, `HArrAngle`, `HArrTangent`, `HArrNormal`).
#[derive(Clone, Default)]
pub(crate) struct AngleHistory {
    pub poles: Vec<f64>,
    pub angle: Vec<f64>,
    pub tangent: Vec<Vector3>,
    pub normal: Vec<Vector3>,
}

/// The fields `Init` and `SetInterval` write.
#[derive(Clone, Default)]
pub(crate) struct CorrectedState {
    /// `EvolAroundT`: the composite angle law (null before `Init`).
    pub evol_around_t: Handle<dyn LawFunction>,
    /// `TLaw`: the angle law on the current interval.
    pub t_law: Handle<dyn LawFunction>,
    /// `AT`, `AN`: the average tangent and normal of the last interval.
    pub at: Vector3,
    pub an: Vector3,
    /// `isFrenet`: the angle is null everywhere.
    pub is_frenet: bool,
    /// `HArrPoles`...: null (`None`) before the first `Init` with steps.
    pub history: Option<AngleHistory>,
}

/// `GeomFill_CorrectedFrenet`: a Frenet law whose normal is rotated by the angle
/// law that keeps it parallel along the curve.
///
/// `let law = GeomFillCorrectedFrenet::new(); law.set_curve(&curve)?; law.d0(0.5, &mut frame)?`.
pub struct GeomFillCorrectedFrenet {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    pub(crate) frenet: Handle<GeomFillFrenet>,
    pub(crate) for_evaluation: bool,
    pub(crate) state: SharedState<CorrectedState>,
}
impl_transient!(
    GeomFillCorrectedFrenet,
    "GeomFill_CorrectedFrenet",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

impl Default for GeomFillCorrectedFrenet {
    fn default() -> Self {
        Self::with_evaluation(false)
    }
}

impl GeomFillCorrectedFrenet {
    /// `GeomFill_CorrectedFrenet()`: for approximation; `GeomFillCorrectedFrenet::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GeomFill_CorrectedFrenet(ForEvaluation)`: with `for_evaluation`, `Init` skips
    /// the search of the curve plane; `GeomFillCorrectedFrenet::with_evaluation(true)`.
    pub fn with_evaluation(for_evaluation: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            frenet: Handle::new(GeomFillFrenet::new()),
            for_evaluation,
            state: SharedState::new(CorrectedState::default()),
        }
    }

    pub(crate) fn frenet(&self) -> &GeomFillFrenet {
        self.frenet
            .get()
            .expect("the constructor creates the Frenet law")
    }

    fn is_frenet(&self) -> bool {
        self.state.read().is_frenet
    }

    fn evol_around_t(&self, member: &str) -> Result<Handle<dyn LawFunction>, Failure> {
        let law = self.state.read().evol_around_t.clone();
        live(&law, member, "EvolAroundT")?;
        Ok(law)
    }

    /// `EvaluateBestMode()`: Frenet before `Init`, the discrete trihedron when the
    /// torsion or the angle law turns too fast, else the corrected Frenet law.
    pub fn evaluate_best_mode(&self) -> Result<GeomFillTrihedron, Failure> {
        let law = self.state.read().evol_around_t.clone();
        if law.is_null() {
            return Ok(GeomFillTrihedron::IsFrenet);
        }
        self.best_mode_of(&law)
    }
}

/// The frame rotated by `angle` around its tangent (D0 of the corrected law).
fn rotate_normal(frame: &mut TrihedronD0, angle: f64) {
    let cross = frame.tangent.crossed(frame.normal);
    let normal = frame.normal;
    frame.normal.set_linear_form_offset(
        angle.sin(),
        cross,
        1.0 - angle.cos(),
        frame.tangent.crossed(cross),
        normal,
    );
    frame.binormal = frame.tangent.crossed(frame.normal);
}

/// `aux = a1 V1 + a2 V2`, then `aux = b1 W1 + b2 W2 + aux`: the two
/// `SetLinearForm` of the corrected derivatives.
fn two_forms(a: (f64, Vector3, f64, Vector3), b: (f64, Vector3, f64, Vector3)) -> Vector3 {
    let mut aux = Vector3::default();
    aux.set_linear_form(a.0, a.1, a.2, a.3);
    let previous = aux;
    aux.set_linear_form_offset(b.0, b.1, b.2, b.3, previous);
    aux
}

/// `1 * V1 + V2`, the native `SetLinearForm(1, V1, V2)`.
fn one_sum(v1: Vector3, v2: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_scaled_sum(1.0, v1, v2);
    out
}

impl GeomFillTrihedronLaw for GeomFillCorrectedFrenet {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    /// A new law (for approximation, whatever `ForEvaluation` was, as natively) on
    /// the same curve.
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillCorrectedFrenet::new();
        let curve = self.base.curve();
        if curve.is_some() {
            copy.set_curve(&curve)?;
        }
        Ok(Handle::new(copy).cast())
    }
    /// Loads the curve into the law and its Frenet law; any curve but a line or a
    /// conic runs `Init`. Returns `isFrenet`.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.base.set_curve(curve);
        if let Some(c) = curve.get() {
            self.frenet().set_curve(curve)?;
            let kind = c.get_type()?;
            self.state.write().is_frenet = true;
            if !matches!(
                kind,
                CurveType::Circle
                    | CurveType::Ellipse
                    | CurveType::Hyperbola
                    | CurveType::Parabola
                    | CurveType::Line
            ) {
                self.init()?;
            }
        }
        Ok(self.is_frenet())
    }
    /// The base trim, the Frenet trim and the angle law trimmed to the interval.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.base.set_interval(first, last)?;
        self.frenet().set_interval(first, last)?;
        if !self.is_frenet() {
            let law = self.evol_around_t("GeomFill_CorrectedFrenet::SetInterval")?;
            let trimmed = law.get().expect("checked law").trim(
                first,
                last,
                precision::p_confusion() / 2.0,
            )?;
            self.state.write().t_law = trimmed;
        }
        Ok(())
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        self.frenet().d0(param, frame)?;
        if self.is_frenet() {
            return Ok(true);
        }
        let angle_at = self.get_angle_at(param)?;
        rotate_normal(frame, angle_at);
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        self.frenet().d1(param, frame)?;
        if self.is_frenet() {
            return Ok(true);
        }
        let (_, d_angle_at) = self.t_law()?.d1(param)?;
        let angle_at = self.get_angle_at(param)?;
        let (sina, cosa) = (angle_at.sin(), angle_at.cos());
        let f = *frame;
        let cross = f.tangent.crossed(f.normal);
        let dcross = one_sum(f.d_tangent.crossed(f.normal), f.tangent.crossed(f.d_normal));
        let tcross = f.tangent.crossed(cross);
        let dtcross = one_sum(f.d_tangent.crossed(cross), f.tangent.crossed(dcross));
        frame.d_normal += two_forms(
            (sina, dcross, cosa * d_angle_at, cross),
            (1.0 - cosa, dtcross, sina * d_angle_at, tcross),
        );
        let normal = frame.normal;
        frame
            .normal
            .set_linear_form_offset(sina, cross, 1.0 - cosa, tcross, normal);
        frame.binormal = frame.tangent.crossed(frame.normal);
        frame.d_binormal = one_sum(
            frame.d_tangent.crossed(frame.normal),
            frame.tangent.crossed(frame.d_normal),
        );
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        self.frenet().d2(param, frame)?;
        if self.is_frenet() {
            return Ok(true);
        }
        let (_, d_angle_at, d2_angle_at) = self.t_law()?.d2(param)?;
        let angle_at = self.get_angle_at(param)?;
        super::corrected_frenet_mode::corrected_d2(frame, angle_at, d_angle_at, d2_angle_at);
        Ok(true)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let nb_frenet = self.frenet().nb_intervals(shape)?;
        if self.is_frenet() {
            return Ok(nb_frenet);
        }
        let law = self.evol_around_t("GeomFill_CorrectedFrenet::NbIntervals")?;
        let nb_law = law.get().expect("checked law").nb_intervals(shape)?;
        if nb_frenet == 1 {
            return Ok(nb_law);
        }
        Ok(self.fused(&law, shape, nb_frenet, nb_law)?.len() as i32 - 1)
    }
    /// The fusion of the Frenet and angle-law intervals; with one Frenet interval the
    /// angle-law intervals are written first, then overwritten by the fusion (the
    /// native code has no return there).
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        if self.is_frenet() {
            return self.frenet().intervals(bounds, shape);
        }
        let nb_frenet = self.frenet().nb_intervals(shape)?;
        let law = self.evol_around_t("GeomFill_CorrectedFrenet::Intervals")?;
        let evol = law.get().expect("checked law");
        if nb_frenet == 1 {
            evol.intervals(bounds, shape)?;
        }
        let nb_law = evol.nb_intervals(shape)?;
        copy_fusion(&self.fused(&law, shape, nb_frenet, nb_law)?, bounds)
    }
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        if self.is_frenet() {
            return self.frenet().get_average_law();
        }
        let state = self.state.read();
        let mut binormal = state.at;
        binormal.cross(state.an);
        Ok(TrihedronD0 {
            tangent: state.at,
            normal: state.an,
            binormal,
        })
    }
    /// Whether the curve is a line.
    fn is_constant(&self) -> Result<bool, Failure> {
        let curve = self.base.curve();
        Ok(
            live(&curve, "GeomFill_CorrectedFrenet::IsConstant", "myCurve")?.get_type()?
                == CurveType::Line,
        )
    }
    /// Always true.
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        Ok(true)
    }
}

impl GeomFillCorrectedFrenet {
    /// `TLaw`, or Standard_NullObject when it was never set.
    fn t_law(&self) -> Result<Handle<dyn LawFunction>, Failure> {
        let law = self.state.read().t_law.clone();
        live(&law, "GeomFill_CorrectedFrenet", "TLaw")?;
        Ok(law)
    }

    /// The fusion of the Frenet and angle-law intervals of `shape`.
    fn fused(
        &self,
        law: &Handle<dyn LawFunction>,
        shape: Shape,
        nb_frenet: i32,
        nb_law: i32,
    ) -> Result<LinearVector<f64>, Failure> {
        let mut frenet_int = one_based(nb_frenet + 1)?;
        let mut law_int = one_based(nb_law + 1)?;
        self.frenet().intervals(&mut frenet_int, shape)?;
        law.get()
            .expect("checked law")
            .intervals(&mut law_int, shape)?;
        let mut fusion = LinearVector::with_capacity(0);
        geom_lib::fuse_intervals(
            &frenet_int,
            &law_int,
            &mut fusion,
            precision::p_confusion(),
            true,
        );
        Ok(fusion)
    }
}
