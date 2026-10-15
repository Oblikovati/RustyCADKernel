// SPDX-License-Identifier: GPL-2.0-only
//! Free-form surfaces: the same Green's theorem, with the `u` antiderivative itself computed
//! by adaptive Gauss-Kronrod from the surface derivatives (nested quadrature). The inner
//! error estimates are integrated along the trim and added to the outer estimate.
use super::accumulate::{AREA, AREA_MOMENT, FLUX, FLUX_MOMENT, QUANTITIES};
use super::planar::cross;
use super::quadrature::{Quadrature, integrate};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// A located free-form surface about `reference`, integrated in `u` from `u_start`.
pub(crate) struct GeneralSurface {
    surface: BRepAdaptorSurface,
    reference: [f64; 3],
    /// A `u` inside the face's parameter range, so every inner path stays on the surface.
    u_start: f64,
}

impl GeneralSurface {
    /// The integrand of `surface`; `GeneralSurface::new(adaptor, q, trim_start_u)`.
    pub(crate) fn new(surface: BRepAdaptorSurface, reference: [f64; 3], u_start: f64) -> Self {
        Self {
            surface,
            reference,
            u_start,
        }
    }

    /// `F(u, v) = integral_{u_start}^{u} f(s, v) ds` per quantity, with its error estimate.
    pub(crate) fn antiderivative(&self, u: f64, v: f64) -> Result<Quadrature<QUANTITIES>, Failure> {
        integrate(&|s| self.density(s, v), self.u_start, u)
    }

    /// `rho, rho (P - Q), (P - Q).N, ((P - Q).N)(P - Q)` with `N = Pu x Pv`, `rho = |N|`.
    fn density(&self, u: f64, v: f64) -> Result<[f64; QUANTITIES], Failure> {
        let derivatives = self.surface.d1(u, v)?;
        let point = derivatives.point.coordinates();
        let offset: [f64; 3] = std::array::from_fn(|k| point[k] - self.reference[k]);
        let (du, dv) = (derivatives.d1u, derivatives.d1v);
        let normal = cross([du.x(), du.y(), du.z()], [dv.x(), dv.y(), dv.z()]);
        let density = normal.iter().map(|c| c * c).sum::<f64>().sqrt();
        let flux: f64 = (0..3).map(|k| offset[k] * normal[k]).sum();
        Ok(std::array::from_fn(|index| match index {
            AREA => density,
            FLUX => flux,
            i if i < FLUX => density * offset[i - AREA_MOMENT],
            i => flux * offset[i - FLUX_MOMENT],
        }))
    }
}
