// Rust translation of the constructors of BRepLib_MakeEdge2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepLib_MakeEdge2d`: each builds the Geom2d curve of its gp
//! argument, if any, and calls one Init.
use super::BRepLibMakeEdge2d;
use super::project::plane_parameters;
use crate::brep_lib::errors::BRepLibEdgeError;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
};
use rustycad_tkmath::gp::{
    self, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};

/// The four constructors of one gp 2d curve kind: whole, between parameters, points
/// or vertices, each on `Geom2d_X(L)`.
macro_rules! gp_curve2d_constructors {
    ($gp:ty, $native:literal, $geom:expr, $plain:ident, $params:ident, $points:ident, $vertices:ident) => {
        #[doc = concat!("`BRepLib_MakeEdge2d(const ", $native, "& L)`: the whole curve; `BRepLibMakeEdge2d::", stringify!($plain), "(l)?`.")]
        pub fn $plain(curve: $gp) -> Result<Self, Failure> {
            let geom: Handle<dyn Geom2dCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve(&geom))
        }
        #[doc = concat!("`BRepLib_MakeEdge2d(const ", $native, "& L, p1, p2)`; `BRepLibMakeEdge2d::", stringify!($params), "(l, 0.0, 1.0)?`.")]
        pub fn $params(curve: $gp, p1: f64, p2: f64) -> Result<Self, Failure> {
            let geom: Handle<dyn Geom2dCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_params(&geom, p1, p2))
        }
        #[doc = concat!("`BRepLib_MakeEdge2d(const ", $native, "& L, P1, P2)`; `BRepLibMakeEdge2d::", stringify!($points), "(l, a, b)?`.")]
        pub fn $points(curve: $gp, p1: Point2, p2: Point2) -> Result<Self, Failure> {
            let geom: Handle<dyn Geom2dCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_points(&geom, p1, p2))
        }
        #[doc = concat!("`BRepLib_MakeEdge2d(const ", $native, "& L, V1, V2)`; `BRepLibMakeEdge2d::", stringify!($vertices), "(l, &v1, &v2)?`.")]
        pub fn $vertices(curve: $gp, v1: &Vertex, v2: &Vertex) -> Result<Self, Failure> {
            let geom: Handle<dyn Geom2dCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_vertices(&geom, v1, v2))
        }
    };
}

/// The 2d segment from `p1` to `p2` as a Geom2d line and its length, or None when the
/// points are within gp::Resolution().
/// A 2d segment line and its length.
type Segment2d = (Handle<dyn Geom2dCurve>, f64);

fn segment2d(p1: Point2, p2: Point2) -> Result<Option<Segment2d>, Failure> {
    let length: f64 = p1.distance(p2);
    if length <= gp::resolution() {
        return Ok(None);
    }
    let direction = Direction2::from_vector(Vector2::between(p1, p2))?;
    let line = Geom2dLine::from_lin2d(Line2::new(p1, direction));
    Ok(Some((Handle::new(line).cast(), length)))
}

impl BRepLibMakeEdge2d {
    /// The maker of a segment whose ends coincide: `BRepLib_LineThroughIdenticPoints`.
    fn identical_points() -> Self {
        let mut maker = Self::blank();
        maker.my_error = BRepLibEdgeError::LineThroughIdenticPoints;
        maker
    }

    /// `BRepLib_MakeEdge2d(V1, V2)`: the segment between the plane parameters of two
    /// vertices; `BRepLibMakeEdge2d::from_vertices(&v1, &v2)?`.
    pub fn from_vertices(v1: &Vertex, v2: &Vertex) -> Result<Self, Failure> {
        let (p1, p2) = (plane_parameters(v1)?, plane_parameters(v2)?);
        let Some((line, length)) = segment2d(p1, p2)? else {
            return Ok(Self::identical_points());
        };
        Self::initialized(|maker| maker.init_curve_vertices_params(&line, v1, v2, 0.0, length))
    }

    /// `BRepLib_MakeEdge2d(P1, P2)`: the segment between two 2d points;
    /// `BRepLibMakeEdge2d::from_points(a, b)?`.
    pub fn from_points(p1: Point2, p2: Point2) -> Result<Self, Failure> {
        let Some((line, length)) = segment2d(p1, p2)? else {
            return Ok(Self::identical_points());
        };
        Self::initialized(|maker| maker.init_curve_points_params(&line, p1, p2, 0.0, length))
    }

    gp_curve2d_constructors!(
        Line2,
        "gp_Lin2d",
        Geom2dLine::from_lin2d,
        from_lin2d,
        from_lin2d_params,
        from_lin2d_points,
        from_lin2d_vertices
    );
    gp_curve2d_constructors!(
        Circle2,
        "gp_Circ2d",
        Geom2dCircle::from_circ2d,
        from_circ2d,
        from_circ2d_params,
        from_circ2d_points,
        from_circ2d_vertices
    );
    gp_curve2d_constructors!(
        Ellipse2,
        "gp_Elips2d",
        Geom2dEllipse::from_elips2d,
        from_elips2d,
        from_elips2d_params,
        from_elips2d_points,
        from_elips2d_vertices
    );
    gp_curve2d_constructors!(
        Hyperbola2,
        "gp_Hypr2d",
        Geom2dHyperbola::from_hypr2d,
        from_hypr2d,
        from_hypr2d_params,
        from_hypr2d_points,
        from_hypr2d_vertices
    );
    gp_curve2d_constructors!(
        Parabola2,
        "gp_Parab2d",
        Geom2dParabola::from_parab2d,
        from_parab2d,
        from_parab2d_params,
        from_parab2d_points,
        from_parab2d_vertices
    );

    /// `BRepLib_MakeEdge2d(L)`: the whole Geom2d curve;
    /// `BRepLibMakeEdge2d::from_curve(&c)?`.
    pub fn from_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve(curve))
    }

    /// `BRepLib_MakeEdge2d(L, p1, p2)`; `BRepLibMakeEdge2d::from_curve_params(&c, 0.0, 1.0)?`.
    pub fn from_curve_params(
        curve: &Handle<dyn Geom2dCurve>,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_params(curve, p1, p2))
    }

    /// `BRepLib_MakeEdge2d(L, P1, P2)`; `BRepLibMakeEdge2d::from_curve_points(&c, a, b)?`.
    pub fn from_curve_points(
        curve: &Handle<dyn Geom2dCurve>,
        p1: Point2,
        p2: Point2,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_points(curve, p1, p2))
    }

    /// `BRepLib_MakeEdge2d(L, V1, V2)`;
    /// `BRepLibMakeEdge2d::from_curve_vertices(&c, &v1, &v2)?`.
    pub fn from_curve_vertices(
        curve: &Handle<dyn Geom2dCurve>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_vertices(curve, v1, v2))
    }

    /// `BRepLib_MakeEdge2d(L, P1, P2, p1, p2)`;
    /// `BRepLibMakeEdge2d::from_curve_points_params(&c, a, b, 0.0, 1.0)?`.
    pub fn from_curve_points_params(
        curve: &Handle<dyn Geom2dCurve>,
        point1: Point2,
        point2: Point2,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_points_params(curve, point1, point2, p1, p2))
    }

    /// `BRepLib_MakeEdge2d(L, V1, V2, p1, p2)`;
    /// `BRepLibMakeEdge2d::from_curve_vertices_params(&c, &v1, &v2, 0.0, 1.0)?`.
    pub fn from_curve_vertices_params(
        curve: &Handle<dyn Geom2dCurve>,
        v1: &Vertex,
        v2: &Vertex,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_vertices_params(curve, v1, v2, p1, p2))
    }
}
