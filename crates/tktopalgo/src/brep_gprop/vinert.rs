// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Vinert.hxx/.cxx
// (the class and its Perform members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Vinert`: the global properties of the region between a face and a point
//! or a plane. The constructors are in `vinert_new.rs`.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::gauss::{BRepGPropGauss, GaussType};
use super::sinert::{gauss_props, store_gauss_props};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Plane3, Point3};
use std::ops::Deref;

/// `BRepGProp_Vinert`: the volume, centre and inertia of the region delimited by a
/// face and the location, a point `O` or a plane (a `GProp_GProps`,
/// docs/topalgo-design.md section 9); the inherited members are reached through
/// `Deref`. By plane, the mass and centre are 0 (the native `convert`).
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropVinert;
/// let mut props = BRepGPropVinert::new();
/// assert_eq!(props.get_epsilon(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepGPropVinert {
    pub(super) base: GProps,
    pub(super) epsilon: f64,
}

impl Deref for BRepGPropVinert {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

/// What delimits the region with the face: the location, a point or a plane.
#[derive(Clone, Copy, Debug)]
pub(super) enum VinertBound {
    /// The location itself (`aCoeff = {0, 0, 0}`).
    Location,
    /// A point `O`: the inertia is computed about it.
    Origin(Point3),
    /// A plane.
    Plane(Plane3),
}

impl BRepGPropVinert {
    /// `SetLocation(VLocation)`; `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `GetEpsilon()`: the error of the last adaptive Perform, 1 after a fixed one;
    /// `props.get_epsilon()`.
    pub fn get_epsilon(&mut self) -> f64 {
        self.epsilon
    }

    /// The coefficients of `Compute`: the shift of the point from the location (by
    /// point) or the plane equation moved to the location (by plane).
    fn coefficients(&self, bound: VinertBound) -> Result<(Vec<f64>, bool), Failure> {
        let loc = self.base.loc;
        Ok(match bound {
            VinertBound::Location => (vec![0.0, 0.0, 0.0], true),
            VinertBound::Origin(o) => (
                vec![o.x() - loc.x(), o.y() - loc.y(), o.z() - loc.z()],
                true,
            ),
            VinertBound::Plane(plane) => {
                let [a, b, c, d] = plane.coefficients()?;
                (
                    vec![a, b, c, d - a * loc.x() - b * loc.y() - c * loc.z()],
                    false,
                )
            }
        })
    }

    /// The fixed-order Perform on the natural bounds; epsilon 1.
    pub(super) fn perform_natural(
        &mut self,
        surface: &BRepGPropFace,
        bound: VinertBound,
    ) -> Result<(), Failure> {
        let (coeff, by_point) = self.coefficients(bound)?;
        self.epsilon = 1.0;
        let mut out = gauss_props(&self.base);
        let mut gauss = BRepGPropGauss::new(GaussType::Vinert);
        gauss.compute_natural(surface, self.base.loc, &coeff, by_point, &mut out)?;
        store_gauss_props(&mut self.base, &out);
        Ok(())
    }

    /// The fixed-order Perform along the edges of the domain; epsilon 1.
    pub(super) fn perform_along(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        bound: VinertBound,
    ) -> Result<(), Failure> {
        let (coeff, by_point) = self.coefficients(bound)?;
        self.epsilon = 1.0;
        let mut out = gauss_props(&self.base);
        let mut gauss = BRepGPropGauss::new(GaussType::Vinert);
        gauss.compute_domain(surface, domain, self.base.loc, &coeff, by_point, &mut out)?;
        store_gauss_props(&mut self.base, &out);
        Ok(())
    }

    /// The adaptive Perform; the reached error, kept as the epsilon.
    pub(super) fn perform_adaptive(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        bound: VinertBound,
        eps: f64,
    ) -> Result<f64, Failure> {
        let (coeff, by_point) = self.coefficients(bound)?;
        let mut out = gauss_props(&self.base);
        let mut gauss = BRepGPropGauss::new(GaussType::Vinert);
        let location = self.base.loc;
        let error =
            gauss.compute_adaptive(surface, domain, location, eps, &coeff, by_point, &mut out)?;
        store_gauss_props(&mut self.base, &out);
        self.epsilon = error;
        Ok(error)
    }

    /// `Perform(S)`: the region between the naturally bounded face and the location;
    /// `props.perform(&face)?`.
    pub fn perform(&mut self, surface: &BRepGPropFace) -> Result<(), Failure> {
        self.perform_natural(surface, VinertBound::Location)
    }

    /// `Perform(S, Eps)`: adaptive, with an empty domain; `props.perform_eps(&mut face, 1e-6)?`.
    pub fn perform_eps(&mut self, surface: &mut BRepGPropFace, eps: f64) -> Result<f64, Failure> {
        let mut empty = BRepGPropDomain::new();
        self.perform_adaptive(surface, &mut empty, VinertBound::Location, eps)
    }

    /// `Perform(S, O)`: the inertia about `origin`; `props.perform_origin(&face, o)?`.
    pub fn perform_origin(
        &mut self,
        surface: &BRepGPropFace,
        origin: Point3,
    ) -> Result<(), Failure> {
        self.perform_natural(surface, VinertBound::Origin(origin))
    }

    /// `Perform(S, O, Eps)`: adaptive about `origin`, with an empty domain;
    /// `props.perform_origin_eps(&mut face, o, 1e-6)?`.
    pub fn perform_origin_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        origin: Point3,
        eps: f64,
    ) -> Result<f64, Failure> {
        let mut empty = BRepGPropDomain::new();
        self.perform_adaptive(surface, &mut empty, VinertBound::Origin(origin), eps)
    }

    /// `Perform(S, Pl)`: the region between the face and `plane`;
    /// `props.perform_plane(&face, plane)?`.
    pub fn perform_plane(&mut self, surface: &BRepGPropFace, plane: Plane3) -> Result<(), Failure> {
        self.perform_natural(surface, VinertBound::Plane(plane))
    }

    /// `Perform(S, Pl, Eps)`: adaptive, with an empty domain;
    /// `props.perform_plane_eps(&mut face, plane, 1e-6)?`.
    pub fn perform_plane_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        plane: Plane3,
        eps: f64,
    ) -> Result<f64, Failure> {
        let mut empty = BRepGPropDomain::new();
        self.perform_adaptive(surface, &mut empty, VinertBound::Plane(plane), eps)
    }

    /// `Perform(S, D)`; `props.perform_domain(&mut face, &mut domain)?`.
    pub fn perform_domain(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
    ) -> Result<(), Failure> {
        self.perform_along(surface, domain, VinertBound::Location)
    }

    /// `Perform(S, D, Eps)`; `props.perform_domain_eps(&mut face, &mut domain, 1e-6)?`.
    pub fn perform_domain_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        eps: f64,
    ) -> Result<f64, Failure> {
        self.perform_adaptive(surface, domain, VinertBound::Location, eps)
    }

    /// `Perform(S, D, O)`; `props.perform_domain_origin(&mut face, &mut domain, o)?`.
    pub fn perform_domain_origin(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        origin: Point3,
    ) -> Result<(), Failure> {
        self.perform_along(surface, domain, VinertBound::Origin(origin))
    }

    /// `Perform(S, D, O, Eps)`; `props.perform_domain_origin_eps(&mut face, &mut domain, o, 1e-6)?`.
    pub fn perform_domain_origin_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        origin: Point3,
        eps: f64,
    ) -> Result<f64, Failure> {
        self.perform_adaptive(surface, domain, VinertBound::Origin(origin), eps)
    }

    /// `Perform(S, D, Pl)`; `props.perform_domain_plane(&mut face, &mut domain, plane)?`.
    pub fn perform_domain_plane(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
    ) -> Result<(), Failure> {
        self.perform_along(surface, domain, VinertBound::Plane(plane))
    }

    /// `Perform(S, D, Pl, Eps)`; `props.perform_domain_plane_eps(&mut face, &mut domain, plane, 1e-6)?`.
    pub fn perform_domain_plane_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        plane: Plane3,
        eps: f64,
    ) -> Result<f64, Failure> {
        self.perform_adaptive(surface, domain, VinertBound::Plane(plane), eps)
    }
}
