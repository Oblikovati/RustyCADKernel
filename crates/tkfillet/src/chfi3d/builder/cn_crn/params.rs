// Rust translation of the parameters of the ends of the connecting curves in
// ChFi3d_Builder::PerformMoreThreeCorner (ChFi3d_Builder_CnCrn.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Where the connecting curves start on the contours and the sharp edges: at the ends of
//! the contours, moved away from the vertex to the farthest end, and limited by the
//! common points of the neighbouring contours.
use super::regul::end_point;
use super::statics::{SurfType, calcul_p2d_on_surf, calcul_param, surf_index};
use super::{Corner, indices};
use crate::chfi_ds::CommonPoint;
use crate::chfi3d::builder::ChFi3dBuilder;
use crate::chfi3d::index_of_surf_data;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

impl ChFi3dBuilder {
    /// The parameters when some neighbours do not meet: the ends of the contours, then
    /// moved away from the vertex as far as the farthest end.
    pub(super) fn params_without_inters(&mut self, c: &mut Corner) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.oksea[ic] {
                continue;
            }
            let (icplus, icmoins) = indices(c.nedge, ic);
            let (icplus2, _) = indices(c.nedge, icplus);
            self.end_param(c, (ic, icplus), icmoins, false)?;
            self.end_param(c, (icplus, ic), icplus2, true)?;
        }
        let sommet = BRepTool::pnt(&c.v1)?;
        let (mut dist1, mut dist2) = (vec![0.0; c.nedge], vec![0.0; c.nedge]);
        let mut distance = 0.0;
        if !c.deuxconges {
            for ic in 0..c.nedge {
                if c.sharp[ic] {
                    continue;
                }
                (dist1[ic], dist2[ic]) = self.end_distances(c, ic, sommet)?;
                distance = f64::max(distance, dist1[ic]);
                distance = f64::max(distance, dist2[ic]);
            }
        }
        if !c.deuxconges && !c.deuxcgnontg {
            for ic in 0..c.nedge {
                if c.sharp[ic] {
                    offset_sharp(c, ic, distance)?;
                } else if !c.distmini {
                    offset_contour(c, ic, distance, (dist1[ic], dist2[ic]));
                }
            }
        }
        Ok(())
    }

    /// `p(a, b)` and `i(a, b)` of the end of the edge `a` toward `b`: the vertex on a
    /// sharp edge (unless already on a continuing edge), else the end of the contour or,
    /// when it meets its other neighbour `other`, the parameter of that meeting.
    fn end_param(
        &self,
        c: &mut Corner,
        (a, b): (usize, usize),
        other: usize,
        second: bool,
    ) -> Result<(), Failure> {
        if c.sharp[a] {
            if !c.samedge[a] {
                c.p[a][b] = BRepTool::parameter(&c.v1, &c.evive[a])?;
                c.i[a][b] = 1;
            }
            return Ok(());
        }
        c.i[a][b] = index_of_surf_data(&c.v1, c.cd(a)?)?.0;
        let met = if second { c.oksea[a] } else { c.oksea[other] };
        c.p[a][b] = if met {
            c.p[a][other]
        } else {
            let side = if second { 3 - c.jf[a] } else { c.jf[a] };
            calcul_param(c.cd(a)?, side, c.i[a][b], c.sens[a] == 1)?
        };
        Ok(())
    }

    /// The distances to the vertex of the ends of the contour `ic` toward its previous
    /// and its next neighbours.
    fn end_distances(&self, c: &Corner, ic: usize, sommet: Point3) -> Result<(f64, f64), Failure> {
        let (icplus, icmoins) = indices(c.nedge, ic);
        let mut d = [0.0; 2];
        for (k, (other, side)) in [(icmoins, 3 - c.jf[ic]), (icplus, c.jf[ic])]
            .into_iter()
            .enumerate()
        {
            let p2 = calcul_p2d_on_surf(c.cd(ic)?, side, c.i[ic][other], c.p[ic][other])?;
            let indice = surf_index(c, ic, c.i[ic][other], SurfType::Surface)?;
            d[k] = sommet.distance(&self.surface_value(indice, p2.x(), p2.y())?);
        }
        Ok((d[0], d[1]))
    }
}

/// The parameter on the sharp edge `ic` moved from the vertex by the 3d `distance`
/// (none on a regular edge tangent to another).
fn offset_sharp(c: &mut Corner, ic: usize, distance: f64) -> Result<(), Failure> {
    let (icplus, icmoins) = indices(c.nedge, ic);
    let curve = BRepAdaptorCurve::with_edge(&c.evive[ic])?;
    let ec = if !c.tangentregul[ic] {
        distance * 100. * curve.resolution(0.01)?
    } else {
        0.0
    };
    if topexp::first_vertex(&c.evive[ic], false).is_same(&c.v1) {
        c.p[ic][icmoins] += ec;
    } else {
        c.p[ic][icmoins] -= ec;
    }
    c.p[ic][icplus] = c.p[ic][icmoins];
    Ok(())
}

/// The ends of the contour `ic` moved along the contour to the distance of the farthest
/// end, a meeting much nearer to the vertex being dropped.
fn offset_contour(c: &mut Corner, ic: usize, distance: f64, (dist1, dist2): (f64, f64)) {
    let (icplus, icmoins) = indices(c.nedge, ic);
    if !c.oksea[icmoins] || distance > 1.3 * dist1 {
        let ec = distance - dist1;
        if c.oksea[icmoins] {
            c.oksea[icmoins] = false;
            c.inters = false;
        }
        if c.sens[ic] == 1 {
            c.p[ic][icmoins] += ec;
        } else {
            c.p[ic][icmoins] -= ec;
        }
    }
    if !c.oksea[ic] || distance > 1.3 * dist2 {
        if c.oksea[ic] {
            c.oksea[ic] = false;
            c.inters = false;
        }
        if c.nconges != 1 {
            let (parold, parnew) = (c.p[ic][icplus], c.p[ic][icmoins]);
            if (c.sens[ic] == 1 && parnew > parold) || (c.sens[ic] != 1 && parnew < parold) {
                c.p[ic][icplus] = c.p[ic][icmoins];
            }
        }
    }
}

impl ChFi3dBuilder {
    /// The sharp edges limited by the common points of their neighbours: stepped back to
    /// the farther one, or moved onto one very close (`ponctuel`).
    pub(super) fn limit_by_common_points(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if c.deuxconges {
            return Ok(());
        }
        let sommet = BRepTool::pnt(&c.v1)?;
        for ic in 0..c.nedge {
            if !c.sharp[ic] {
                continue;
            }
            let (icplus, icmoins) = indices(c.nedge, ic);
            let pe = BRepAdaptorCurve::with_edge(&c.evive[ic])?.value(c.p[ic][icplus])?;
            let (mut cp1, mut cp2) = (CommonPoint::new(), CommonPoint::new());
            let (mut d1, mut d2) = (0.0, 0.0);
            if !c.sharp[icplus] {
                cp1 = end_point(c, icplus, c.i[icplus][ic], 3 - c.jf[icplus])?;
                d1 = cp1.point().distance(&sommet);
            }
            if !c.sharp[icmoins] {
                cp2 = end_point(c, icmoins, c.i[icmoins][ic], c.jf[icmoins])?;
                d2 = cp2.point().distance(&sommet);
            }
            let samecompoint = !c.sharp[icmoins]
                && !c.sharp[icplus]
                && cp1.point().distance(&cp2.point()) < 1.0e-4;
            let ds = pe.distance(&sommet);
            if (ds < d1 || ds < d2) && !samecompoint {
                if d2 < d1 && cp1.is_on_arc() {
                    limit_on(c, ic, (&cp1, true), false)?;
                } else if cp2.is_on_arc() {
                    limit_on(c, ic, (&cp2, false), false)?;
                }
                continue;
            }
            let one = c.nconges == 1;
            let near =
                |cp: &CommonPoint| cp.point().distance(&pe) < cp.tolerance() || samecompoint || one;
            if !c.sharp[icplus] && near(&cp1) && cp1.is_on_arc() {
                limit_on(c, ic, (&cp1, true), true)?;
            }
            if !c.sharp[icmoins] && near(&cp2) && cp2.is_on_arc() {
                limit_on(c, ic, (&cp2, false), true)?;
            }
        }
        Ok(())
    }
}

/// The sharp edge `ic` limited at the common point `cp` of its next (else previous)
/// neighbour, whose end parameter is taken again, and the tolerance of the common points;
/// `ponctuel` set (on `ic` for the next neighbour, on the previous one otherwise) when
/// very close.
fn limit_on(
    c: &mut Corner,
    ic: usize,
    (cp, next): (&CommonPoint, bool),
    ponctuel: bool,
) -> Result<(), Failure> {
    let (icplus, icmoins) = indices(c.nedge, ic);
    let n = if next { icplus } else { icmoins };
    if ponctuel {
        c.ponctuel[if next { ic } else { icmoins }] = true;
    }
    c.p[ic][icmoins] = cp.parameter_on_arc()?;
    c.p[ic][icplus] = c.p[ic][icmoins];
    let side = if next { 3 - c.jf[n] } else { c.jf[n] };
    c.p[n][ic] = calcul_param(c.cd(n)?, side, c.i[n][ic], c.sens[n] == 1)?;
    if cp.tolerance() > c.tolcp && cp.tolerance() < 1.0 {
        c.tolcp = cp.tolerance();
    }
    Ok(())
}

impl ChFi3dBuilder {
    /// On a free border, the parameter of the common point of the neighbouring contour
    /// on it.
    pub(super) fn free_border_params(&mut self, c: &mut Corner) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if !c.is_libre_edge(ic) {
                continue;
            }
            let (icplus, icmoins) = indices(c.nedge, ic);
            let indic = if c.libre[ic] { icmoins } else { icplus };
            if c.sharp[indic] {
                continue;
            }
            for ons in [1, 2] {
                let cp = end_point(c, indic, c.index[indic], ons)?;
                if cp.is_on_arc() && cp.arc()?.is_same(&c.evive[ic]) {
                    c.p[ic][icmoins] = cp.parameter_on_arc()?;
                    c.p[ic][icplus] = c.p[ic][icmoins];
                    break;
                }
            }
        }
        Ok(())
    }
}
