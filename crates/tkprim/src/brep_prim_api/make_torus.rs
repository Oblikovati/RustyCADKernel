// Rust translation of BRepPrimAPI_MakeTorus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeTorus`: the maker of tori.
use super::make_one_axis::one_axis_maker;
use crate::brep_prim::{BRepPrimOneAxis, BRepPrimTorus};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Axis2;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepPrimAPI_MakeTorus`: a torus of radii R1 (the distance from the center of the pipe
/// to the center of the torus) and R2 (the radius of the pipe) at the origin or on
/// `Axes`, possibly limited on the pipe (`angle1..angle2`, the ring segment) and around
/// the axis (`0..angle`, the pipe segment) (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeTorus;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepPrimAPIMakeTorus::new(10.0, 2.0)?;
/// assert!(!maker.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimAPIMakeTorus {
    base: BRepBuilderAPIMakeShapeBase,
    my_torus: BRepPrimTorus,
}

impl BRepPrimAPIMakeTorus {
    /// The maker of `torus` with its pipe range and angle set.
    fn limited(mut torus: BRepPrimTorus, pipe: Option<(f64, f64)>, angle: Option<f64>) -> Result<Self, Failure> {
        if let Some((angle1, angle2)) = pipe {
            torus.set_vmin(angle1)?;
            torus.set_vmax(angle2)?;
        }
        if let Some(angle) = angle {
            torus.set_angle(angle)?;
        }
        Ok(Self { base: BRepBuilderAPIMakeShapeBase::new(), my_torus: torus })
    }

    /// `BRepPrimAPI_MakeTorus(R1, R2)`; `BRepPrimAPIMakeTorus::new(10.0, 2.0)?`.
    pub fn new(r1: f64, r2: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::new(r1, r2)?, None, None)
    }

    /// `BRepPrimAPI_MakeTorus(R1, R2, angle)`; `BRepPrimAPIMakeTorus::with_angle(10.0, 2.0, 3.0)?`.
    pub fn with_angle(r1: f64, r2: f64, angle: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::new(r1, r2)?, None, Some(angle))
    }

    /// `BRepPrimAPI_MakeTorus(R1, R2, angle1, angle2)`;
    /// `BRepPrimAPIMakeTorus::with_pipe(10.0, 2.0, -1.0, 1.0)?`.
    pub fn with_pipe(r1: f64, r2: f64, angle1: f64, angle2: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::new(r1, r2)?, Some((angle1, angle2)), None)
    }

    /// `BRepPrimAPI_MakeTorus(R1, R2, angle1, angle2, angle)`;
    /// `BRepPrimAPIMakeTorus::with_pipe_angle(10.0, 2.0, -1.0, 1.0, 3.0)?`.
    pub fn with_pipe_angle(r1: f64, r2: f64, angle1: f64, angle2: f64, angle: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::new(r1, r2)?, Some((angle1, angle2)), Some(angle))
    }

    /// `BRepPrimAPI_MakeTorus(Axes, R1, R2)`; `BRepPrimAPIMakeTorus::with_axes(axes, 10.0, 2.0)?`.
    pub fn with_axes(axes: Axis2, r1: f64, r2: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::with_axes(axes, r1, r2)?, None, None)
    }

    /// `BRepPrimAPI_MakeTorus(Axes, R1, R2, angle)`;
    /// `BRepPrimAPIMakeTorus::with_axes_angle(axes, 10.0, 2.0, 3.0)?`.
    pub fn with_axes_angle(axes: Axis2, r1: f64, r2: f64, angle: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::with_axes(axes, r1, r2)?, None, Some(angle))
    }

    /// `BRepPrimAPI_MakeTorus(Axes, R1, R2, angle1, angle2)`;
    /// `BRepPrimAPIMakeTorus::with_axes_pipe(axes, 10.0, 2.0, -1.0, 1.0)?`.
    pub fn with_axes_pipe(axes: Axis2, r1: f64, r2: f64, angle1: f64, angle2: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::with_axes(axes, r1, r2)?, Some((angle1, angle2)), None)
    }

    /// `BRepPrimAPI_MakeTorus(Axes, R1, R2, angle1, angle2, angle)`;
    /// `BRepPrimAPIMakeTorus::with_axes_pipe_angle(axes, 10.0, 2.0, -1.0, 1.0, 3.0)?`.
    pub fn with_axes_pipe_angle(axes: Axis2, r1: f64, r2: f64, angle1: f64, angle2: f64, angle: f64) -> Result<Self, Failure> {
        Self::limited(BRepPrimTorus::with_axes(axes, r1, r2)?, Some((angle1, angle2)), Some(angle))
    }

    /// `Torus()`: the algorithm; `maker.torus()`.
    pub fn torus(&mut self) -> &mut BRepPrimTorus {
        &mut self.my_torus
    }
}

one_axis_maker!(BRepPrimAPIMakeTorus, my_torus);
