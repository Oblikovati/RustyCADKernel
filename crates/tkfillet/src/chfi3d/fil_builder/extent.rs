// Rust translation of ChFi3d_FilBuilder::ExtentOneCorner, ExtentTwoCorner,
// ExtentThreeCorner and SetRegul (ChFi3d_FilBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extensions of the contours at their vertices, and the continuities of the edges
//! between the fillets and the faces of the result.
use super::simul::max_rad;
use super::{ChFi3dFilBuilder, fil_spine};
use crate::chfi_ds::{SpineHandle, State, StripeHandle, TypeOfConcavity, lock, same_object};
use crate::chfi3d::builder::spine_of;
use crate::chfi3d::{edge_from_v1, evalconti, index_of_surf_data};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State as TopState;

/// The spine extended at its first end to `-dist`, or at its last end by `dist` beyond
/// its length, the tangent extension starting at the end.
fn extend_spine(spine: &SpineHandle, first: bool, dist: f64) -> Result<(), Failure> {
    let mut sp = lock(spine);
    if first {
        sp.set_first_parameter(-dist);
        return sp.set_first_tgt(0.);
    }
    let du = sp.last_parameter_of(sp.nb_edges())?;
    sp.set_last_parameter(du + dist);
    sp.set_last_tgt(du)
}

impl ChFi3dFilBuilder {
    /// `ExtentOneCorner(V, S)`: the contour ending alone at `v` extended by half its length
    /// (its whole length at a free boundary), unless it ends tangentially.
    pub(super) fn extent_one_corner_impl(
        &mut self,
        v: &Vertex,
        s: &StripeHandle,
    ) -> Result<(), Failure> {
        let spine = spine_of(s)?;
        let (_, sens) = index_of_surf_data(v, s)?;
        let first = sens == 1;
        let (du, tangent, status) = {
            let sp = lock(&spine);
            (
                sp.last_parameter_of(sp.nb_edges())?,
                sp.is_tangency_extremity(first),
                sp.status(first),
            )
        };
        if tangent {
            return Ok(());
        }
        let mut coeff = 0.5;
        if status == State::FreeBoundary {
            coeff *= 2.;
        }
        if first {
            return extend_spine(&spine, true, du * coeff);
        }
        let mut sp = lock(&spine);
        sp.set_last_parameter(du * (1. + coeff));
        sp.set_last_tgt(du)
    }

    /// `ExtentTwoCorner(V, LS)`: the two contours meeting at `v` extended by the same
    /// length, at least 0.3 of the longest and 1.5 the largest radius.
    pub(super) fn extent_two_corner_impl(
        &mut self,
        v: &Vertex,
        ls: &[StripeHandle],
    ) -> Result<(), Failure> {
        let coeff = 0.3;
        let mut eval: f64 = 0.;
        for st in ls {
            let spine = spine_of(st)?;
            let mut du = {
                let sp = lock(&spine);
                sp.last_parameter_of(sp.nb_edges())? * coeff
            };
            let constant = {
                let g = lock(&spine);
                let fsp = fil_spine(&g, "ExtentTwoCorner")?;
                if fsp.is_constant() {
                    Some(fsp.radius()?)
                } else {
                    None
                }
            };
            let mut rad = match constant {
                Some(r) => r,
                None => max_rad(&spine, &edge_from_v1(v, st)?.0)?,
            };
            rad *= 1.5;
            if rad > du {
                du = rad;
            }
            if du > eval {
                eval = du;
            }
        }
        // `Stripe` is the last contour of the first loop when the second one starts.
        let mut stripe = ls.last().cloned();
        let mut ff = true;
        for st in ls {
            let (_, mut sens) = index_of_surf_data(v, st)?;
            if !ff && stripe.as_ref().is_some_and(|s| same_object(s, st)) {
                sens = -sens;
            }
            stripe = Some(st.clone());
            let spine = spine_of(st)?;
            if !lock(&spine).is_tangency_extremity(sens == 1) {
                extend_spine(&spine, sens == 1, eval)?;
                ff = false;
            }
        }
        Ok(())
    }

    /// `ExtentThreeCorner(V, LS)`: the three contours meeting at `v` extended by a tenth
    /// of their length, but at a convex end on the same face; none when one ends
    /// tangentially (the contours before it are extended).
    pub(super) fn extent_three_corner_impl(
        &mut self,
        v: &Vertex,
        ls: &[StripeHandle],
    ) -> Result<(), Failure> {
        let coeff = 0.1;
        let mut check: Vec<StripeHandle> = Vec::new();
        for st in ls {
            let (_, mut sens) = index_of_surf_data(v, st)?;
            if check.iter().any(|c| same_object(c, st)) {
                sens = -sens;
            }
            let spine = spine_of(st)?;
            let first = sens == 1;
            let (tangent, du, concavity, status) = {
                let sp = lock(&spine);
                (
                    sp.is_tangency_extremity(first),
                    sp.last_parameter_of(sp.nb_edges())?,
                    sp.get_type_of_concavity(),
                    sp.status(first),
                )
            };
            if tangent {
                return Ok(());
            }
            if concavity != TypeOfConcavity::Convex || status != State::OnSame {
                let mut sp = lock(&spine);
                if first {
                    sp.set_first_parameter(-du * coeff);
                    sp.set_first_tgt(0.)?;
                } else {
                    sp.set_last_parameter(du * (1. + coeff));
                    sp.set_last_tgt(du)?;
                }
            }
            check.push(st.clone());
        }
        Ok(())
    }

    /// `SetRegul()`: the continuity of each regular edge between a fillet and a face (or
    /// two fillets) of the result, evaluated on the new faces.
    pub(super) fn set_regul_impl(&mut self) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let coup = self.base.my_coup.clone();
        let hds = self.base.my_ds.clone();
        for reg in &self.base.my_regul {
            let Some(e) = coup.new_edges(reg.curve()).iter().next().cloned() else {
                continue;
            };
            let e = topods::edge(&e)?;
            let support = |is_surface: bool,
                           s: i32|
             -> Result<Option<rustycad_tkbrep::topods::Shape>, Failure> {
                let list = if is_surface {
                    coup.new_faces(s)?
                } else {
                    coup.merged(&hds.shape(s, true), TopState::In)
                };
                Ok(list.iter().next().cloned())
            };
            let (Some(f1), Some(f2)) = (
                support(reg.is_surface1(), reg.s1())?,
                support(reg.is_surface2(), reg.s2())?,
            ) else {
                continue;
            };
            let (f1, f2) = (topods::face(&f1)?, topods::face(&f2)?);
            let cont = evalconti(&e, &f1, &f2)?;
            b.continuity_on_faces(&e, &f1, &f2, cont)?;
        }
        Ok(())
    }
}
