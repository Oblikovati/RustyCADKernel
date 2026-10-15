//! The same domain queries (FUN_ds_getVsdm, FUN_ds_sdm, FDS_aresamdom,
//! FDS_EdgeIsConnexToSameDomainFaces, FDS_SIisGIofIofSBAofTofI, FDS_HasSameDomain3d/2d,
//! FUN_ds_getoov).
use super::{fds_idata, has_same_domain, href};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, fdscnx_edge_connexity_same_shape_ds,
};
use crate::top_ope_brep_tool::{TopOpeBRepToolShapeTool, fun_tool_ng_s};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;

/// `FUN_ds_getVsdm(BDS, iV, iVsdm)`: the index of the first same domain shape of the
/// shape `iv` other than itself; `fun_ds_get_vsdm(&ds, 3)` is `Some(4)`.
pub fn fun_ds_get_vsdm(bds: &TopOpeBRepDSDataStructure, iv: i32) -> Option<i32> {
    if iv < 1 || iv > bds.nb_shapes() {
        return None;
    }
    let v = bds.shape(iv, true);
    bds.shape_same_domain(v)
        .iter()
        .find(|vv| !v.is_same(vv))
        .map(|vv| bds.shape_index(vv, true))
}

/// `FUN_ds_sdm(BDS, s1, s2)`: `s2` among the same domain shapes of `s1` (both known);
/// `fun_ds_sdm(&ds, &f1, &f2)`.
pub fn fun_ds_sdm(bds: &TopOpeBRepDSDataStructure, s1: &Shape, s2: &Shape) -> bool {
    if !bds.has_shape(s1, true) || !bds.has_shape(s2, true) {
        return false;
    }
    bds.shape_same_domain(s1).iter().any(|s| s.is_same(s2))
}

/// `FDS_aresamdom(BDS, ES, F1, F2)`: `f2` same domain as `f1`, or both planes with
/// (anti)parallel normals at the ends of `es` on `f1`; `fds_aresamdom(&ds, &e, &f1, &f2)?`.
pub fn fds_aresamdom(
    bds: &TopOpeBRepDSDataStructure,
    es: &Shape,
    f1: &Shape,
    f2: &Shape,
) -> Result<bool, Failure> {
    if bds.shape_same_domain(f1).iter().any(|ss| ss.is_same(f2)) {
        return Ok(true);
    }
    let su1 = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f1)?)?;
    let st1 = GeomAdaptorSurface::with_surface(&su1)?.get_type()?;
    let su2 = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f2)?)?;
    let st2 = GeomAdaptorSurface::with_surface(&su2)?.get_type()?;
    if st1 != SurfaceType::Plane || st2 != SurfaceType::Plane {
        return Ok(false);
    }
    let (p2d1, p2d2) = BRepTool::uv_points_on_face(&topods::edge(es)?, &topods::face(f1)?)?;
    let d1 = fun_tool_ng_s(p2d1, &su1)?;
    let d2 = fun_tool_ng_s(p2d2, &su2)?;
    let dot = d1.dot(d2);
    Ok((1. - dot.abs()).abs() < precision::angular())
}

/// `FDS_aresamdom(BDS, SI, isb1, isb2)`: by indices, false unless an edge and two faces;
/// `fds_aresamdom_index(&ds, 3, 1, 2)?`.
pub fn fds_aresamdom_index(
    bds: &TopOpeBRepDSDataStructure,
    si: i32,
    isb1: i32,
    isb2: i32,
) -> Result<bool, Failure> {
    let es = bds.shape(si, true);
    if es.shape_type()? != ShapeEnum::Edge {
        return Ok(false);
    }
    let f1 = bds.shape(isb1, true);
    if f1.shape_type()? != ShapeEnum::Face {
        return Ok(false);
    }
    let f2 = bds.shape(isb2, true);
    if f2.shape_type()? != ShapeEnum::Face {
        return Ok(false);
    }
    fds_aresamdom(bds, es, f1, f2)
}

/// `FDS_EdgeIsConnexToSameDomainFaces(E, HDS)`: two connex faces of the edge of its
/// rank are same domain; `fds_edge_is_connex_to_same_domain_faces(&e, &hds)?`.
pub fn fds_edge_is_connex_to_same_domain_faces(
    e: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<bool, Failure> {
    let bds = href(hds)?.ds();
    let lf: Vec<Shape> = fdscnx_edge_connexity_same_shape_ds(e, &bds)?
        .iter()
        .cloned()
        .collect();
    if lf.len() < 2 {
        return Ok(false);
    }
    for (k, f1) in lf.iter().enumerate() {
        for f2 in &lf[k..] {
            if fds_aresamdom(&bds, e, f1, f2)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// `FDS_SIisGIofIofSBAofTofI(BDS, SI, I)`: the face before (else after) the transition
/// of `i` has an interference of geometry the EDGE `si`;
/// `fds_si_is_gi_of_i_of_sba_of_t_of_i(&ds, 3, &i)?`.
pub fn fds_si_is_gi_of_i_of_sba_of_t_of_i(
    bds: &TopOpeBRepDSDataStructure,
    si: i32,
    i: &Handle<TopOpeBRepDSInterference>,
) -> Result<bool, Failure> {
    if si == 0 || i.is_null() {
        return Ok(false);
    }
    let ((sb1, ib1, sa1, ia1), _) = fds_idata(i);
    let iface = if sb1 == ShapeEnum::Face {
        ib1
    } else if sa1 == ShapeEnum::Face {
        ia1
    } else {
        return Ok(false);
    };
    for ib in bds.shape_interferences_index(iface, true)?.iter() {
        let (_, (gtb, gb, _, _)) = fds_idata(ib);
        if gtb == TopOpeBRepDSKind::Edge && gb == si {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The same domain shapes of `e` whose configuration passes `ok`, in `plsd` (cleared
/// first) or the first one only without it.
fn has_same_domain_of(
    bds: &TopOpeBRepDSDataStructure,
    e: &Shape,
    mut plsd: Option<&mut List<Shape>>,
    ok: impl Fn(TopOpeBRepDSConfig) -> bool,
) -> bool {
    let lssd = bds.shape_same_domain(e);
    if let Some(l) = plsd.as_deref_mut() {
        l.clear();
    }
    let mut hsd = false;
    for esd in lssd.iter() {
        if !ok(bds.same_domain_ori(esd)) {
            continue;
        }
        hsd = true;
        match plsd.as_deref_mut() {
            Some(l) => {
                l.append(esd.clone());
            }
            None => break,
        }
    }
    hsd
}

/// `FDS_HasSameDomain3d(BDS, E, PLSD)`: same domain shapes of UNSHGEOMETRY
/// configuration (all of them in `plsd`); `fds_has_same_domain3d(&ds, &e, None)`.
pub fn fds_has_same_domain3d(
    bds: &TopOpeBRepDSDataStructure,
    e: &Shape,
    plsd: Option<&mut List<Shape>>,
) -> bool {
    has_same_domain_of(bds, e, plsd, |c| c == TopOpeBRepDSConfig::UnshGeometry)
}

/// `FDS_HasSameDomain2d(BDS, E, PLSD)`: same domain shapes SAMEORIENTED or
/// DIFFORIENTED (all of them in `plsd`); `fds_has_same_domain2d(&ds, &e, None)`.
pub fn fds_has_same_domain2d(
    bds: &TopOpeBRepDSDataStructure,
    e: &Shape,
    plsd: Option<&mut List<Shape>>,
) -> bool {
    has_same_domain_of(bds, e, plsd, |c| {
        c == TopOpeBRepDSConfig::SameOriented || c == TopOpeBRepDSConfig::DiffOriented
    })
}

/// `FUN_ds_getoov(v, BDS, oov)`: the first same domain shape of `v` other than itself;
/// `fun_ds_getoov(&v, &ds)`.
pub fn fun_ds_getoov(v: &Shape, bds: &TopOpeBRepDSDataStructure) -> Option<Shape> {
    bds.shape_same_domain(v)
        .iter()
        .find(|c| !c.is_same(v))
        .cloned()
}

/// `FUN_ds_getoov(v, HDS, oov)`: the same for a shape of the structure;
/// `fun_ds_getoov_hds(&v, &hds)?`.
pub fn fun_ds_getoov_hds(
    v: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<Option<Shape>, Failure> {
    let bds = href(hds)?.ds();
    if !has_same_domain(&bds, v) {
        return Ok(None);
    }
    Ok(fun_ds_getoov(v, &bds))
}
