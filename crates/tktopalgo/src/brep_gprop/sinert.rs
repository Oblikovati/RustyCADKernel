// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Sinert.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Sinert`: the global properties of a face.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::gauss::{BRepGPropGauss, GaussProps, GaussType};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use std::ops::Deref;

/// `BRepGProp_Sinert`: the area, centre and inertia of a face (a `GProp_GProps`,
/// docs/topalgo-design.md section 9); the inherited members are reached through
/// `Deref`.
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropSinert;
/// let mut props = BRepGPropSinert::new();
/// assert_eq!(props.get_epsilon(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepGPropSinert {
    base: GProps,
    epsilon: f64,
}

impl Deref for BRepGPropSinert {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

/// The outputs of a `Compute` read from, and written back to, the protected members.
pub(super) fn gauss_props(base: &GProps) -> GaussProps {
    GaussProps {
        mass: base.dim,
        gravity_center: base.g,
        inertia: base.inertia,
    }
}

/// Writes the outputs of a `Compute` into the protected members.
pub(super) fn store_gauss_props(base: &mut GProps, out: &GaussProps) {
    base.dim = out.mass;
    base.g = out.gravity_center;
    base.inertia = out.inertia;
}

impl BRepGPropSinert {
    /// `BRepGProp_Sinert()`: an empty framework at the origin, epsilon 0;
    /// `BRepGPropSinert::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepGProp_Sinert(S, SLocation)`: the face on its natural bounds;
    /// `BRepGPropSinert::with_face(&face, location)?`.
    pub fn with_face(surface: &BRepGPropFace, location: Point3) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform(surface)?;
        Ok(props)
    }

    /// `BRepGProp_Sinert(S, D, SLocation)`: the face bounded by the domain;
    /// `BRepGPropSinert::with_domain(&mut face, &mut domain, location)?`.
    pub fn with_domain(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_domain(surface, domain)?;
        Ok(props)
    }

    /// `BRepGProp_Sinert(S, SLocation, Eps)`: the adaptive integration without domain;
    /// `BRepGPropSinert::with_face_eps(&mut face, location, 1.0e-6)?`.
    pub fn with_face_eps(
        surface: &mut BRepGPropFace,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_eps(surface, eps)?;
        Ok(props)
    }

    /// `BRepGProp_Sinert(S, D, SLocation, Eps)`: the adaptive integration on the domain;
    /// `BRepGPropSinert::with_domain_eps(&mut face, &mut domain, location, 1.0e-6)?`.
    pub fn with_domain_eps(
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        eps: f64,
    ) -> Result<Self, Failure> {
        let mut props = Self::located(location);
        props.perform_domain_eps(surface, domain, eps)?;
        Ok(props)
    }

    fn located(location: Point3) -> Self {
        let mut props = Self::new();
        props.set_location(location);
        props
    }

    /// `SetLocation(SLocation)`; `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(S)`: fixed Gauss orders on the natural bounds; epsilon 1;
    /// `props.perform(&face)?`.
    pub fn perform(&mut self, surface: &BRepGPropFace) -> Result<(), Failure> {
        self.epsilon = 1.0;
        let mut out = gauss_props(&self.base);
        BRepGPropGauss::new(GaussType::Sinert).compute_natural_surface(
            surface,
            self.base.loc,
            &mut out,
        )?;
        store_gauss_props(&mut self.base, &out);
        Ok(())
    }

    /// `Perform(S, D)`: fixed Gauss orders along the edges of the domain; epsilon 1;
    /// `props.perform_domain(&mut face, &mut domain)?`.
    pub fn perform_domain(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
    ) -> Result<(), Failure> {
        self.epsilon = 1.0;
        let mut out = gauss_props(&self.base);
        BRepGPropGauss::new(GaussType::Sinert).compute_domain_surface(
            surface,
            domain,
            self.base.loc,
            &mut out,
        )?;
        store_gauss_props(&mut self.base, &out);
        Ok(())
    }

    /// `Perform(S, Eps)`: the adaptive integration with an empty domain (a face with
    /// wires gives nothing, as natively); the reached error;
    /// `props.perform_eps(&mut face, 1.0e-6)?`.
    pub fn perform_eps(&mut self, surface: &mut BRepGPropFace, eps: f64) -> Result<f64, Failure> {
        self.perform_domain_eps(surface, &mut BRepGPropDomain::new(), eps)
    }

    /// `Perform(S, D, Eps)`: the adaptive integration; the reached error, kept as the
    /// epsilon; `props.perform_domain_eps(&mut face, &mut domain, 1.0e-6)?`.
    pub fn perform_domain_eps(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        eps: f64,
    ) -> Result<f64, Failure> {
        let mut out = gauss_props(&self.base);
        let mut gauss = BRepGPropGauss::new(GaussType::Sinert);
        let error =
            gauss.compute_adaptive_surface(surface, domain, self.base.loc, eps, &mut out)?;
        store_gauss_props(&mut self.base, &out);
        self.epsilon = error;
        Ok(error)
    }

    /// `GetEpsilon()`: the error of the last adaptive Perform, 1 after a fixed one;
    /// `props.get_epsilon()`.
    pub fn get_epsilon(&mut self) -> f64 {
        self.epsilon
    }
}
