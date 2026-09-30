// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Vinert.cxx (the
// constructors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepGProp_Vinert`: each sets the location, then runs the
//! Perform of the same arguments.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::vinert::{BRepGPropVinert, VinertBound};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Plane3, Point3};

impl BRepGPropVinert {
    /// `BRepGProp_Vinert()`: an empty framework at the origin, epsilon 0;
    /// `BRepGPropVinert::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn located(location: Point3) -> Self {
        let mut props = Self::new();
        props.set_location(location);
        props
    }

    fn fixed(
        surface: &BRepGPropFace,
        location: Point3,
        bound: VinertBound,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_natural(surface, bound)?;
        Ok(props)
    }

    fn along(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        bound: VinertBound,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_along(surface, domain, bound)?;
        Ok(props)
    }

    fn adaptive(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        (bound, eps): (VinertBound, f64),
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_adaptive(surface, domain, bound, eps)?;
        Ok(props)
    }

    /// `BRepGProp_Vinert(S, VLocation)`; `BRepGPropVinert::with_face(&face, location)?`.
    pub fn with_face(surface: &BRepGPropFace, location: Point3) -> Result<Self, Failure> {
        Self::fixed(surface, location, VinertBound::Location)
    }

    /// `BRepGProp_Vinert(S, VLocation, Eps)`;
    /// `BRepGPropVinert::with_face_eps(&mut face, location, 1e-6)?`.
    pub fn with_face_eps(
        surface: &mut BRepGPropFace,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        let mut empty = BRepGPropDomain::new();
        Self::adaptive(surface, &mut empty, location, (VinertBound::Location, eps))
    }

    /// `BRepGProp_Vinert(S, O, VLocation)`;
    /// `BRepGPropVinert::with_face_origin(&face, o, location)?`.
    pub fn with_face_origin(
        surface: &BRepGPropFace,
        origin: Point3,
        location: Point3,
    ) -> Result<Self, Failure> {
        Self::fixed(surface, location, VinertBound::Origin(origin))
    }

    /// `BRepGProp_Vinert(S, O, VLocation, Eps)`;
    /// `BRepGPropVinert::with_face_origin_eps(&mut face, o, location, 1e-6)?`.
    pub fn with_face_origin_eps(
        surface: &mut BRepGPropFace,
        origin: Point3,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        let mut empty = BRepGPropDomain::new();
        Self::adaptive(
            surface,
            &mut empty,
            location,
            (VinertBound::Origin(origin), eps),
        )
    }

    /// `BRepGProp_Vinert(S, Pl, VLocation)`;
    /// `BRepGPropVinert::with_face_plane(&face, plane, location)?`.
    pub fn with_face_plane(
        surface: &BRepGPropFace,
        plane: Plane3,
        location: Point3,
    ) -> Result<Self, Failure> {
        Self::fixed(surface, location, VinertBound::Plane(plane))
    }

    /// `BRepGProp_Vinert(S, Pl, VLocation, Eps)`;
    /// `BRepGPropVinert::with_face_plane_eps(&mut face, plane, location, 1e-6)?`.
    pub fn with_face_plane_eps(
        surface: &mut BRepGPropFace,
        plane: Plane3,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        let mut empty = BRepGPropDomain::new();
        Self::adaptive(
            surface,
            &mut empty,
            location,
            (VinertBound::Plane(plane), eps),
        )
    }

    /// `BRepGProp_Vinert(S, D, VLocation)`;
    /// `BRepGPropVinert::with_domain(&mut face, &mut domain, location)?`.
    pub fn with_domain(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
    ) -> Result<Self, Failure> {
        Self::along(surface, domain, location, VinertBound::Location)
    }

    /// `BRepGProp_Vinert(S, D, VLocation, Eps)`;
    /// `BRepGPropVinert::with_domain_eps(&mut face, &mut domain, location, 1e-6)?`.
    pub fn with_domain_eps(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        Self::adaptive(surface, domain, location, (VinertBound::Location, eps))
    }

    /// `BRepGProp_Vinert(S, D, O, VLocation)`;
    /// `BRepGPropVinert::with_domain_origin(&mut face, &mut domain, o, location)?`.
    pub fn with_domain_origin(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        origin: Point3,
        location: Point3,
    ) -> Result<Self, Failure> {
        Self::along(surface, domain, location, VinertBound::Origin(origin))
    }

    /// `BRepGProp_Vinert(S, D, O, VLocation, Eps)`;
    /// `BRepGPropVinert::with_domain_origin_eps(&mut face, &mut domain, o, location, 1e-6)?`.
    pub fn with_domain_origin_eps(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        origin: Point3,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        Self::adaptive(
            surface,
            domain,
            location,
            (VinertBound::Origin(origin), eps),
        )
    }

    /// `BRepGProp_Vinert(S, D, Pl, VLocation)`;
    /// `BRepGPropVinert::with_domain_plane(&mut face, &mut domain, plane, location)?`.
    pub fn with_domain_plane(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
        location: Point3,
    ) -> Result<Self, Failure> {
        Self::along(surface, domain, location, VinertBound::Plane(plane))
    }

    /// `BRepGProp_Vinert(S, D, Pl, VLocation, Eps)`;
    /// `BRepGPropVinert::with_domain_plane_eps(&mut face, &mut domain, plane, location, 1e-6)?`.
    pub fn with_domain_plane_eps(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        Self::adaptive(surface, domain, location, (VinertBound::Plane(plane), eps))
    }
}
