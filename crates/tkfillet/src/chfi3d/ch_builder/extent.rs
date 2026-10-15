// Rust translation of ChFi3d_ChBuilder::ExtentOneCorner, ExtentTwoCorner,
// ExtentThreeCorner, SetRegul and the file static ExtentSpineOnCommonFace
// (ChFi3d_ChBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extensions of the chamfer contours at their vertices, and the continuities of the
//! edges between tangent chamfers.
use super::ChFi3dChBuilder;
use super::functions::chamfer_kind;
use crate::chfi_ds::{ChamfMethod, SpineHandle, State, StripeHandle, lock, same_object};
use crate::chfi3d::builder::spine_of;
use crate::chfi3d::{evalconti, index_of_surf_data};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// The spine extended at its first end to `-dist`, or at its last end by `dist` beyond
/// its length.
fn extend_spine(spine: &SpineHandle, first: bool, dist: f64) -> Result<(), Failure> {
    let mut sp = lock(spine);
    if first {
        sp.set_first_parameter(-dist);
        return sp.set_first_tgt(0.);
    }
    let param = sp.last_parameter_of(sp.nb_edges())?;
    sp.set_last_parameter(dist + param);
    sp.set_last_tgt(param)
}

/// The unit tangent of the spine at the vertex, pointing away from it at a first end.
fn tangent_at(
    spine: &SpineHandle,
    v: &Vertex,
    first: bool,
) -> Result<rustycad_tkmath::gp::Vector3, Failure> {
    let mut g = lock(spine);
    let a = g.absc_of_vertex(v)?;
    let (_, mut tg) = g.d1(a)?;
    tg.normalize()?;
    if first {
        tg.reverse();
    }
    Ok(tg)
}

/// `ExtentSpineOnCommonFace(Spine1, Spine2, V, dis1, dis2, isfirst1, isfirst2)`: the two
/// spines extended at the vertex by three (and one and a half) times the length the
/// chamfers need on their common face, from the opening angle between them.
fn extent_spine_on_common_face(
    (spine1, spine2): (&SpineHandle, &SpineHandle),
    v: &Vertex,
    (dis1, dis2): (f64, f64),
    (isfirst1, isfirst2): (bool, bool),
) -> Result<(), Failure> {
    let tolesp = 1.0e-7;
    let tg1 = tangent_at(spine1, v, isfirst1)?;
    let tg2 = tangent_at(spine2, v, isfirst2)?;
    let cosalpha = tg1.dot(tg2);
    let sinalpha = (1. - cosalpha * cosalpha).sqrt();
    let (mut d1plus, mut d2plus) = (0., 0.);
    let temp = cosalpha + dis2 / dis1;
    if temp.abs() > tolesp {
        d1plus = dis1 / (sinalpha / temp);
    }
    let temp = cosalpha + dis1 / dis2;
    if temp.abs() > tolesp {
        d2plus = dis2 / (sinalpha / temp);
    }
    if d1plus > 0. {
        extend_spine(spine1, isfirst1, d1plus * 3.)?;
    }
    if d2plus > 0. {
        extend_spine(spine2, isfirst2, d2plus * 1.5)?;
    }
    Ok(())
}

/// The two distances of a chamfer on its faces (the second one approximate for a
/// distance and an angle).
fn chamfer_distances(spine: &SpineHandle) -> Result<[f64; 2], Failure> {
    let kind = chamfer_kind(spine, "Extent : this is not the spine of a chamfer")?;
    Ok(match kind.method {
        ChamfMethod::Sym => [kind.d1, kind.d1],
        ChamfMethod::TwoDist => [kind.d1, kind.d2],
        ChamfMethod::DistAngle => [kind.d1, kind.d1 * kind.angle.tan()],
    })
}

/// The ends of the contours at the vertex: their spines, whether at their first end,
/// the index of their edge there (a contour met twice has its other end the second
/// time).
type Ends = (Vec<SpineHandle>, Vec<bool>, Vec<i32>);

fn ends_at(v: &Vertex, ls: &[StripeHandle]) -> Result<Ends, Failure> {
    let (mut spines, mut firsts, mut iedges) = (Vec::new(), Vec::new(), Vec::new());
    let mut check: Vec<StripeHandle> = Vec::new();
    for st in ls {
        let (_, mut sens) = index_of_surf_data(v, st)?;
        if check.iter().any(|c| same_object(c, st)) {
            sens = -sens;
        }
        let spine = spine_of(st)?;
        let isfirst = sens == 1;
        iedges.push(if isfirst { 1 } else { lock(&spine).nb_edges() });
        firsts.push(isfirst);
        spines.push(spine);
        check.push(st.clone());
    }
    Ok((spines, firsts, iedges))
}

impl ChFi3dChBuilder {
    /// `ExtentOneCorner(V, S)`: the contour ending alone at `v` extended by half its
    /// length, unless it ends tangentially.
    pub(super) fn extent_one_corner_impl(
        &mut self,
        v: &Vertex,
        s: &StripeHandle,
    ) -> Result<(), Failure> {
        let spine = spine_of(s)?;
        let (_, sens) = index_of_surf_data(v, s)?;
        if lock(&spine).is_tangency_extremity(sens == 1) {
            return Ok(());
        }
        let du = {
            let g = lock(&spine);
            g.last_parameter_of(g.nb_edges())?
        };
        let mut g = lock(&spine);
        if sens == 1 {
            g.set_first_parameter(-du * 0.5);
            return g.set_first_tgt(0.);
        }
        g.set_last_parameter(du * 1.5);
        g.set_last_tgt(du)
    }

    /// `ExtentTwoCorner(V, LS)`: both contours extended to the face at the end (all on
    /// the same faces), or along their common face (both on same).
    pub(super) fn extent_two_corner_impl(
        &mut self,
        v: &Vertex,
        ls: &[StripeHandle],
    ) -> Result<(), Failure> {
        let (spines, isfirst, iedge) = self.two_ends(v, ls)?;
        let mut f: [Face; 4] = Default::default();
        let mut d = [0.; 4];
        for i in 0..2 {
            let (a, b) = self.conex_faces_of(&spines[i], iedge[i])?;
            f[2 * i] = a;
            f[2 * i + 1] = b;
            let [d1, d2] = chamfer_distances(&spines[i])?;
            d[2 * i] = d1;
            d[2 * i + 1] = d2;
        }
        let mut dis = [0., 0.];
        'found: for i in 0..2 {
            for j in 0..2 {
                if f[i].is_same(&f[j + 2]) {
                    dis = [d[i], d[j + 2]];
                    break 'found;
                }
            }
        }
        let state: Vec<State> = (0..2)
            .map(|i| lock(&spines[i]).status(isfirst[i]))
            .collect();
        if state[0] == State::AllSame {
            // Both chamfers must reach the face at the end.
            for st in ls.iter().take(2) {
                self.extent_one_corner_impl(v, st)?;
            }
        } else if state[0] == State::OnSame && state[1] == State::OnSame {
            extent_spine_on_common_face(
                (&spines[0], &spines[1]),
                v,
                (dis[0], dis[1]),
                (isfirst[0], isfirst[1]),
            )?;
        }
        Ok(())
    }

    /// The ends of the two contours at the vertex (the second end of a contour met
    /// twice, as the native `Stripe[1] == itel.Value()` test).
    fn two_ends(&self, v: &Vertex, ls: &[StripeHandle]) -> Result<Ends, Failure> {
        let (mut spines, mut firsts, mut iedges) = (Vec::new(), Vec::new(), Vec::new());
        let mut stripe: [Option<StripeHandle>; 2] = [None, None];
        for (i, st) in ls.iter().enumerate().take(2) {
            let (_, mut sens) = index_of_surf_data(v, st)?;
            if i > 0 && stripe[1].as_ref().is_some_and(|s| same_object(s, st)) {
                sens = -sens;
            }
            stripe[i] = Some(st.clone());
            let spine = spine_of(st)?;
            let isfirst = sens == 1;
            iedges.push(if isfirst { 1 } else { lock(&spine).nb_edges() });
            firsts.push(isfirst);
            spines.push(spine);
        }
        Ok((spines, firsts, iedges))
    }

    /// `ExtentThreeCorner(V, LS)`: each pair of the three contours extended along their
    /// common face.
    pub(super) fn extent_three_corner_impl(
        &mut self,
        v: &Vertex,
        ls: &[StripeHandle],
    ) -> Result<(), Failure> {
        let (spines, isfirst, iedge) = ends_at(v, ls)?;
        let mut f: Vec<[Face; 2]> = Vec::new();
        let mut d: Vec<[f64; 2]> = Vec::new();
        for i in 0..3 {
            let (a, b) = self.conex_faces_of(&spines[i], iedge[i])?;
            f.push([a, b]);
            d.push(chamfer_distances(&spines[i])?);
        }
        let mut dis = [[0.; 3]; 3];
        for i in 0..3 {
            let j = (i + 1) % 3;
            'found: for k in 0..2 {
                for l in 0..2 {
                    if f[i][k].is_same(&f[j][l]) {
                        dis[i][j] = d[i][k];
                        dis[j][i] = d[j][l];
                        break 'found;
                    }
                }
            }
        }
        for i in 0..3 {
            let j = (i + 1) % 3;
            extent_spine_on_common_face(
                (&spines[i], &spines[j]),
                v,
                (dis[i][j], dis[j][i]),
                (isfirst[i], isfirst[j]),
            )?;
        }
        Ok(())
    }

    /// `SetRegul()`: the continuity of the edges between two chamfers whose normals are
    /// parallel within half a degree.
    pub(super) fn set_regul_impl(&mut self) -> Result<(), Failure> {
        let seuil = std::f64::consts::PI / 360.;
        let seuil2 = seuil * seuil;
        let b = BRepBuilder::default();
        let coup = self.base.my_coup.clone();
        for reg in &self.base.my_regul {
            let Some(e) = coup.new_edges(reg.curve()).iter().next().cloned() else {
                continue;
            };
            if !(reg.is_surface1() && reg.is_surface2()) {
                continue;
            }
            let e = topods::edge(&e)?;
            let (Some(f1), Some(f2)) = (
                coup.new_faces(reg.s1())?.iter().next().cloned(),
                coup.new_faces(reg.s2())?.iter().next().cloned(),
            ) else {
                continue;
            };
            let (f1, f2) = (topods::face(&f1)?, topods::face(&f2)?);
            let pc1 = BRepAdaptorCurve2d::with_edge_on_face(&e, &f1)?;
            let t = 0.5 * (pc1.first_parameter()? + pc1.last_parameter()?);
            let normal = |f: &Face,
                          pc: &BRepAdaptorCurve2d|
             -> Result<rustycad_tkmath::gp::Vector3, Failure> {
                let s = BRepAdaptorSurface::with_face(f, false)?;
                let uv = pc.value(t)?;
                let d1 = s.d1(uv.x(), uv.y())?;
                Ok(d1.d1u.crossed(d1.d1v))
            };
            let mut n1 = normal(&f1, &pc1)?;
            let mut n2 = normal(&f2, &BRepAdaptorCurve2d::with_edge_on_face(&e, &f2)?)?;
            if n1.square_magnitude() > 1.0e-14 && n2.square_magnitude() > 1.0e-14 {
                n1.normalize()?;
                n2.normalize()?;
                if n1.crossed(n2).square_magnitude() < seuil2 {
                    let cont = evalconti(&e, &f1, &f2)?;
                    b.continuity_on_faces(&e, &f1, &f2, cont)?;
                }
            }
        }
        Ok(())
    }
}
