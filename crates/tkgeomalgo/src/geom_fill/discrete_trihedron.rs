// Copyright (c) 2001-2013 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_DiscreteTrihedron.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_DiscreteTrihedron`: a frame transported along the curve by rotations
//! between the tangents at sampled parameters (the rotation-minimizing frame of a
//! polyline), with the Frenet law on lines and conics.
use super::frenet::{GeomFillFrenet, average_frame};
use super::sweep_failures::{construction_error, live, sweep_failure};
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{self, Axis1, Axis2, Direction3, Point3, Vector3};

/// The sampled parameters and frames (`myKnots`, `myTrihedrons`, `myUseFrenet`).
#[derive(Clone, Default)]
struct DiscreteState {
    knots: Vec<f64>,
    trihedrons: Vec<Axis2>,
    use_frenet: bool,
}

/// `GeomFill_DiscreteTrihedron`: the discrete transported frame. `Init` appends to
/// the samples of a previous curve without clearing them, as natively; the native
/// member `myPoint` only receives evaluation points and is kept local.
/// `let law = GeomFillDiscreteTrihedron::new(); law.set_curve(&curve)?;`.
pub struct GeomFillDiscreteTrihedron {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    my_frenet: Handle<GeomFillFrenet>,
    state: SharedState<DiscreteState>,
}
impl_transient!(
    GeomFillDiscreteTrihedron,
    "GeomFill_DiscreteTrihedron",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

impl Default for GeomFillDiscreteTrihedron {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            my_frenet: Handle::new(GeomFillFrenet::new()),
            state: SharedState::new(DiscreteState::default()),
        }
    }
}

/// The tangent at `param`, or the chord to `param + subdelta` when it is null.
fn sample_tangent(
    curve: &dyn Adaptor3dCurve,
    param: f64,
    next: impl FnOnce() -> Result<f64, Failure>,
) -> Result<Vector3, Failure> {
    let (pnt, mut tangent) = curve.d1(param)?;
    if tangent.magnitude() < precision::confusion() {
        let next = next()?;
        let mut subdelta = (next - param) / 10.0;
        if subdelta < precision::p_confusion() {
            subdelta = next - param;
        }
        let sub_pnt = curve.value(param + subdelta)?;
        tangent = Vector3::from(sub_pnt.xyz() - pnt.xyz());
    }
    Ok(tangent)
}

/// `LastAxis` turned onto `tang_dir` around `last ^ tangent`, or `None` when the
/// two tangents are parallel.
fn rotated_axis(last_axis: Axis2, tangent: Vector3) -> Result<Option<Axis2>, Failure> {
    let last_tangent = Vector3::from(last_axis.direction());
    let axis_of_rotation = last_tangent ^ tangent;
    if axis_of_rotation.magnitude() <= gp::resolution() {
        return Ok(None);
    }
    let the_angle = last_tangent.angle_with_ref(tangent, axis_of_rotation)?;
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::from_vector(axis_of_rotation)?,
    );
    Ok(Some(last_axis.rotated(axis, the_angle)?))
}

impl GeomFillDiscreteTrihedron {
    /// `GeomFill_DiscreteTrihedron()`; `GeomFillDiscreteTrihedron::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn frenet(&self) -> &GeomFillFrenet {
        self.my_frenet
            .get()
            .expect("the constructor creates the Frenet law")
    }

    /// `Init()`: samples ten parameters per CN interval of the curve and transports
    /// the frame between them; `law.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        let trimmed = self.base.trimmed();
        let curve = live(&trimmed, "GeomFill_DiscreteTrihedron::Init", "myTrimmed")?;
        let nb_intervals = curve.nb_intervals(Shape::CN)?;
        let mut knots = one_based(nb_intervals + 1)?;
        curve.intervals(&mut knots, Shape::CN)?;
        let mut state = self.state.snapshot();
        for i in 1..=nb_intervals {
            let delta = (knots[i + 1] - knots[i]) / 10.0;
            for j in 0..10 {
                state.knots.push(knots[i] + f64::from(j) * delta);
            }
        }
        state.knots.push(knots[nb_intervals + 1]);
        let result = transport_frames(curve, &mut state);
        *self.state.write() = state;
        result
    }
}

/// The frame loop of `Init`: one frame per knot, inserting a middle knot where two
/// consecutive tangents are opposite.
fn transport_frames(curve: &dyn Adaptor3dCurve, state: &mut DiscreteState) -> Result<(), Failure> {
    let mut i = 0;
    while i < state.knots.len() {
        let param = state.knots[i];
        let knots = &state.knots;
        let tangent = sample_tangent(curve, param, || {
            knots.get(i + 1).copied().ok_or_else(|| no_next_knot(i + 2))
        })?;
        let tang_dir = Direction3::from_vector(tangent)?;
        let tangent = Vector3::from(tang_dir);
        if i == 0 {
            state
                .trihedrons
                .push(Axis2::new(Point3::new(0.0, 0.0, 0.0), tang_dir)?);
            i += 1;
            continue;
        }
        let last_axis = *state.trihedrons.last().expect("a first frame");
        match rotated_axis(last_axis, tangent)? {
            Some(mut new_axis) => {
                new_axis.set_direction(tang_dir)?;
                state.trihedrons.push(new_axis);
            }
            None if Vector3::from(last_axis.direction()) * tangent > 0.0 => {
                state.trihedrons.push(last_axis)
            }
            None => {
                let new_param = (state.knots[i - 1] + state.knots[i]) / 2.0;
                if new_param - state.knots[i - 1] < gp::resolution() {
                    return Err(construction_error(
                        "GeomFill_DiscreteTrihedron : impassable singularities on path curve",
                    ));
                }
                state.knots.insert(i, new_param);
                continue;
            }
        }
        i += 1;
    }
    Ok(())
}

/// The Standard_OutOfRange of `myKnots->Value(index)` past the last knot.
fn no_next_knot(index: usize) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!("GeomFill_DiscreteTrihedron: knot {index} does not exist; expected an inner knot"),
    )
}

/// The index of the knot interval holding `param` (bisection of `D0`), moved to
/// the upper knot when `param` is on it.
fn knot_index(knots: &[f64], param: f64) -> usize {
    let (mut i1, mut i2) = (1, knots.len());
    loop {
        let i = (i1 + i2) / 2;
        if param <= knots[i - 1] {
            i2 = i;
        } else {
            i1 = i;
        }
        if i2 - i1 <= 1 {
            break;
        }
    }
    if (param - knots[i2 - 1]).abs() < precision::p_confusion() {
        return i2;
    }
    i1
}

impl GeomFillDiscreteTrihedron {
    /// The transported frame at `param` (the non-Frenet branch of `D0`).
    fn discrete_frame(&self, param: f64) -> Result<Axis2, Failure> {
        let state = self.state.snapshot();
        let index = knot_index(&state.knots, param);
        let prev_param = state.knots[index - 1];
        let prev_axis = *state
            .trihedrons
            .get(index - 1)
            .ok_or_else(|| no_next_knot(index))?;
        if (param - prev_param).abs() < precision::p_confusion() {
            return Ok(prev_axis);
        }
        let trimmed = self.base.trimmed();
        let curve = live(&trimmed, "GeomFill_DiscreteTrihedron::D0", "myTrimmed")?;
        let tangent = sample_tangent(curve, param, || {
            state
                .knots
                .get(index)
                .copied()
                .ok_or_else(|| no_next_knot(index + 1))
        })?;
        let tang_dir = Direction3::from_vector(tangent)?;
        let mut the_axis = rotated_axis(prev_axis, Vector3::from(tang_dir))?.unwrap_or(prev_axis);
        the_axis.set_direction(tang_dir)?;
        Ok(the_axis)
    }
}

/// The frame of an axis: tangent along its direction, normal along X, binormal
/// along Y.
fn axis_frame(axis: Axis2) -> TrihedronD0 {
    TrihedronD0 {
        tangent: Vector3::from(axis.direction()),
        normal: Vector3::from(axis.x_direction()),
        binormal: Vector3::from(axis.y_direction()),
    }
}

impl GeomFillTrihedronLaw for GeomFillDiscreteTrihedron {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillDiscreteTrihedron::new();
        let curve = self.base.curve();
        if curve.is_some() {
            copy.set_curve(&curve)?;
        }
        Ok(Handle::new(copy).cast())
    }
    /// Lines and conics use the Frenet law; other curves run `Init`. Returns
    /// `myUseFrenet`.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.base.set_curve(curve);
        if let Some(c) = curve.get() {
            let conic = matches!(
                c.get_type()?,
                CurveType::Circle
                    | CurveType::Ellipse
                    | CurveType::Hyperbola
                    | CurveType::Parabola
                    | CurveType::Line
            );
            self.state.write().use_frenet = conic;
            if conic {
                self.frenet().set_curve(curve)?;
            } else {
                self.init()?;
            }
        }
        Ok(self.state.read().use_frenet)
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        if self.state.read().use_frenet {
            self.frenet().d0(param, frame)?;
            return Ok(true);
        }
        *frame = axis_frame(self.discrete_frame(param)?);
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        if self.state.read().use_frenet {
            self.frenet().d1(param, frame)?;
            return Ok(true);
        }
        let mut d0 = TrihedronD0::default();
        self.d0(param, &mut d0)?;
        *frame = TrihedronD1 {
            tangent: d0.tangent,
            normal: d0.normal,
            binormal: d0.binormal,
            ..TrihedronD1::default()
        };
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        if self.state.read().use_frenet {
            self.frenet().d2(param, frame)?;
            return Ok(true);
        }
        let mut d0 = TrihedronD0::default();
        self.d0(param, &mut d0)?;
        *frame = TrihedronD2 {
            tangent: d0.tangent,
            normal: d0.normal,
            binormal: d0.binormal,
            ..TrihedronD2::default()
        };
        Ok(true)
    }
    /// The CN intervals of the trimmed curve, whatever `shape`.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        let trimmed = self.base.trimmed();
        live(
            &trimmed,
            "GeomFill_DiscreteTrihedron::NbIntervals",
            "myTrimmed",
        )?
        .nb_intervals(Shape::CN)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let trimmed = self.base.trimmed();
        live(
            &trimmed,
            "GeomFill_DiscreteTrihedron::Intervals",
            "myTrimmed",
        )?
        .intervals(bounds, Shape::CN)
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
        Ok(
            live(&curve, "GeomFill_DiscreteTrihedron::IsConstant", "myCurve")?.get_type()?
                == CurveType::Line,
        )
    }
    /// Always true.
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        Ok(true)
    }
}
