// Rust translation of the SurfaceProperties class of BRepLib.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SurfaceProperties`: the surface properties at the points of a curve on surface,
//! for the continuity decision of `BRepLib::ContinuityOfFaces`.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::gp::{Direction3, Point3, Transform3, Vector2, Vector3};

/// "Structure for calculation of properties, necessary for decision about continuity".
pub(super) struct SurfaceProperties {
    // properties calculator
    surface_props: SLProps,
    surface_trsf: Transform3,
    curve2d: Handle<dyn Geom2dCurve>,
    // the face based on the surface is reversed
    is_reversed: bool,
    // tangent vector to Pcurve in UV
    curve_tangent: Vector2,
}

impl SurfaceProperties {
    /// The properties of order 2 at Precision::Confusion() on `surface`.
    pub(super) fn new(
        surface: &Handle<dyn GeomSurface>,
        surface_trsf: Transform3,
        curve2d: &Handle<dyn Geom2dCurve>,
        reversed: bool,
    ) -> Result<Self, Failure> {
        Ok(Self {
            surface_props: SLProps::new(surface.clone(), 2, precision::confusion())?,
            surface_trsf,
            curve2d: curve2d.clone(),
            is_reversed: reversed,
            curve_tangent: Vector2::default(),
        })
    }

    /// `Calculate`: derivatives on the surface at the point of the curve.
    pub(super) fn calculate(&mut self, param_on_curve: f64) -> Result<(), Failure> {
        let curve = self.curve2d.get().ok_or_else(|| {
            crate::brep_lib::errors::null_object(
                "BRepLib::ContinuityOfFaces: null curve on surface",
            )
        })?;
        let (uv, tangent) = curve.d1(param_on_curve)?;
        self.curve_tangent = tangent;
        self.surface_props.set_parameters(uv.x(), uv.y())
    }

    /// `Value`: the point just calculated.
    pub(super) fn value(&self) -> Point3 {
        self.surface_props.value().transformed(self.surface_trsf)
    }

    /// `Derivative`: the surface derivative orthogonal to the curve tangent (a null
    /// vector for a tangent shorter than Precision::Confusion()).
    pub(super) fn derivative(&mut self) -> Result<Vector3, Failure> {
        let mut deriv = Vector3::default();
        // direction orthogonal to tangent vector of the curve
        let mut ortho = Vector2::new(-self.curve_tangent.y(), self.curve_tangent.x());
        let len: f64 = ortho.magnitude();
        if len < precision::confusion() {
            return Ok(deriv);
        }
        ortho /= len;
        if self.is_reversed {
            ortho.reverse();
        }
        let (d1u, d1v) = (self.surface_props.d1u()?, self.surface_props.d1v()?);
        deriv.set_linear_form(ortho.x(), d1u, ortho.y(), d1v);
        Ok(deriv.transformed(self.surface_trsf))
    }

    /// `Normal`: the surface normal (LProp_NotDefined when undefined).
    pub(super) fn normal(&mut self) -> Result<Direction3, Failure> {
        Ok(self.surface_props.normal()?.transformed(self.surface_trsf))
    }

    /// `Curvature`: the principal directions and curvatures (maximal, then minimal),
    /// negated for a reversed face and for a negative transformation.
    pub(super) fn curvature(&mut self) -> Result<([Direction3; 2], [f64; 2]), Failure> {
        let (mut dir1, mut dir2) = self.surface_props.curvature_directions()?;
        let mut curvature1: f64 = self.surface_props.max_curvature()?;
        let mut curvature2: f64 = self.surface_props.min_curvature()?;
        if self.is_reversed {
            curvature1 = -curvature1;
            curvature2 = -curvature2;
        }
        if self.surface_trsf.is_negative() {
            curvature1 = -curvature1;
            curvature2 = -curvature2;
        }
        dir1.transform(self.surface_trsf);
        dir2.transform(self.surface_trsf);
        Ok(([dir1, dir2], [curvature1, curvature2]))
    }
}
