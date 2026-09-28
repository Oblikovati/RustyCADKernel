// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/LProp/LProp_CurveUtils.hxx (access policies)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The access policies of `LProp_CurveUtils` and the geometry they evaluate.
//!
//! The native template parameters `Pnt`, `Vec` and `Dir` become the [`PropsVector`]
//! trait (with its point and direction types), implemented by `gp_Vec` and
//! `gp_Vec2d`. The policies `DirectAccess` (the curve's own D0...D3) and
//! `ToolAccess<Tool>` (static members of a tool type) implement [`CurveAccess`].
//! The native `Deref` helpers only unwrap a handle; a Rust handle derefs itself, and
//! a null one gives Standard_NullObject instead of the native null dereference.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Vector2, Vector3};
use std::marker::PhantomData;
use std::ops::{Mul, Neg, Sub};

/// The vector operations of the property algorithms, with the matching point and
/// direction types (the native `Pnt`/`Vec`/`Dir` template parameters);
/// `<Vector3 as PropsVector>::between(p1, p2)`.
pub trait PropsVector:
    Copy + Default + Mul<f64, Output = Self> + Sub<Output = Self> + Neg<Output = Self>
{
    /// The point type (`gp_Pnt` or `gp_Pnt2d`).
    type Point: Copy + Default;
    /// The direction type (`gp_Dir` or `gp_Dir2d`).
    type Direction: Copy + Default;
    /// `SquareMagnitude()`; `v.square_magnitude()`.
    fn square_magnitude(self) -> f64;
    /// `Dot(Other)`; `v.dot(w)`.
    fn dot(self, other: Self) -> f64;
    /// `CrossSquareMagnitude(Other)`; `v.cross_square_magnitude(w)`.
    fn cross_square_magnitude(self, other: Self) -> f64;
    /// `Vec(P1, P2)`; `Vector3::between(p1, p2)`.
    fn between(first: Self::Point, second: Self::Point) -> Self;
    /// `Dir(V)`, Standard_ConstructionError for a null vector; `v.to_direction()?`.
    fn to_direction(self) -> Result<Self::Direction, Failure>;
    /// `Normalize()`, Standard_ConstructionError for a null vector; `v.normalize()?`.
    fn normalize(&mut self) -> Result<(), Failure>;
    /// `Divide(Scalar)`; `v.divide(2.0)`.
    fn divide(&mut self, scalar: f64);
    /// `P.Translated(V)`; `Vector3::translate(p, v)`.
    fn translate(point: Self::Point, vector: Self) -> Self::Point;
}

// `$square` reads the square magnitude: gp_Vec takes `&self`, gp_Vec2d `self`.
macro_rules! props_vector {
    ($vector:ty, $point:ty, $direction:ty, $square:expr) => {
        impl PropsVector for $vector {
            type Point = $point;
            type Direction = $direction;
            fn square_magnitude(self) -> f64 {
                $square(self)
            }
            fn dot(self, other: Self) -> f64 {
                <$vector>::dot(self, other)
            }
            fn cross_square_magnitude(self, other: Self) -> f64 {
                <$vector>::cross_square_magnitude(self, other)
            }
            fn between(first: $point, second: $point) -> Self {
                <$vector>::between(first, second)
            }
            fn to_direction(self) -> Result<$direction, Failure> {
                <$direction>::from_vector(self)
            }
            fn normalize(&mut self) -> Result<(), Failure> {
                <$vector>::normalize(self)
            }
            fn divide(&mut self, scalar: f64) {
                *self /= scalar;
            }
            fn translate(point: $point, vector: Self) -> $point {
                point.translated(vector)
            }
        }
    };
}
props_vector!(Vector3, Point3, Direction3, |v: Vector3| {
    Vector3::square_magnitude(&v)
});
props_vector!(Vector2, Point2, Direction2, Vector2::square_magnitude);

/// Point and first derivative of a curve.
pub type CurveD1<V> = (<V as PropsVector>::Point, V);
/// Point and first two derivatives of a curve.
pub type CurveD2<V> = (<V as PropsVector>::Point, V, V);
/// Point and first three derivatives of a curve.
pub type CurveD3<V> = (<V as PropsVector>::Point, V, V, V);

/// An access policy: how the curve property algorithms evaluate a curve of type
/// `C` (the native `Access` template parameter);
/// `<DirectAccess as CurveAccess<_, Vector3>>::d0(&curve, 0.5)?`.
pub trait CurveAccess<C, V: PropsVector> {
    /// Point at `u` (`Access::D0`).
    fn d0(curve: &C, u: f64) -> Result<V::Point, Failure>;
    /// Point and first derivative (`Access::D1`).
    fn d1(curve: &C, u: f64) -> Result<CurveD1<V>, Failure>;
    /// Point and two derivatives (`Access::D2`).
    fn d2(curve: &C, u: f64) -> Result<CurveD2<V>, Failure>;
    /// Point and three derivatives (`Access::D3`).
    fn d3(curve: &C, u: f64) -> Result<CurveD3<V>, Failure>;
    /// First parameter (`Access::FirstParameter`).
    fn first_parameter(curve: &C) -> Result<f64, Failure>;
    /// Last parameter (`Access::LastParameter`).
    fn last_parameter(curve: &C) -> Result<f64, Failure>;
}

/// A curve that evaluates itself, as `DirectAccess` requires (a Geom or Geom2d
/// curve handle, an Adaptor3d or Adaptor2d curve handle); `curve.props_d0(0.5)?`.
pub trait DirectCurve {
    /// The derivative type of the curve.
    type Vector: PropsVector;
    /// `D0(U, P)`.
    fn props_d0(&self, u: f64) -> Result<<Self::Vector as PropsVector>::Point, Failure>;
    /// `D1(U, P, V1)`.
    fn props_d1(&self, u: f64) -> Result<CurveD1<Self::Vector>, Failure>;
    /// `D2(U, P, V1, V2)`.
    fn props_d2(&self, u: f64) -> Result<CurveD2<Self::Vector>, Failure>;
    /// `D3(U, P, V1, V2, V3)`.
    fn props_d3(&self, u: f64) -> Result<CurveD3<Self::Vector>, Failure>;
    /// `FirstParameter()`.
    fn props_first_parameter(&self) -> Result<f64, Failure>;
    /// `LastParameter()`.
    fn props_last_parameter(&self) -> Result<f64, Failure>;
}

/// `LProp_CurveUtils::DirectAccess`: evaluates the curve through its own members;
/// `CLPropsBase::<Vector3, Handle<dyn GeomCurve>, DirectAccess>`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DirectAccess;

impl<C: DirectCurve> CurveAccess<C, C::Vector> for DirectAccess {
    fn d0(curve: &C, u: f64) -> Result<<C::Vector as PropsVector>::Point, Failure> {
        curve.props_d0(u)
    }
    fn d1(curve: &C, u: f64) -> Result<CurveD1<C::Vector>, Failure> {
        curve.props_d1(u)
    }
    fn d2(curve: &C, u: f64) -> Result<CurveD2<C::Vector>, Failure> {
        curve.props_d2(u)
    }
    fn d3(curve: &C, u: f64) -> Result<CurveD3<C::Vector>, Failure> {
        curve.props_d3(u)
    }
    fn first_parameter(curve: &C) -> Result<f64, Failure> {
        curve.props_first_parameter()
    }
    fn last_parameter(curve: &C) -> Result<f64, Failure> {
        curve.props_last_parameter()
    }
}

/// The static tool interface of `ToolAccess` (the HLRBRep curve tools natively):
/// `Tool::Value`, `D1`, `D2`, `D3`, `FirstParameter` and `LastParameter`.
pub trait CurveTool<C> {
    /// The derivative type of the curve.
    type Vector: PropsVector;
    /// `Tool::Value(C, U, P)`.
    fn value(curve: &C, u: f64) -> Result<<Self::Vector as PropsVector>::Point, Failure>;
    /// `Tool::D1(C, U, P, V1)`.
    fn d1(curve: &C, u: f64) -> Result<CurveD1<Self::Vector>, Failure>;
    /// `Tool::D2(C, U, P, V1, V2)`.
    fn d2(curve: &C, u: f64) -> Result<CurveD2<Self::Vector>, Failure>;
    /// `Tool::D3(C, U, P, V1, V2, V3)`.
    fn d3(curve: &C, u: f64) -> Result<CurveD3<Self::Vector>, Failure>;
    /// `Tool::FirstParameter(C)`.
    fn first_parameter(curve: &C) -> Result<f64, Failure>;
    /// `Tool::LastParameter(C)`.
    fn last_parameter(curve: &C) -> Result<f64, Failure>;
}

/// `LProp_CurveUtils::ToolAccess<Tool>`: evaluates the curve through the static
/// members of `Tool`; `CLPropsBase::<Vector3, MyCurve, ToolAccess<MyTool>>`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolAccess<Tool>(PhantomData<Tool>);

impl<C, Tool: CurveTool<C>> CurveAccess<C, Tool::Vector> for ToolAccess<Tool> {
    fn d0(curve: &C, u: f64) -> Result<<Tool::Vector as PropsVector>::Point, Failure> {
        Tool::value(curve, u)
    }
    fn d1(curve: &C, u: f64) -> Result<CurveD1<Tool::Vector>, Failure> {
        Tool::d1(curve, u)
    }
    fn d2(curve: &C, u: f64) -> Result<CurveD2<Tool::Vector>, Failure> {
        Tool::d2(curve, u)
    }
    fn d3(curve: &C, u: f64) -> Result<CurveD3<Tool::Vector>, Failure> {
        Tool::d3(curve, u)
    }
    fn first_parameter(curve: &C) -> Result<f64, Failure> {
        Tool::first_parameter(curve)
    }
    fn last_parameter(curve: &C) -> Result<f64, Failure> {
        Tool::last_parameter(curve)
    }
}

/// The object of a curve handle; Standard_NullObject for a null handle, where the
/// native `Deref` dereferences a null pointer.
pub(crate) fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "LProp: null curve or surface handle; expected a loaded handle",
            "",
        )
    })
}

// Geom and Geom2d curves: infallible parameter range.
macro_rules! direct_geom_curve {
    ($curve:ty, $vector:ty) => {
        impl DirectCurve for Handle<$curve> {
            type Vector = $vector;
            fn props_d0(&self, u: f64) -> Result<<$vector as PropsVector>::Point, Failure> {
                live(self)?.d0(u)
            }
            fn props_d1(&self, u: f64) -> Result<CurveD1<$vector>, Failure> {
                live(self)?.d1(u)
            }
            fn props_d2(&self, u: f64) -> Result<CurveD2<$vector>, Failure> {
                live(self)?.d2(u)
            }
            fn props_d3(&self, u: f64) -> Result<CurveD3<$vector>, Failure> {
                live(self)?.d3(u)
            }
            fn props_first_parameter(&self) -> Result<f64, Failure> {
                Ok(live(self)?.first_parameter())
            }
            fn props_last_parameter(&self) -> Result<f64, Failure> {
                Ok(live(self)?.last_parameter())
            }
        }
    };
}
direct_geom_curve!(dyn GeomCurve, Vector3);
direct_geom_curve!(dyn Geom2dCurve, Vector2);

// Adaptor curves: every member can raise.
macro_rules! direct_adaptor_curve {
    ($curve:ty, $vector:ty) => {
        impl DirectCurve for Handle<$curve> {
            type Vector = $vector;
            fn props_d0(&self, u: f64) -> Result<<$vector as PropsVector>::Point, Failure> {
                live(self)?.d0(u)
            }
            fn props_d1(&self, u: f64) -> Result<CurveD1<$vector>, Failure> {
                live(self)?.d1(u)
            }
            fn props_d2(&self, u: f64) -> Result<CurveD2<$vector>, Failure> {
                live(self)?.d2(u)
            }
            fn props_d3(&self, u: f64) -> Result<CurveD3<$vector>, Failure> {
                live(self)?.d3(u)
            }
            fn props_first_parameter(&self) -> Result<f64, Failure> {
                live(self)?.first_parameter()
            }
            fn props_last_parameter(&self) -> Result<f64, Failure> {
                live(self)?.last_parameter()
            }
        }
    };
}
direct_adaptor_curve!(dyn Adaptor3dCurve, Vector3);
direct_adaptor_curve!(dyn Adaptor2dCurve2d, Vector2);
