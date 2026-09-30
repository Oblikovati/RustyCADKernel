// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_AppSurf.gxx (the section
// loops of both Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The multi-point of one section: the poles multiplied by their weights (the
//! rational curves become polynomial in homogeneous coordinates), the 2d poles, then
//! one 2d point `(w, 0)` per weight, with their derivatives when the generator gives
//! them.
use super::{AppBlendSection, TheSectionGenerator, get_context_approx_with_no_tgt};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkgeombase::app_def::MultiPointConstraint;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3, Xyz};

/// The affine map of the 3d poles and of each 2d curve into the unit box
/// (`x * DX + X`), identity for the plain `Perform`.
pub(super) struct BoxScale {
    /// `(X, DX)`, `(Y, DY)`, `(Z, DZ)`.
    pub(super) axes: [(f64, f64); 3],
    /// Per 2d curve: `(X2d, DX2d)` and `(Y2d, DY2d)`.
    pub(super) curves2d: Vec<[(f64, f64); 2]>,
}

/// The working arrays of a perform (the native `tab*` locals).
pub(super) struct SectionArrays {
    pub(super) section: AppBlendSection,
    pub(super) app_p2d: Array1<'static, Point2>,
    pub(super) app_v2d: Array1<'static, Vector2>,
    pub(super) nb_u_poles: i32,
    pub(super) nb_poles2d: i32,
}

impl SectionArrays {
    /// The arrays of `NbUPoles` poles and `NbPoles2d` 2d poles (at least one 2d
    /// slot, as natively).
    pub(super) fn new(nb_u_poles: i32, nb_poles2d: i32) -> Result<Self, Failure> {
        let n2d: i32 = nb_poles2d.max(1);
        Ok(Self {
            section: AppBlendSection {
                poles: one_based(nb_u_poles)?,
                d_poles: one_based(nb_u_poles)?,
                poles2d: one_based(n2d)?,
                d_poles2d: one_based(n2d)?,
                weights: one_based(nb_u_poles)?,
                d_weights: one_based(nb_u_poles)?,
            },
            app_p2d: one_based(nb_poles2d + nb_u_poles)?,
            app_v2d: one_based(nb_poles2d + nb_u_poles)?,
            nb_u_poles,
            nb_poles2d,
        })
    }
    /// The section with derivatives at `p`; the derivatives are dropped when
    /// `AppBlend_GetContextApproxWithNoTgt()`.
    pub(super) fn load_d1<G: TheSectionGenerator>(
        &mut self,
        f: &mut G,
        p: &G::Point,
    ) -> Result<bool, Failure> {
        let withderiv: bool = f.section_d1(p, &mut self.section)?;
        Ok(withderiv && !get_context_approx_with_no_tgt())
    }
    /// Copies the 2d poles and weights the poles into the multi-point arrays; the
    /// tangents follow `(W P)' = W' P + W P'` when `withderiv`. With a `scale`, the
    /// 3d and 2d values are mapped into the unit box (`Perform(Lin, F, NbMaxP)`);
    /// without, they are copied unchanged (no `* 1 + 0`, which would turn -0 into 0).
    pub(super) fn weighted(&mut self, withderiv: bool, scale: Option<&BoxScale>) {
        for j in 1..=self.nb_poles2d {
            let (p, v) = (self.section.poles2d[j], self.section.d_poles2d[j]);
            let Some(scale) = scale else {
                self.app_p2d[j] = p;
                if withderiv {
                    self.app_v2d[j] = v;
                }
                continue;
            };
            let [(x0, dx), (y0, dy)] = scale.curves2d[(j - 1) as usize];
            self.app_p2d[j] = Point2::new(dx * p.x() + x0, dy * p.y() + y0);
            if withderiv {
                self.app_v2d[j] = Vector2::new(dx * v.x(), dy * v.y());
            }
        }
        for j in 1..=self.nb_u_poles {
            self.weight_pole(j, withderiv, scale);
        }
    }
    /// Pole `j` times its weight (and its tangent) into homogeneous coordinates.
    fn weight_pole(&mut self, j: i32, withderiv: bool, scale: Option<&BoxScale>) {
        let s = &mut self.section;
        let (w, dw) = (s.weights[j], s.d_weights[j]);
        if withderiv {
            self.app_v2d[self.nb_poles2d + j] = Vector2::new(dw, 0.0);
            let mut new_dv = Xyz::default();
            new_dv.set_linear_form(dw, s.poles[j].xyz(), w, *s.d_poles[j].xyz());
            s.d_poles[j] = match scale {
                None => Vector3::from(new_dv),
                Some(b) => {
                    let [(_, dx), (_, dy), (_, dz)] = b.axes;
                    Vector3::new(dx * new_dv.x(), dy * new_dv.y(), dz * new_dv.z())
                }
            };
        }
        let p: Xyz = s.poles[j].xyz() * w;
        self.app_p2d[self.nb_poles2d + j] = Point2::new(w, 0.0);
        s.poles[j] = match scale {
            None => Point3::from(p),
            Some(b) => {
                let [(x0, dx), (y0, dy), (z0, dz)] = b.axes;
                Point3::new(dx * p.x() + x0, dy * p.y() + y0, dz * p.z() + z0)
            }
        };
    }
    /// The multi-point constraint of the current arrays, with tangents when
    /// `withderiv`.
    pub(super) fn constraint(&self, withderiv: bool) -> Result<MultiPointConstraint, Failure> {
        let s = &self.section;
        if withderiv {
            MultiPointConstraint::with_tangents(&s.poles, &self.app_p2d, &s.d_poles, &self.app_v2d)
        } else {
            Ok(MultiPointConstraint::from_points_and_2d(
                &s.poles,
                &self.app_p2d,
            ))
        }
    }
}
