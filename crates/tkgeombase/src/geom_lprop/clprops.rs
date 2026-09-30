// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_CLProps.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLProp_CLPropsBase`: point, derivatives, tangent, curvature, normal and
//! centre of curvature of a curve at one parameter.
use crate::lprop::Status;
use crate::lprop::curve_access::{CurveAccess, DirectAccess, PropsVector};
use crate::lprop::curve_utils::{self, CurveProps, CurvePropsView};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{Vector2, Vector3};
use std::marker::PhantomData;

/// `GeomLProp_CLPropsBase<Pnt, Vec, Dir, CurveType, Access>`: the local properties
/// of a curve `C` at a parameter, computed lazily and cached.
///
/// `V` is the derivative type (`gp_Vec` or `gp_Vec2d`, which fix the point and
/// direction types); `A` evaluates the curve. The members that evaluate the curve
/// return its exceptions as `Err`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkgeombase::geom_lprop::CLProps2d;
/// use rustycad_tkmath::gp::{Axis2d, Circle2};
/// let circle = Circle2::from_x_axis(Axis2d::default(), 5.0, true).unwrap();
/// let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(circle)).cast();
/// let mut props = CLProps2d::with_parameter(curve, 0.0, 2, 1e-7).unwrap();
/// assert!((props.curvature().unwrap() - 0.2).abs() < 1e-12);
/// ```
pub struct CLPropsBase<V: PropsVector, C, A = DirectAccess> {
    my_curve: C,
    my_u: f64,
    my_der_order: i32,
    my_cn: i32,
    my_lin_tol: f64,
    my_pnt: V::Point,
    my_deriv_arr: [V; 3],
    my_curvature: f64,
    my_tangent_status: Status,
    my_significant_first_derivative_order: i32,
    access: PhantomData<A>,
}

/// `GeomLProp_CLProps`: the curve local properties of a `Geom_Curve` handle.
pub type CLProps = CLPropsBase<Vector3, Handle<dyn GeomCurve>>;
/// `GeomLProp_CLProps2d`: the curve local properties of a `Geom2d_Curve` handle.
pub type CLProps2d = CLPropsBase<Vector2, Handle<dyn Geom2dCurve>>;

// The native `Standard_OutOfRange_Raise_if(N < 0 || N > 3, ...)` of the constructors.
fn check_level(n: i32) -> Result<(), Failure> {
    if !(0..=3).contains(&n) {
        return Err(Failure::out_of_range(&format!(
            "GeomLProp_CLProps::GeomLProp_CLProps(): N = {n}; expected in [0, 3]"
        )));
    }
    Ok(())
}

impl<V: PropsVector, C, A: CurveAccess<C, V>> CLPropsBase<V, C, A> {
    /// `GeomLProp_CLPropsBase(C, N, Resolution)`: properties of `curve` up to
    /// derivative `n` (0 to 3, else Standard_OutOfRange), with `resolution` as the
    /// null-vector tolerance; no parameter is set; `CLProps::new(curve, 2, 1e-7)?`.
    pub fn new(curve: C, n: i32, resolution: f64) -> Result<Self, Failure> {
        check_level(n)?;
        Ok(Self::unchecked(curve, n, 4, resolution))
    }

    /// `GeomLProp_CLPropsBase(C, U, N, Resolution)`: as [`Self::new`], at the
    /// parameter `u`; `CLProps::with_parameter(curve, 0.5, 2, 1e-7)?`.
    pub fn with_parameter(curve: C, u: f64, n: i32, resolution: f64) -> Result<Self, Failure> {
        check_level(n)?;
        let mut props: Self = Self::unchecked(curve, n, 4, resolution);
        props.set_parameter(u)?;
        Ok(props)
    }

    /// `GeomLProp_CLPropsBase(N, Resolution)`: properties without a curve (a
    /// default `C`, a null handle); no tangent is defined until
    /// [`Self::set_curve`]; `CLProps::without_curve(2, 1e-7)?`.
    pub fn without_curve(n: i32, resolution: f64) -> Result<Self, Failure>
    where
        C: Default,
    {
        check_level(n)?;
        Ok(Self::unchecked(C::default(), n, 0, resolution))
    }

    fn unchecked(curve: C, n: i32, cn: i32, resolution: f64) -> Self {
        Self {
            my_curve: curve,
            my_u: f64::MAX,
            my_der_order: n,
            my_cn: cn,
            my_lin_tol: resolution,
            my_pnt: V::Point::default(),
            my_deriv_arr: [V::default(); 3],
            my_curvature: 0.0,
            my_tangent_status: Status::Undecided,
            my_significant_first_derivative_order: 0,
            access: PhantomData,
        }
    }

    /// `SetParameter(U)`: evaluate the point and the derivatives at `u`;
    /// `props.set_parameter(0.5)?`.
    pub fn set_parameter(&mut self, u: f64) -> Result<(), Failure> {
        curve_utils::set_parameter::<A, C, V>(
            &self.my_curve,
            u,
            &mut self.my_u,
            self.my_der_order,
            &mut self.my_pnt,
            &mut self.my_deriv_arr,
            &mut self.my_tangent_status,
        )
    }

    /// `SetCurve(C)`: replace the curve; the parameter must be set again;
    /// `props.set_curve(curve)`.
    pub fn set_curve(&mut self, curve: C) {
        self.my_curve = curve;
        self.my_cn = 4;
    }

    /// `Value()`: the point at the current parameter; `props.value()`.
    pub fn value(&self) -> V::Point {
        self.my_pnt
    }

    /// `D1()`: the first derivative, computed if not yet; `props.d1()?`.
    pub fn d1(&mut self) -> Result<V, Failure> {
        self.ensure(1)
    }

    /// `D2()`: the second derivative, computed if not yet; `props.d2()?`.
    pub fn d2(&mut self) -> Result<V, Failure> {
        self.ensure(2)
    }

    /// `D3()`: the third derivative, computed if not yet; `props.d3()?`.
    pub fn d3(&mut self) -> Result<V, Failure> {
        self.ensure(3)
    }

    fn ensure(&mut self, required: i32) -> Result<V, Failure> {
        curve_utils::ensure_deriv::<A, C, V>(
            &self.my_curve,
            self.my_u,
            &mut self.my_der_order,
            required,
            &mut self.my_pnt,
            &mut self.my_deriv_arr,
        )
    }

    /// `IsTangentDefined()`: whether a derivative up to order 3 is longer than the
    /// resolution; `props.is_tangent_defined()?`.
    pub fn is_tangent_defined(&mut self) -> Result<bool, Failure> {
        let (cn, lin_tol): (i32, f64) = (self.my_cn, self.my_lin_tol);
        let mut order: i32 = self.my_significant_first_derivative_order;
        let mut status: Status = self.my_tangent_status;
        let defined = curve_utils::is_tangent_defined(self, cn, lin_tol, &mut order, &mut status);
        self.my_significant_first_derivative_order = order;
        self.my_tangent_status = status;
        defined
    }

    /// `Tangent(D)`: the tangent direction, LProp_NotDefined when it is not
    /// defined; `props.tangent()?`.
    pub fn tangent(&mut self) -> Result<V::Direction, Failure> {
        curve_utils::tangent::<A, Self, V>(self)
    }

    /// `Curvature()`: the curvature, RealLast() when the first derivative is null,
    /// LProp_NotDefined without tangent; `props.curvature()?`.
    pub fn curvature(&mut self) -> Result<f64, Failure> {
        let lin_tol: f64 = self.my_lin_tol;
        let mut stored: f64 = self.my_curvature;
        let curvature = curve_utils::curvature(self, lin_tol, &mut stored);
        self.my_curvature = stored;
        curvature
    }

    /// `Normal(N)`: the principal normal, LProp_NotDefined for a null or infinite
    /// curvature; `props.normal()?`.
    pub fn normal(&mut self) -> Result<V::Direction, Failure> {
        let lin_tol: f64 = self.my_lin_tol;
        curve_utils::normal(self, lin_tol)
    }

    /// `CentreOfCurvature(P)`: the centre of curvature, LProp_NotDefined for a null
    /// curvature; `props.centre_of_curvature()?`.
    pub fn centre_of_curvature(&mut self) -> Result<V::Point, Failure> {
        let lin_tol: f64 = self.my_lin_tol;
        curve_utils::centre_of_curvature(self, lin_tol)
    }
}

impl<V: PropsVector, C, A: CurveAccess<C, V>> CurveProps<V> for CLPropsBase<V, C, A> {
    type Curve = C;
    fn props_curve(&self) -> &C {
        &self.my_curve
    }
    fn view(&self) -> CurvePropsView<V> {
        CurvePropsView {
            u: self.my_u,
            point: self.my_pnt,
            derivatives: self.my_deriv_arr,
            significant_order: self.my_significant_first_derivative_order,
            curvature: self.my_curvature,
        }
    }
    fn d1(&mut self) -> Result<V, Failure> {
        self.ensure(1)
    }
    fn d2(&mut self) -> Result<V, Failure> {
        self.ensure(2)
    }
    fn d3(&mut self) -> Result<V, Failure> {
        self.ensure(3)
    }
    fn is_tangent_defined(&mut self) -> Result<bool, Failure> {
        CLPropsBase::is_tangent_defined(self)
    }
    fn curvature(&mut self) -> Result<f64, Failure> {
        CLPropsBase::curvature(self)
    }
}
