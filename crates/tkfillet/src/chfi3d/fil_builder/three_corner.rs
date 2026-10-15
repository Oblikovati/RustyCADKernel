// Rust translation of ChFi3d_FilBuilder::PerformThreeCorner and the file statics
// SearchPivot, SearchFD and ToricCorner (ChFi3d_FilBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of three fillet contours at a vertex: the pivot (the contour whose two
//! neighbours meet on its opposite face), then a torus, a sphere, a walked fillet along
//! a circular or Bezier spine between the neighbours, or a filling.
use super::ChFi3dFilBuilder;
use crate::chfi_ds::{StripeHandle, SurfDataHandle, lock, same_object};
use crate::chfi3d::{InFront, extr_spine_carac, index_of_surf_data, is_in_front, search_pivot};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// The number of surface data of the contour.
pub(super) fn nb_surf_data(st: &StripeHandle) -> i32 {
    lock(st)
        .set_of_surf_data()
        .map(|s| lock(s).len() as i32)
        .unwrap_or(0)
}

/// `SearchFD(DStr, cd1, cd2, sens1, sens2, i1, i2, p1, p2, ind1, ind2, face, sameside,
/// jf1, jf2)`: the surface data of the two contours in front of each other on a common
/// face, searched from their ends at the vertex inwards; `(i1, i2)` (None when there
/// are none) and what `ChFi3d_IsInFront` last wrote, which the native outputs keep even
/// when the search fails.
fn search_fd(
    dstr: &TopOpeBRepDSDataStructure,
    (cd1, cd2): (&StripeHandle, &StripeHandle),
    (sens1, sens2): (i32, i32),
    (ind1, ind2): (i32, i32),
) -> Result<(Option<(i32, i32)>, InFront), Failure> {
    let (id1, id2) = (ind1, ind2);
    let (mut if1, mut if2) = (ind1, ind2);
    let (l1, l2) = (nb_surf_data(cd1), nb_surf_data(cd2));
    let (mut fini1, mut fini2) = (false, false);
    let vtx = Vertex::default();
    let mut res = InFront::default();
    let try_pair = |i1: i32, i2: i32, res: &mut InFront| -> Result<bool, Failure> {
        is_in_front(
            dstr,
            (cd1, cd2),
            (i1, i2),
            (sens1, sens2),
            res,
            &vtx,
            (false, false),
        )
    };
    loop {
        let mut i = id1;
        while i * sens1 <= if1 * sens1 && !fini2 {
            if try_pair(i, if2, &mut res)? {
                return Ok((Some((i, if2)), res));
            }
            i += sens1;
        }
        if !fini1 {
            if1 += sens1;
            if if1 < 1 || if1 > l1 {
                if1 -= sens1;
                fini1 = true;
            }
        }
        let mut i = id2;
        while i * sens2 <= if2 * sens2 && !fini1 {
            if try_pair(if1, i, &mut res)? {
                return Ok((Some((if1, i)), res));
            }
            i += sens2;
        }
        if !fini2 {
            if2 += sens2;
            if if2 < 1 || if2 > l2 {
                if2 -= sens2;
                fini2 = true;
            }
        }
        if fini1 && fini2 {
            return Ok((None, res));
        }
    }
}

/// `ToricCorner(F, rd, rf, v)`: equal radii and a plane face perpendicular to `v`.
fn toric_corner(f: &Face, rd: f64, rf: f64, v: Vector3) -> Result<bool, Failure> {
    if (rd - rf).abs() > precision::confusion() {
        return Ok(false);
    }
    let bs = BRepAdaptorSurface::with_face(f, true)?;
    if bs.get_type()? != SurfaceType::Plane {
        return Ok(false);
    }
    let pos = bs.plane()?.position();
    // The vector is converted to a direction, as the native gp_Dir argument.
    let dv = Direction3::from_vector(v)?;
    let scal1 = pos.x_direction().dot(dv).abs();
    let scal2 = pos.y_direction().dot(dv).abs();
    Ok(scal1 <= precision::confusion() && scal2 <= precision::confusion())
}

/// The state of the corner of three contours (the native arrays: `i[j][k]` and
/// `p[j][k]` are the surface data and the parameter of the contour `j` in front of the
/// contour `k`, `jf[j][k]` its side on their common face `face[3 - j - k]`).
pub(super) struct ThreeCorner {
    pub vtx: Vertex,
    pub index: [i32; 3],
    pub cd: [StripeHandle; 3],
    pub sens: [i32; 3],
    pub face: [Face; 3],
    pub jf: [[i32; 3]; 3],
    pub sameside: [bool; 3],
    pub i: [[i32; 3]; 3],
    pub p: [[f64; 3]; 3],
    pub pivot: usize,
    pub deb: usize,
    pub fin: usize,
    pub pivdif: bool,
    pub c1pointu: bool,
    pub c1toric: bool,
    pub c1spheric: bool,
    pub filling: bool,
}

/// The characteristics of an end of a contour: its point, its tangent and its radius.
#[derive(Clone, Copy, Default)]
pub(super) struct SpineEnd {
    pub p: Point3,
    pub v: Vector3,
    pub r: f64,
}

impl ThreeCorner {
    /// The surface data `i[j][k]` of the contour `j`.
    pub fn sd(&self, j: usize, k: usize) -> Result<SurfDataHandle, Failure> {
        crate::chfi3d::surf_data_at(&self.cd[j], self.i[j][k])
    }
}

impl ChFi3dFilBuilder {
    /// `PerformThreeCorner(Jndex)`: the corner of three fillet contours at the
    /// `jndex`-th vertex.
    pub(super) fn perform_three_corner_impl(&mut self, jndex: i32) -> Result<(), Failure> {
        let Some(mut tc) = self.three_corner_pivot(jndex)? else {
            return self.base.perform_more_three_corner(jndex, 3);
        };
        let ifacdeb = lock(&tc.sd(tc.deb, tc.pivot)?).index(3 - tc.jf[tc.deb][tc.pivot]);
        let ifacfin = lock(&tc.sd(tc.fin, tc.pivot)?).index(3 - tc.jf[tc.fin][tc.pivot]);
        if ifacfin != ifacdeb || tc.i[tc.pivot][tc.deb] != tc.i[tc.pivot][tc.fin] {
            return self.base.perform_more_three_corner(jndex, 3);
        }
        if tc.c1pointu {
            self.toric_or_spheric(&mut tc)?;
        }
        let ends = self.spine_ends(&mut tc)?;
        self.three_corner_surface(&mut tc, &ends)
    }

    /// The contours at the vertex, their facing surface data and the pivot; None when no
    /// pivot is found (the corner is then a plate).
    fn three_corner_pivot(&self, jndex: i32) -> Result<Option<ThreeCorner>, Failure> {
        let vtx = self.base.my_vdata_map.find_key(jndex)?;
        let list = self.base.my_vdata_map.find_from_index(jndex)?.to_vec();
        if list.len() < 3 {
            return Err(Failure::out_of_range(
                "PerformThreeCorner: less than three contours",
            ));
        }
        let cd = [list[0].clone(), list[1].clone(), list[2].clone()];
        let mut index = [0; 3];
        let mut sens = [0; 3];
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
        let mut tc = ThreeCorner {
            vtx,
            index,
            cd,
            sens,
            face: Default::default(),
            jf: [[0; 3]; 3],
            sameside: [false; 3],
            i: [[0; 3]; 3],
            p: [[0.; 3]; 3],
            pivot: 0,
            deb: 0,
            fin: 0,
            pivdif: true,
            c1pointu: false,
            c1toric: false,
            c1spheric: false,
            filling: false,
        };
        let mut oksea = [false; 3];
        for (k, (a, b)) in [(2, (0, 1)), (1, (0, 2)), (0, (1, 2))] {
            oksea[k] = self.search_pair(&mut tc, k, a, b)?;
        }
        let (ok, ss) = (oksea, tc.sameside);
        let (pivot, deb, fin) = if ok[2] && ok[1] && !ss[2] && !ss[1] {
            (0, 1, 2)
        } else if ok[2] && ok[0] && !ss[2] && !ss[0] {
            (1, 2, 0)
        } else if ok[1] && ok[0] && !ss[1] && !ss[0] {
            (2, 0, 1)
        } else if ok[0] && ok[1] && ok[2] {
            let pivot = search_pivot(&tc.sens, &tc.p, self.base.tol2d);
            if pivot < 0 {
                return Ok(None);
            }
            let pivot = pivot as usize;
            tc.pivdif = false;
            let t = self.base.tol2d;
            let p = &tc.p;
            if (p[0][1] - p[0][2]).abs() <= t
                && (p[1][0] - p[1][2]).abs() <= t
                && (p[2][0] - p[2][1]).abs() <= t
            {
                tc.c1pointu = true;
            }
            (pivot, (pivot + 1) % 3, (pivot + 2) % 3)
        } else {
            return Ok(None);
        };
        (tc.pivot, tc.deb, tc.fin) = (pivot, deb, fin);
        Ok(Some(tc))
    }

    /// `SearchFD` between the contours `a` and `b`, their common face being `face[k]`.
    fn search_pair(
        &self,
        tc: &mut ThreeCorner,
        k: usize,
        a: usize,
        b: usize,
    ) -> Result<bool, Failure> {
        let (found, res) = search_fd(
            &self.base.ds,
            (&tc.cd[a], &tc.cd[b]),
            (tc.sens[a], tc.sens[b]),
            (tc.index[a], tc.index[b]),
        )?;
        // The outputs of ChFi3d_IsInFront are kept even when the search fails, as
        // natively (the face of a failed pair may become the face of the pivot).
        tc.p[a][b] = res.p.0;
        tc.p[b][a] = res.p.1;
        tc.face[k] = res.face;
        tc.sameside[k] = res.sameside;
        tc.jf[a][b] = res.jf.0;
        tc.jf[b][a] = res.jf.1;
        let Some((ia, ib)) = found else {
            return Ok(false);
        };
        tc.i[a][b] = ia;
        tc.i[b][a] = ib;
        Ok(true)
    }

    /// A pointed corner: a torus when a face is a plane perpendicular to the contour of
    /// equal radii, else a sphere when the three radii are equal.
    fn toric_or_spheric(&self, tc: &mut ThreeCorner) -> Result<(), Failure> {
        let mut qv = [Vector3::default(); 3];
        let mut qr = [0.; 3];
        for ii in 0..3 {
            let jj = (ii + 1) % 3;
            let (_, v, r) = extr_spine_carac(
                &self.base.ds,
                &tc.cd[jj],
                tc.i[jj][ii],
                tc.p[jj][ii],
                1,
                tc.sens[jj],
            )?;
            (qv[jj], qr[jj]) = (v, r);
        }
        for ii in 0..3 {
            let (jj, kk) = ((ii + 1) % 3, (ii + 2) % 3);
            if toric_corner(&tc.face[ii], qr[jj], qr[kk], qv[ii])? {
                tc.c1toric = true;
                (tc.pivot, tc.deb, tc.fin) = (ii, jj, kk);
                return Ok(());
            }
        }
        let t = self.base.tolapp3d;
        tc.c1spheric = (qr[0] - qr[1]).abs() < t && (qr[0] - qr[2]).abs() < t;
        Ok(())
    }

    /// The ends of the neighbours on the face of the pivot and of the pivot in front of
    /// them; the corner pointed or toric when they meet.
    fn spine_ends(&self, tc: &mut ThreeCorner) -> Result<[SpineEnd; 4], Failure> {
        let (pivot, deb, fin) = (tc.pivot, tc.deb, tc.fin);
        let carac = |j: usize, k: usize, jf: i32| -> Result<SpineEnd, Failure> {
            let (p, v, r) = extr_spine_carac(
                &self.base.ds,
                &tc.cd[j],
                tc.i[j][k],
                tc.p[j][k],
                jf,
                tc.sens[j],
            )?;
            Ok(SpineEnd { p, v, r })
        };
        let ends = [
            carac(deb, pivot, tc.jf[deb][pivot])?,
            carac(fin, pivot, tc.jf[fin][pivot])?,
            carac(pivot, deb, 0)?,
            carac(pivot, fin, 0)?,
        ];
        let [edeb, efin, edp, efp] = ends;
        if !tc.pivdif {
            let ptestdeb = carac(deb, pivot, tc.jf[deb][fin])?.p;
            let ptestfin = carac(fin, pivot, tc.jf[fin][deb])?.p;
            let distest = ptestdeb.distance(&ptestfin);
            if distest < (edp.r + efp.r) * 0.05 {
                tc.filling = true;
            }
            if distest < (edp.r + efp.r) * 0.005 {
                tc.c1pointu = true;
            }
        }
        if !tc.c1pointu {
            let t = self.base.tol2d;
            let p = &tc.p;
            if !tc.pivdif {
                tc.c1pointu = (p[deb][pivot] - p[deb][fin]).abs() <= t
                    && (p[fin][pivot] - p[fin][deb]).abs() <= t;
            }
            if (p[pivot][deb] - p[pivot][fin]).abs() <= t {
                tc.c1toric = toric_corner(&tc.face[pivot], edeb.r, efin.r, edp.v)?;
            }
        }
        Ok(ends)
    }
}

/// The pivot face as a general surface adaptor.
pub(super) fn general(
    h: &rustycad_tkernel::standard::Handle<BRepAdaptorSurface>,
) -> rustycad_tkernel::standard::Handle<dyn Adaptor3dSurface> {
    h.clone().cast()
}
