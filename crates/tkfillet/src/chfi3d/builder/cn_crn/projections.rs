// Rust translation of the connecting curves projected on several faces in
// ChFi3d_Builder::PerformMoreThreeCorner, and of the file static cherche_edge1
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The connecting curves that cross several faces: across regular sharp edges (from the
//! last edge before them to the first one after), and between two contours on different
//! faces; both are Hermite curves projected on the faces crossed.
use super::hermite::{HermiteEnd, Projections, curve_hermite};
use super::same_ext::perform_two_corner_same_ext;
use super::statics::{SurfType, calcul_p2d_on_surf, surf_index};
use super::{Corner, indices};
use crate::chfi3d::builder::ChFi3dBuilder;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// `cherche_edge1(F1, F2, Edge)`: `edge` set to the first edge common to the two faces;
/// it keeps its value when there is none, the failure coming only when it is null.
fn cherche_edge1(f1: &Face, f2: &Face, edge: &mut Edge) -> Result<(), Failure> {
    let (mut m1, mut m2) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes_of_type(f1.as_ref(), ShapeEnum::Edge, &mut m1)?;
    topexp::map_shapes_of_type(f2.as_ref(), ShapeEnum::Edge, &mut m2)?;
    'search: for i in 1..=m1.extent() {
        for j in 1..=m2.extent() {
            if m2.find_key(j)?.is_same(m1.find_key(i)?) {
                *edge = topods::edge(m1.find_key(i)?)?;
                break 'search;
            }
        }
    }
    if edge.is_null() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "Failed to find edge",
        ));
    }
    Ok(())
}

/// The `k`-th (from 1) element of a sequence, Standard_OutOfRange outside it.
fn nth<T: Clone>(v: &[T], k: usize) -> Result<T, Failure> {
    v.get(k.wrapping_sub(1))
        .cloned()
        .ok_or_else(|| Failure::out_of_range("PerformMoreThreeCorner: sequence"))
}

impl ChFi3dBuilder {
    /// The face of the side `side` of the end of the contour `ic` toward `other`.
    fn contour_face(
        &self,
        c: &Corner,
        (ic, other): (usize, usize),
        side: i32,
    ) -> Result<Face, Failure> {
        let kind = if side == 1 {
            SurfType::Face1
        } else {
            SurfType::Face2
        };
        let ilin = surf_index(c, ic, c.i[ic][other], kind)?;
        topods::face(self.ds.shape(ilin, true))
    }

    /// The end of a Hermite curve at the edge `a`, toward `b`.
    fn hermite_end<'a>(c: &'a Corner, a: usize, b: usize) -> HermiteEnd<'a> {
        HermiteEnd {
            cd: c.cd[a].as_ref(),
            jf: c.jf[a],
            sd: c.i[a][b],
            par: c.p[a][b],
            sens: c.sens[a],
            sharp: c.sharp[a],
            evive: &c.evive[a],
        }
    }

    /// Across regular edges: the Hermite curve from the edge before them to the first
    /// edge after them that is not regular (`indfin`), projected on the faces between.
    pub(super) fn regul_projections(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if c.deuxconges {
            return Ok(());
        }
        let mut ic = 0;
        while ic < c.nedge {
            if c.regul[ic] {
                ic += self.regul_projection(c, ic)?;
            }
            ic += 1;
        }
        Ok(())
    }

    /// The regular edges from `ic`: the Hermite curve and its projections; the number of
    /// edges skipped (`nbface - 1`).
    fn regul_projection(&mut self, c: &mut Corner, ic: usize) -> Result<usize, Failure> {
        let (_, icmoins) = indices(c.nedge, ic);
        let mut indfin = indices(c.nedge, ic).0;
        while c.regul[indfin] {
            indfin = indices(c.nedge, indfin).0;
        }
        let (_, indfinmoins) = indices(c.nedge, indfin);
        let mut lface = vec![if c.sharp[icmoins] {
            c.fvive[ic][icmoins].clone()
        } else {
            self.contour_face(c, (icmoins, ic), c.jf[icmoins])?
        }];
        c.nbface = if indfin > icmoins {
            (indfin - icmoins) as i32
        } else {
            (c.nedge - (icmoins - indfin)) as i32
        };
        let nbface = c.nbface as usize;
        let (mut ledge, mut seqpr) = (Vec::new(), Vec::new());
        let mut ii = ic;
        for nf in 1..nbface {
            let (iiplus, _) = indices(c.nedge, ii);
            ledge.push(c.evive[ii].clone());
            seqpr.push(c.p[ii][iiplus]);
            if nf != nbface - 1 {
                lface.push(c.fvive[ii][iiplus].clone());
            }
            ii = iiplus;
        }
        lface.push(if c.sharp[indfin] {
            c.fvive[indfin][indfinmoins].clone()
        } else {
            self.contour_face(c, (indfin, indfinmoins), 3 - c.jf[indfin])?
        });
        let (mut pr, mut cr, mut epj) = (Vec::new(), Vec::new(), Vec::new());
        let ends = [
            &Self::hermite_end(c, icmoins, ic),
            &Self::hermite_end(c, indfin, indfinmoins),
        ];
        let mut out = Projections {
            proj2d: &mut pr,
            cproj: &mut cr,
            eproj: &mut epj,
        };
        c.error = curve_hermite(
            &self.ds,
            ends,
            (c.nbface, &ledge, &lface),
            &mut out,
            &mut seqpr,
        )?;
        store_regul_projections(c, ic, (&pr, &cr, &seqpr))?;
        if !c.sharp[icmoins] && !c.sharp[indfin] {
            let mut ii = icmoins;
            while ii != indfin {
                c.is_g1[ii] = true;
                ii = indices(c.nedge, ii).0;
            }
        }
        Ok(nbface - 1)
    }

    /// Between contours on different faces: the faces crossed (`Fproj`), the edges
    /// between them, and the Hermite curve projected on them (`moresurf`).
    pub(super) fn more_surf_projections(&mut self, c: &mut Corner) -> Result<(), Failure> {
        if c.deuxconges {
            return Ok(());
        }
        for ic in 0..c.nedge {
            if c.oksea[ic] {
                continue;
            }
            let (icplus, _) = indices(c.nedge, ic);
            let face1 = if c.sharp[ic] {
                self.ds_face(c.numfa[ic][icplus])?
            } else {
                self.contour_face(c, (ic, icplus), c.jf[ic])?
            };
            let side2 = if c.jf[icplus] == 1 { 2 } else { 1 };
            let face2 = if c.sharp[icplus] {
                self.ds_face(c.numfa[icplus][ic])?
            } else {
                self.contour_face(c, (icplus, ic), side2)?
            };
            if !face1.is_same(&face2) {
                self.more_surf_projection(c, (ic, icplus), (face1, face2))?;
            }
        }
        Ok(())
    }

    /// The face of index `i` of the structure.
    fn ds_face(&self, i: i32) -> Result<Face, Failure> {
        topods::face(self.ds.shape(i, true))
    }

    /// The Hermite curve from `ic` to `icplus` projected on the faces of `Fproj`, whose
    /// ends are made `face1` and `face2`.
    fn more_surf_projection(
        &mut self,
        c: &mut Corner,
        (ic, icplus): (usize, usize),
        (face1, face2): (Face, Face),
    ) -> Result<(), Failure> {
        if c.fproj.is_empty() {
            c.fproj.push(face1.clone());
            c.fproj.push(face2.clone());
        }
        c.moresurf[ic] = true;
        c.nbface = c.fproj.len() as i32;
        let nbface = c.fproj.len();
        if !c.fproj[nbface - 1].is_same(&face2) {
            c.fproj[nbface - 1] = face2;
        }
        if !c.fproj[0].is_same(&face1) {
            c.fproj[0] = face1;
        }
        let mut edge = Edge::default();
        for nb in 1..nbface {
            cherche_edge1(&c.fproj[nb - 1], &c.fproj[nb], &mut edge)?;
            c.ecom.push(edge.clone());
            c.parcom.push(BRepTool::parameter(
                &topexp::first_vertex(&edge, false),
                &edge,
            )?);
        }
        let (mut pr, mut cr) = (
            Vec::<Handle<dyn Geom2dCurve>>::new(),
            Vec::<Handle<dyn GeomCurve>>::new(),
        );
        let mut eproj = std::mem::take(&mut c.eproj);
        let mut parcom = std::mem::take(&mut c.parcom);
        let ends = [
            &Self::hermite_end(c, ic, icplus),
            &Self::hermite_end(c, icplus, ic),
        ];
        let mut out = Projections {
            proj2d: &mut pr,
            cproj: &mut cr,
            eproj: &mut eproj,
        };
        let error = curve_hermite(
            &self.ds,
            ends,
            (c.nbface, &c.ecom, &c.fproj),
            &mut out,
            &mut parcom,
        );
        (c.eproj, c.parcom) = (eproj, parcom);
        c.error = error?;
        c.ecom.push(nth(&c.ecom, nbface - 1)?);
        c.parcom.push(nth(&c.parcom, nbface - 1)?);
        Ok(())
    }
}

/// The projections of a Hermite curve (pcurves and 3d curves) and its parameters on the
/// edges it crosses.
type HermiteResult<'a> = (
    &'a [Handle<dyn Geom2dCurve>],
    &'a [Handle<dyn GeomCurve>],
    &'a [f64],
);

/// The parameters and projections of the Hermite curve kept on the regular edges from
/// `ic`.
fn store_regul_projections(
    c: &mut Corner,
    ic: usize,
    (pr, cr, seqpr): HermiteResult<'_>,
) -> Result<(), Failure> {
    let mut ii = ic;
    for ind in 1..c.nbface.max(1) as usize {
        let (iiplus, iimoins) = indices(c.nedge, ii);
        c.p[ii][iiplus] = nth(seqpr, ind)?;
        c.p[ii][iimoins] = nth(seqpr, ind)?;
        c.proj2d1[ii] = nth(pr, ind)?;
        c.proj2d2[ii] = nth(pr, ind + 1)?;
        c.cproj1[ii] = nth(cr, ind)?;
        c.cproj2[ii] = nth(cr, ind + 1)?;
        ii = iiplus;
    }
    Ok(())
}

impl ChFi3dBuilder {
    /// Two contours whose ends meet at the same points are joined by the intersection of
    /// their fillets; true when it succeeds (the corner is then done).
    pub(super) fn two_fillets_by_intersection(&mut self, c: &mut Corner) -> Result<bool, Failure> {
        if c.nconges != 2 || c.deuxconges {
            return Ok(false);
        }
        let ic1 = (0..c.nedge).find(|&ic| !c.sharp[ic]).unwrap_or(0);
        let ic2 = (0..c.nedge)
            .rev()
            .find(|&ic| !c.sharp[ic] && ic != ic1)
            .unwrap_or(0);
        let [p1, p2] = self.end_points(c, ic1)?;
        let [p3, p4] = self.end_points(c, ic2)?;
        let intersection = (p1.distance(&p4) <= 1.0e-7 || p1.distance(&p3) <= 1.0e-7)
            && (p2.distance(&p4) <= 1.0e-7 || p2.distance(&p3) <= 1.0e-7);
        if !intersection {
            return Ok(false);
        }
        let (cd1, cd2) = (c.cd(ic1)?.clone(), c.cd(ic2)?.clone());
        perform_two_corner_same_ext(
            &mut self.ds,
            (&cd1, c.index[ic1], c.sens[ic1]),
            (&cd2, c.index[ic2], c.sens[ic2]),
        )
    }

    /// The points of the ends of the contour `ic` toward its previous and next
    /// neighbours.
    fn end_points(&self, c: &Corner, ic: usize) -> Result<[Point3; 2], Failure> {
        let (icplus, icmoins) = indices(c.nedge, ic);
        let mut out = [Point3::new(0., 0., 0.); 2];
        for (k, (other, side)) in [(icmoins, 3 - c.jf[ic]), (icplus, c.jf[ic])]
            .into_iter()
            .enumerate()
        {
            let p2 = calcul_p2d_on_surf(c.cd(ic)?, side, c.i[ic][other], c.p[ic][other])?;
            let indice = surf_index(c, ic, c.i[ic][other], SurfType::Surface)?;
            out[k] = self.surface_value(indice, p2.x(), p2.y())?;
        }
        Ok(out)
    }
}
