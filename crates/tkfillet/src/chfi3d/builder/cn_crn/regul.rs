// Rust translation of the regular edges, of the cases of two fillets and of the search of
// the sharp edges limited by a common point in ChFi3d_Builder::PerformMoreThreeCorner
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sharp edges between tangent faces (regular), the two-fillet cases and the sharp
//! edges replaced by the edge where the neighbouring fillet ends.
use super::{Corner, indices};
use crate::chfi_ds::{CommonPoint, State, lock};
use crate::chfi3d::builder::{ChFi3dBuilder, spine_of};
use crate::chfi3d::{
    angle_edge, cherche_vertex, edge_common_faces, is_tangent_faces, surf_data_at,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use std::f64::consts::PI;

/// The angle is about 0 or pi.
fn flat(ang: f64) -> bool {
    ang < 0.01 || (ang - PI).abs() < 0.01
}

impl ChFi3dBuilder {
    /// `regul`: the sharp edges between tangent faces, unless tangent to another edge
    /// (cts60072); then the two-fillet cases.
    pub(super) fn update_regul(&mut self, c: &mut Corner) -> Result<(), Failure> {
        for ic in 0..c.nedge {
            if c.sharp[ic] && !c.evive[ic].is_same(&c.edgecouture) {
                let ecur = c.evive[ic].clone();
                let (f1, f2) = edge_common_faces(self.my_ef_map.find_from_key(ecur.as_ref())?)?;
                c.regul[ic] = is_tangent_faces(&ecur, &f1, &f2, Continuity::G1)?;
            }
        }
        for ic in 0..c.nedge {
            if c.regul[ic] && tangent_to_another(c, ic)? {
                c.regul[ic] = false;
                c.tangentregul[ic] = true;
            }
        }
        self.two_fillets(c)
    }

    /// `deuxconges` (two fillets on tangent edges, or two tangent sharp edges among four),
    /// `deuxcgnontg` and the OnSame + OnDiff case of three edges (occ266), whose corner is
    /// first prepared by `PerformOneCorner`.
    fn two_fillets(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if c.nconges == 2 {
            c.deuxconges = two_tangent_fillets(c)?;
        }
        if c.nconges == 2 && c.nedge == 4 && !c.deuxconges {
            c.deuxconges = two_tangent_sharp(c)?;
        }
        c.deuxcgnontg = c.nconges == 2 && c.nedge == 3 && !c.deuxconges;
        if c.deuxconges {
            c.regul[..c.nedge].fill(false);
        }
        if c.deuxcgnontg {
            c.on_same_diff = on_same_and_on_diff(c)?;
        }
        if c.on_same_diff {
            self.perform_one_corner(c.jndex, true)?;
        }
        Ok(())
    }
}

/// The regular edge `ic` is tangent to another edge at the vertex.
fn tangent_to_another(c: &Corner, ic: usize) -> Result<bool, Failure> {
    for ind in 0..c.nedge {
        if ind == ic {
            continue;
        }
        if flat(angle_edge(&c.v1, &c.evive[ind], &c.evive[ic])?.abs()) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Two consecutive contours on tangent edges.
fn two_tangent_fillets(c: &Corner) -> Result<bool, Failure> {
    for ic in 0..c.nedge {
        let (icplus, _) = indices(c.nedge, ic);
        if !c.sharp[ic]
            && !c.sharp[icplus]
            && angle_edge(&c.v1, &c.evive[ic], &c.evive[icplus])?.abs() < 0.01
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Two consecutive sharp edges, not free borders, are tangent.
fn two_tangent_sharp(c: &Corner) -> Result<bool, Failure> {
    let mut deuxconges = false;
    for ic in 0..c.nedge {
        if deuxconges {
            break;
        }
        let (icplus, _) = indices(c.nedge, ic);
        if c.sharp[ic] && c.sharp[icplus] && !c.is_libre_edge(ic) && !c.is_libre_edge(icplus) {
            deuxconges = flat(angle_edge(&c.v1, &c.evive[ic], &c.evive[icplus])?.abs());
        }
    }
    Ok(deuxconges)
}

/// One contour ends OnSame and another OnDiff.
fn on_same_and_on_diff(c: &Corner) -> Result<bool, Failure> {
    let (mut on_same, mut on_diff) = (false, false);
    for ic in 0..c.nedge {
        if c.sharp[ic] {
            continue;
        }
        let sp = spine_of(c.cd(ic)?)?;
        let stat = if c.sens[ic] == 1 {
            lock(&sp).first_status()
        } else {
            lock(&sp).last_status()
        };
        on_same |= stat == State::OnSame;
        on_diff |= stat == State::OnDiff;
    }
    Ok(on_same && on_diff)
}

/// The common point of the side `ons` of the end at the vertex of the contour `ic`.
pub(super) fn end_point(c: &Corner, ic: usize, sd: i32, ons: i32) -> Result<CommonPoint, Failure> {
    let fd = surf_data_at(c.cd(ic)?, sd)?;
    let cp = lock(&fd).vertex(c.sens[ic] == 1, ons).clone();
    Ok(cp)
}

impl ChFi3dBuilder {
    /// When the common point of a neighbouring contour is on an edge continuing the
    /// sharp edge `ic` (cts16288), `Evive` becomes that edge and the faces around it are
    /// found again.
    pub(super) fn find_evive_anew(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if c.deuxconges || c.on_same_diff {
            return Ok(());
        }
        for ic in 0..c.nedge {
            if !c.sharp[ic] {
                continue;
            }
            let (icplus, icmoins) = indices(c.nedge, ic);
            let arc = c.evive[ic].clone();
            if !c.sharp[icplus] {
                let cp1 = end_point(c, icplus, c.index[icplus], 3 - c.jf[icplus])?;
                self.continue_on(c, (ic, icplus, icmoins), &arc, &cp1)?;
            }
            if !c.sharp[icmoins] {
                let cp2 = end_point(c, icmoins, c.index[icmoins], c.jf[icmoins])?;
                self.continue_on(c, (ic, icmoins, icplus), &arc, &cp2)?;
            }
        }
        Ok(())
    }

    /// The sharp edge `ic` continued by the arc of `cp` (at pi from `arc`): the faces
    /// toward `side` found again when neither is a face of the arc, and the parameters.
    fn continue_on(
        &mut self,
        c: &mut Corner,
        (ic, side, other): (usize, usize, usize),
        arc: &Edge,
        cp: &CommonPoint,
    ) -> Result<(), Failure> {
        if !cp.is_on_arc() {
            return Ok(());
        }
        let cparc = cp.arc()?.clone();
        let mut angedg = PI;
        if let Some(vcom) = cherche_vertex(arc, &cparc)? {
            angedg = angle_edge(&vcom, arc, &cparc)?.abs();
        }
        if cparc.is_same(arc) || (angedg - PI).abs() >= 0.01 {
            return Ok(());
        }
        c.evive[ic] = cparc.clone();
        let (f1, f2) = edge_common_faces(self.my_ef_map.find_from_key(cparc.as_ref())?)?;
        if !c.fvive[ic][side].is_same(&f1) && !c.fvive[ic][side].is_same(&f2) {
            let f = if c.fvive[ic][other].is_same(&f2) {
                f1
            } else {
                f2
            };
            c.fvive[ic][side] = f.clone();
            c.fvive[side][ic] = f.clone();
            c.numfa[ic][side] = self.ds.add_shape(f.as_ref());
            c.numfa[side][ic] = self.ds.add_shape(f.as_ref());
        }
        c.samedge[ic] = true;
        let par = cp.parameter_on_arc()?;
        c.p[ic][side] = par;
        c.p[ic][other] = par;
        c.i[ic][side] = 1;
        Ok(())
    }

    /// The first free border, and the smallest distance the corner can step back
    /// (`distmin`, two thirds of the shortest edge at the vertex).
    pub(super) fn limits(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if let Some(ic) = (0..c.nedge).find(|&ic| c.is_libre_edge(ic)) {
            c.libre[ic] = true;
        }
        for ic in 0..c.nedge {
            let edgemin = if c.sharp[ic] {
                c.evive[ic].clone()
            } else {
                let sp = spine_of(c.cd(ic)?)?;
                let g = lock(&sp);
                g.edges(if c.sens[ic] == 1 { 1 } else { g.nb_edges() })?
                    .clone()
            };
            let v = topexp::first_vertex(&edgemin, false);
            let v2 = topexp::last_vertex(&edgemin, false);
            let dst = BRepTool::pnt(&v)?.distance(&BRepTool::pnt(&v2)?) / 1.5;
            if dst < c.distmin {
                c.distmin = dst;
            }
        }
        Ok(())
    }
}
