// Rust translation of ChFi3d_ChBuilder::PerformThreeCorner (ChFi3d_ChBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of three chamfer contours at a vertex: the pivot and its neighbours, the
//! points where the chamfers meet, then a plane corner (a triangle, or four coplanar
//! points) or a filling.
use super::ChFi3dChBuilder;
use crate::chfi_ds::{StripeHandle, SurfDataHandle, lock, same_object};
use crate::chfi3d::{
    InFront, SearchFd, bound_surf, computes_int_pc, index_of_surf_data, is_in_front, search_fd,
    search_pivot, surf_data_at,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Point2, Point3};

/// The state of the corner of three chamfers (the native arrays, as in the fillet
/// corner: `i[j][k]`, `p[j][k]` and `jf[j][k]` of the contour `j` in front of `k`).
pub(super) struct ChThreeCorner {
    pub vtx: Vertex,
    pub index: [i32; 3],
    pub cd: [StripeHandle; 3],
    pub sens: [i32; 3],
    pub face: [Face; 3],
    pub jf: [[i32; 3]; 3],
    pub sameside: [bool; 3],
    pub i: [[i32; 3]; 3],
    pub p: [[f64; 3]; 3],
    pub oksea: [bool; 3],
    pub pivot: usize,
    pub deb: usize,
    pub fin: usize,
    pub corner_all_same: bool,
}

/// The face of the pivot, the chamfer of the pivot (natively also given the unused
/// domains `IFac` and `ISurf`) and the points where the chamfers meet: `p2d[k]`,
/// `p3d[k]` (the end of the pivot in front of the other contours, and on the face of the
/// pivot), `psom` (where the neighbours meet).
pub(super) struct CornerPoints {
    pub fac: Handle<BRepAdaptorSurface>,
    pub surf: Handle<GeomAdaptorSurface>,
    pub p2d: [Point2; 4],
    pub p3d: [Point3; 4],
    pub psom: Point3,
    pub norpl: rustycad_tkmath::gp::Vector3,
}

impl ChThreeCorner {
    /// The surface data `i[j][k]` of the contour `j`.
    pub fn sd(&self, j: usize, k: usize) -> Result<SurfDataHandle, Failure> {
        surf_data_at(&self.cd[j], self.i[j][k])
    }
}

/// The number of surface data of the contour.
fn nb_surf_data(st: &StripeHandle) -> i32 {
    lock(st)
        .set_of_surf_data()
        .map(|s| lock(s).len() as i32)
        .unwrap_or(0)
}

impl ChFi3dChBuilder {
    /// `PerformThreeCorner(Jndex)`: the corner of three chamfer contours at the
    /// `jndex`-th vertex.
    pub(super) fn perform_three_corner_impl(&mut self, jndex: i32) -> Result<(), Failure> {
        let mut tc = self.ch_three_setup(jndex)?;
        self.ch_three_pivot(&mut tc)?;
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        if tc.i[pivot][deb] != tc.i[pivot][fin] {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "coin mutant non programme",
            ));
        }
        if tc.oksea[pivot]
            && (tc.i[deb][pivot] != tc.i[deb][fin] || tc.i[fin][pivot] != tc.i[fin][deb])
        {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "coin sur plusieurs faces non programme",
            ));
        }
        let pts = self.corner_points(&mut tc)?;
        self.ch_three_corner_surface(&mut tc, pts)
    }

    /// The contours at the vertex and their facing chamfers.
    fn ch_three_setup(&self, jndex: i32) -> Result<ChThreeCorner, Failure> {
        let vtx = self.base.my_vdata_map.find_key(jndex)?;
        let list = self.base.my_vdata_map.find_from_index(jndex)?.to_vec();
        if list.len() < 3 {
            return Err(Failure::out_of_range(
                "PerformThreeCorner: less than three contours",
            ));
        }
        let cd = [list[0].clone(), list[1].clone(), list[2].clone()];
        let (mut index, mut sens) = ([0; 3], [0; 3]);
        for k in 0..3 {
            (index[k], sens[k]) = index_of_surf_data(&vtx, &cd[k])?;
        }
        if same_object(&cd[0], &cd[1]) {
            index[1] = nb_surf_data(&cd[1]);
            sens[1] = -1;
        } else if same_object(&cd[1], &cd[2]) || same_object(&cd[0], &cd[2]) {
            index[2] = nb_surf_data(&cd[2]);
            sens[2] = -1;
        }
        let mut tc = ChThreeCorner {
            vtx,
            index,
            cd,
            sens,
            face: Default::default(),
            jf: [[0; 3]; 3],
            sameside: [false; 3],
            i: [[0; 3]; 3],
            p: [[0.; 3]; 3],
            oksea: [false; 3],
            pivot: 0,
            deb: 0,
            fin: 0,
            corner_all_same: false,
        };
        for (k, (a, b)) in [(2, (0, 1)), (1, (0, 2)), (0, (1, 2))] {
            let mut res = SearchFd::default();
            tc.oksea[k] = search_fd(
                &self.base.ds,
                (&tc.cd[a], &tc.cd[b]),
                (tc.sens[a], tc.sens[b]),
                (tc.index[a], tc.index[b]),
                &mut res,
            )?;
            store_pair(&mut tc, k, (a, b), (res.i, &res.front));
        }
        Ok(tc)
    }

    /// The pivot: the contour whose two neighbours meet it on opposite sides; the third
    /// pair intersected when it was not found.
    fn ch_three_pivot(&self, tc: &mut ChThreeCorner) -> Result<(), Failure> {
        let (ok, ss) = (tc.oksea, tc.sameside);
        let mut okinter = true;
        let mut third: Option<(usize, usize, usize)> = None;
        if ok[2] && ok[1] && !ss[2] && !ss[1] {
            (tc.pivot, tc.deb, tc.fin) = (0, 1, 2);
            if !ok[0] {
                third = Some((0, 1, 2));
            }
        } else if ok[2] && ok[0] && !ss[2] && !ss[0] {
            (tc.pivot, tc.deb, tc.fin) = (1, 2, 0);
            if !ok[1] {
                third = Some((1, 0, 2));
            }
        } else if ok[1] && ok[0] && !ss[1] && !ss[0] {
            (tc.pivot, tc.deb, tc.fin) = (2, 0, 1);
            if !ok[2] {
                third = Some((2, 0, 1));
            }
        } else if ok[0] && ok[1] && ok[2] {
            let pivot = search_pivot(&tc.sens, &tc.p, self.base.tol2d).max(0) as usize;
            (tc.pivot, tc.deb, tc.fin) = (pivot, (pivot + 1) % 3, (pivot + 2) % 3);
            tc.corner_all_same = true;
        } else {
            return Err(Failure::new("FD en vis a vis non trouvees"));
        }
        if let Some((k, a, b)) = third {
            let mut res = InFront::default();
            okinter = is_in_front(
                &self.base.ds,
                (&tc.cd[a], &tc.cd[b]),
                (tc.i[a][b], tc.i[b][a]),
                (tc.sens[a], tc.sens[b]),
                &mut res,
                &tc.vtx,
                (false, true),
            )?;
            let ij = (tc.i[a][b], tc.i[b][a]);
            store_pair(tc, k, (a, b), (ij, &res));
        }
        if !okinter {
            return Err(Failure::new("Echec intersection PCurves OnCommonFace"));
        }
        Ok(())
    }

    /// The face of the pivot, the chamfer of the pivot (natively also given the unused
    /// domains `IFac` and `ISurf`) and the points where the chamfers meet each other and
    /// the face of the pivot.
    fn corner_points(&self, tc: &mut ChThreeCorner) -> Result<CornerPoints, Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let (fddeb, fdfin, fdpiv) = (tc.sd(deb, pivot)?, tc.sd(fin, pivot)?, tc.sd(pivot, deb)?);
        let fac = Handle::new(BRepAdaptorSurface::with_face(&tc.face[pivot], true)?);
        let surf = bound_surf(&self.base.ds, &fdpiv, tc.jf[pivot][deb], tc.jf[pivot][fin])?;
        let hs = |fd: &SurfDataHandle| -> Result<Handle<GeomAdaptorSurface>, Failure> {
            Ok(Handle::new(GeomAdaptorSurface::with_surface(
                self.base.ds.surface(lock(fd).surf()).surface(),
            )?))
        };
        let (hsdeb, hsfin, hspiv) = (hs(&fddeb)?, hs(&fdfin)?, hs(&fdpiv)?);
        let fi = |fd: &SurfDataHandle, on_s: i32| lock(fd).interference(on_s).clone();
        let mut p3d = [Point3::default(); 4];
        let (a, b, pt) = computes_int_pc(
            (&fi(&fdpiv, tc.jf[pivot][deb]), &hspiv),
            (&fi(&fddeb, tc.jf[deb][pivot]), &hsdeb),
            (tc.p[pivot][deb], tc.p[deb][pivot]),
        )?;
        (tc.p[pivot][deb], tc.p[deb][pivot], p3d[fin]) = (a, b, pt);
        let (a, b, pt) = computes_int_pc(
            (&fi(&fdpiv, tc.jf[pivot][fin]), &hspiv),
            (&fi(&fdfin, tc.jf[fin][pivot]), &hsfin),
            (tc.p[pivot][fin], tc.p[fin][pivot]),
        )?;
        (tc.p[pivot][fin], tc.p[fin][pivot], p3d[deb]) = (a, b, pt);
        let (a, b, psom) = computes_int_pc(
            (&fi(&fddeb, tc.jf[deb][fin]), &hsdeb),
            (&fi(&fdfin, tc.jf[fin][deb]), &hsfin),
            (tc.p[deb][fin], tc.p[fin][deb]),
        )?;
        (tc.p[deb][fin], tc.p[fin][deb]) = (a, b);
        let mut p2d = [Point2::default(); 4];
        p2d[pivot] = fi(&fddeb, tc.jf[deb][fin])
            .pcurve_on_face()
            .value(tc.p[deb][pivot])?;
        p2d[3] = fi(&fdfin, tc.jf[fin][deb])
            .pcurve_on_face()
            .value(tc.p[fin][pivot])?;
        p2d[fin] = fi(&fdpiv, tc.jf[pivot][deb])
            .pcurve_on_surf()
            .value(tc.p[pivot][deb])?;
        p2d[deb] = fi(&fdpiv, tc.jf[pivot][fin])
            .pcurve_on_surf()
            .value(tc.p[pivot][fin])?;
        let d1 = fac.d1(p2d[pivot].x(), p2d[pivot].y())?;
        p3d[pivot] = d1.point;
        let norpl = d1.d1u.crossed(d1.d1v);
        p3d[3] = fac.value(p2d[3].x(), p2d[3].y())?;
        Ok(CornerPoints {
            fac,
            surf,
            p2d,
            p3d,
            psom,
            norpl,
        })
    }
}

/// The pair `(a, b)` found in front of each other on `face[k]`.
fn store_pair(
    tc: &mut ChThreeCorner,
    k: usize,
    (a, b): (usize, usize),
    (ij, res): ((i32, i32), &InFront),
) {
    tc.i[a][b] = ij.0;
    tc.i[b][a] = ij.1;
    tc.p[a][b] = res.p.0;
    tc.p[b][a] = res.p.1;
    tc.face[k] = res.face.clone();
    tc.sameside[k] = res.sameside;
    tc.jf[a][b] = res.jf.0;
    tc.jf[b][a] = res.jf.1;
}
