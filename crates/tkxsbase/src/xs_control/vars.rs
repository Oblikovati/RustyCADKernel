// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Vars (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_Vars`: the named variables of the exchange session (transients, points,
//! shapes); the DRAW session redefines them on its own variables.
use rustycad_tkbrep::topods::{HShape, Shape};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::DataMap,
    standard::{Handle, SharedState, StandardTransient, Transient},
    tcollection::AsciiString,
};
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dPoint};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCurve, GeomGeometry, GeomPoint, GeomSurface};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::gp::Point3;

/// `XSControl_Vars`: the variables by name.
///
/// ```
/// use rustycad_tkxsbase::xs_control::{XSControlVars, XSControlVarsOps};
/// use rustycad_tkmath::gp::Point3;
/// let vars = XSControlVars::new();
/// vars.set_point("p", &Point3::new(1.0, 2.0, 3.0));
/// assert_eq!(vars.get_point(&mut String::from("p")).unwrap().x(), 1.0);
/// ```
#[derive(Default)]
pub struct XSControlVars {
    transient: StandardTransient,
    thevars: SharedState<DataMap<AsciiString, Handle<StandardTransient>>>,
}
impl_transient!(XSControlVars, "XSControl_Vars", StandardTransient, transient, interfaces: [dyn XSControlVarsOps]);

impl XSControlVars {
    /// `XSControl_Vars()`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The virtual members of `XSControl_Vars` (all of them are virtual natively). The name
/// is in-out (`const char*&` natively): an override may change it.
pub trait XSControlVarsOps: Transient {
    /// The embedded variables.
    fn vars(&self) -> &XSControlVars;
    /// `Set(name, val)`.
    fn set(&self, name: &str, val: &Handle<StandardTransient>) {
        self.vars()
            .thevars
            .write()
            .bind(AsciiString::new(name), val.clone());
    }
    /// `Get(name)`: null when unknown.
    fn get(&self, name: &mut String) -> Handle<StandardTransient> {
        self.vars()
            .thevars
            .read()
            .seek(&AsciiString::new(name))
            .cloned()
            .unwrap_or_default()
    }
    /// `GetGeom(name)`: null when not a geometry.
    fn get_geom(&self, name: &mut String) -> Handle<dyn GeomGeometry> {
        self.get(name).cast()
    }
    /// `GetCurve2d(name)`: null when not a 2D curve.
    fn get_curve2d(&self, name: &mut String) -> Handle<dyn Geom2dCurve> {
        self.get(name).cast()
    }
    /// `GetCurve(name)`: null when not a curve.
    fn get_curve(&self, name: &mut String) -> Handle<dyn GeomCurve> {
        self.get(name).cast()
    }
    /// `GetSurface(name)`: null when not a surface.
    fn get_surface(&self, name: &mut String) -> Handle<dyn GeomSurface> {
        self.get(name).cast()
    }
    /// `SetPoint(name, val)`: a cartesian point.
    fn set_point(&self, name: &str, val: &Point3) {
        self.set(
            name,
            &Handle::new(GeomCartesianPoint::from_pnt(*val)).cast(),
        );
    }
    /// `SetPoint2d(name, val)`: a 2D cartesian point.
    fn set_point2d(&self, name: &str, val: &Point2) {
        self.set(
            name,
            &Handle::new(Geom2dCartesianPoint::from_pnt2d(*val)).cast(),
        );
    }
    /// `GetPoint(name, pnt)`: the point, `None` when not a cartesian point.
    fn get_point(&self, name: &mut String) -> Option<Point3> {
        self.get(name)
            .cast::<GeomCartesianPoint>()
            .get()
            .map(GeomPoint::pnt)
    }
    /// `GetPoint2d(name, pnt)`: the point, `None` when not a 2D cartesian point.
    fn get_point2d(&self, name: &mut String) -> Option<Point2> {
        self.get(name)
            .cast::<Geom2dCartesianPoint>()
            .get()
            .map(Geom2dPoint::pnt2d)
    }
    /// `SetShape(name, val)`: the shape as a handle shape.
    fn set_shape(&self, name: &str, val: &Shape) {
        self.set(name, &Handle::new(HShape::new(val)).cast());
    }
    /// `GetShape(name)`: null when not a shape.
    fn get_shape(&self, name: &mut String) -> Shape {
        self.get(name)
            .cast::<HShape>()
            .get()
            .map(HShape::shape)
            .unwrap_or_default()
    }
}
impl_transient_interface!(dyn XSControlVarsOps, "XSControl_Vars", StandardTransient);

impl XSControlVarsOps for XSControlVars {
    fn vars(&self) -> &XSControlVars {
        self
    }
}
