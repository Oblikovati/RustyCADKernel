// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_connex.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (the Dump functions print to the standard output
// as natively).

//! The `FDSCNX_*` functions: the faces of each edge (per ancestor rank) and the edges of
//! each face among the shapes of the data structure, kept in a per-thread cache
//! (`static thread_local` natively) that `FDSCNX_Prepare` fills. Each function taking
//! the shared data structure has a `_ds` twin on the locked structure, for the callers
//! that already hold its lock.
use super::{TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::cell::RefCell;

type ShapeLists = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The per-thread connexity cache (`GLOBAL_elf1`, `GLOBAL_elf2`, `GLOBAL_fle`,
/// `GLOBAL_FDSCNX_prepared`).
#[derive(Default)]
struct ConnexCache {
    elf1: Option<ShapeLists>,
    elf2: Option<ShapeLists>,
    fle: Option<ShapeLists>,
    prepared: bool,
}

thread_local! {
    static CONNEX: RefCell<ConnexCache> = RefCell::new(ConnexCache::default());
}

/// `FDSCNX_Close()`: the cache released; `fdscnx_close()`.
pub fn fdscnx_close() {
    CONNEX.with(|c| *c.borrow_mut() = ConnexCache::default());
}

/// The faces (or edges) bound to a shape in a cache map, a copy.
fn cached(s: &Shape, pick: impl Fn(&ConnexCache) -> Option<&ShapeLists>) -> Option<List<Shape>> {
    CONNEX.with(|c| pick(&c.borrow()).and_then(|m| m.seek(s).cloned()))
}

/// `FDSCNX_EdgeConnexityShapeIndex(E, HDS, SI)` on the locked structure: the faces of
/// rank `si` (1 or 2) of a known edge; empty otherwise (the shared empty list natively).
pub fn fdscnx_edge_connexity_shape_index_ds(
    e: &Shape,
    ds: &TopOpeBRepDSDataStructure,
    si: i32,
) -> Result<List<Shape>, Failure> {
    let prepared = CONNEX.with(|c| c.borrow().prepared);
    if !prepared || (si != 1 && si != 2) || e.shape_type()? != ShapeEnum::Edge {
        return Ok(List::default());
    }
    if !fdscnx_has_connex_face_ds(e, ds)? || ds.ancestor_rank(e) == 0 {
        return Ok(List::default());
    }
    let found = cached(e, |c| {
        if si == 1 {
            c.elf1.as_ref()
        } else {
            c.elf2.as_ref()
        }
    });
    found.ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "FDSCNX_EdgeConnexityShapeIndex: the edge is not in the map of its rank",
            "TopOpeBRepDS",
        )
    })
}

/// `FDSCNX_EdgeConnexityShapeIndex(E, HDS, SI)`; empty for a null structure;
/// `fdscnx_edge_connexity_shape_index(&e, &hds, 1)?`.
pub fn fdscnx_edge_connexity_shape_index(
    e: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
    si: i32,
) -> Result<List<Shape>, Failure> {
    match hds.get() {
        Some(h) => fdscnx_edge_connexity_shape_index_ds(e, &h.ds(), si),
        None => Ok(List::default()),
    }
}

/// `FDSCNX_EdgeConnexitySameShape(S, HDS)` on the locked structure: the faces of an edge
/// of its own rank, or the edges of a face; empty otherwise.
pub fn fdscnx_edge_connexity_same_shape_ds(
    s: &Shape,
    ds: &TopOpeBRepDSDataStructure,
) -> Result<List<Shape>, Failure> {
    match s.shape_type()? {
        ShapeEnum::Edge => fdscnx_edge_connexity_shape_index_ds(s, ds, ds.ancestor_rank(s)),
        ShapeEnum::Face => Ok(cached(s, |c| c.fle.as_ref()).unwrap_or_default()),
        _ => Ok(List::default()),
    }
}

/// `FDSCNX_EdgeConnexitySameShape(S, HDS)`; `fdscnx_edge_connexity_same_shape(&e, &hds)?`.
pub fn fdscnx_edge_connexity_same_shape(
    s: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<List<Shape>, Failure> {
    let h = hds.get().ok_or_else(null_hds)?;
    fdscnx_edge_connexity_same_shape_ds(s, &h.ds())
}

/// Standard_NullObject of a null structure (a null dereference natively).
fn null_hds() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "FDSCNX: null data structure; expected a TopOpeBRepDS_HDataStructure",
        "TopOpeBRepDS",
    )
}

/// The list bound to a key, appended to.
fn append_bound(m: &mut ShapeLists, key: &Shape, item: &Shape) {
    match m.change_seek(key) {
        Some(l) => {
            l.append(item.clone());
        }
        None => {
            let mut l: List<Shape> = List::default();
            l.append(item.clone());
            m.bind(key.clone(), l);
        }
    }
}

/// `FDSCNX_Prepare(S1, S2, HDS)` on the locked structure: for each face of rank 1 or 2,
/// its edges known to the structure, and for each such edge, its faces of that rank.
pub fn fdscnx_prepare_ds(ds: &TopOpeBRepDSDataStructure) -> Result<(), Failure> {
    let (mut elf1, mut elf2, mut fle) = (
        ShapeLists::default(),
        ShapeLists::default(),
        ShapeLists::default(),
    );
    for i in 1..=ds.nb_shapes() {
        let f = ds.shape(i, true).clone();
        if f.shape_type()? != ShapeEnum::Face {
            continue;
        }
        let rf = ds.ancestor_rank(&f);
        if rf == 0 {
            continue;
        }
        for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !ds.has_shape(&e, true) {
                continue;
            }
            append_bound(&mut fle, &f, &e);
            append_bound(if rf == 1 { &mut elf1 } else { &mut elf2 }, &e, &f);
        }
    }
    CONNEX.with(|c| {
        *c.borrow_mut() = ConnexCache {
            elf1: Some(elf1),
            elf2: Some(elf2),
            fle: Some(fle),
            prepared: true,
        };
    });
    Ok(())
}

/// `FDSCNX_Prepare(S1, S2, HDS)`: a null structure unprepares the cache;
/// `fdscnx_prepare(&s1, &s2, &hds)?`.
pub fn fdscnx_prepare(
    _s1: &Shape,
    _s2: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<(), Failure> {
    match hds.get() {
        Some(h) => fdscnx_prepare_ds(&h.ds()),
        None => {
            CONNEX.with(|c| c.borrow_mut().prepared = false);
            Ok(())
        }
    }
}

/// `FDSCNX_HasConnexFace(S, HDS)` on the locked structure: whether a face or edge of
/// rank 1 or 2 is in the cache (the faces in the face map, the edges in the map of
/// their rank).
pub fn fdscnx_has_connex_face_ds(
    s: &Shape,
    ds: &TopOpeBRepDSDataStructure,
) -> Result<bool, Failure> {
    let t = s.shape_type()?;
    if t != ShapeEnum::Face && t != ShapeEnum::Edge {
        return Ok(false);
    }
    let rs = ds.ancestor_rank(s);
    if rs == 0 {
        return Ok(false);
    }
    Ok(CONNEX.with(|c| {
        let c = c.borrow();
        let pelf = if rs == 1 {
            c.elf1.as_ref()
        } else {
            c.elf2.as_ref()
        };
        let Some(pelf) = pelf else {
            return false;
        };
        if t == ShapeEnum::Edge {
            pelf.is_bound(s)
        } else {
            c.fle.as_ref().is_some_and(|m| m.is_bound(s))
        }
    }))
}

/// `FDSCNX_HasConnexFace(S, HDS)`; false for a null structure;
/// `fdscnx_has_connex_face(&e, &hds)?`.
pub fn fdscnx_has_connex_face(
    s: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<bool, Failure> {
    match hds.get() {
        Some(h) => fdscnx_has_connex_face_ds(s, &h.ds()),
        None => Ok(false),
    }
}

/// `FDSCNX_FaceEdgeConnexFaces(F, E, HDS, LF)` on the locked structure: the other faces
/// of the edge when it is an edge of the face.
pub fn fdscnx_face_edge_connex_faces_ds(
    f: &Shape,
    e: &Shape,
    ds: &TopOpeBRepDSDataStructure,
) -> Result<List<Shape>, Failure> {
    let mut lf: List<Shape> = List::default();
    let loe = fdscnx_edge_connexity_same_shape_ds(f, ds)?;
    if !loe.iter().any(|x| x.is_same(e)) {
        return Ok(lf);
    }
    for x in fdscnx_edge_connexity_same_shape_ds(e, ds)?.iter() {
        if !x.is_same(f) {
            lf.append(x.clone());
        }
    }
    Ok(lf)
}

/// `FDSCNX_FaceEdgeConnexFaces(F, E, HDS, LF)`; `fdscnx_face_edge_connex_faces(&f, &e, &hds)?`.
pub fn fdscnx_face_edge_connex_faces(
    f: &Shape,
    e: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<List<Shape>, Failure> {
    let h = hds.get().ok_or_else(null_hds)?;
    fdscnx_face_edge_connex_faces_ds(f, e, &h.ds())
}

/// The text of `FDSCNX_DumpIndex` for one shape.
fn dump_index_text(ds: &TopOpeBRepDSDataStructure, s: &Shape) -> Result<String, Failure> {
    let mut out = String::new();
    let ls = fdscnx_edge_connexity_same_shape_ds(s, ds)?;
    match s.shape_type()? {
        ShapeEnum::Edge => {
            for x in ls.iter() {
                out.push_str(&format!("{} ", ds.shape_index(x, true)));
            }
        }
        ShapeEnum::Face => {
            for e in ls.iter() {
                for x in fdscnx_face_edge_connex_faces_ds(s, e, ds)?.iter() {
                    out.push_str(&format!("{} ", ds.shape_index(x, true)));
                }
            }
        }
        _ => {}
    }
    Ok(out)
}

/// `FDSCNX_DumpIndex(HDS, I)`: the indices of the connex shapes of the shape `i`
/// printed on the standard output; `fdscnx_dump_index(&hds, 1)?`.
pub fn fdscnx_dump_index(hds: &Handle<TopOpeBRepDSHDataStructure>, i: i32) -> Result<(), Failure> {
    let Some(h) = hds.get() else {
        return Ok(());
    };
    let ds = h.ds();
    if i < 1 || i > ds.nb_shapes() {
        return Ok(());
    }
    print!("{}", dump_index_text(&ds, &ds.shape(i, true).clone())?);
    Ok(())
}

/// The DRAW commands of `FDSCNX_Dump` for one shape.
fn dump_text(ds: &TopOpeBRepDSDataStructure, s: &Shape) -> Result<String, Failure> {
    let mut out = String::new();
    let is = ds.shape_index(s, true);
    let ls = fdscnx_edge_connexity_same_shape_ds(s, ds)?;
    match s.shape_type()? {
        ShapeEnum::Edge if !ls.is_empty() => {
            out.push_str("clear;");
            for x in ls.iter() {
                out.push_str(&format!("tsee f {};", ds.shape_index(x, true)));
            }
            out.push_str(&format!("tsee e {is};### edge {is} connexity\n"));
        }
        ShapeEnum::Face => {
            for e in ls.iter() {
                let ie = ds.shape_index(e, true);
                let lf = fdscnx_face_edge_connex_faces_ds(s, e, ds)?;
                if lf.is_empty() {
                    continue;
                }
                out.push_str(&format!("clear;tsee f {is};"));
                for x in lf.iter() {
                    out.push_str(&format!("tsee f {};", ds.shape_index(x, true)));
                }
                out.push_str(&format!("tsee e {ie};### face {is} connexity\n"));
            }
        }
        _ => {}
    }
    Ok(out)
}

/// `FDSCNX_Dump(HDS, I)`: the DRAW commands showing the connexity of the shape `i`
/// printed on the standard output; `fdscnx_dump_shape(&hds, 1)?`.
pub fn fdscnx_dump_shape(hds: &Handle<TopOpeBRepDSHDataStructure>, i: i32) -> Result<(), Failure> {
    let Some(h) = hds.get() else {
        return Ok(());
    };
    let ds = h.ds();
    if i < 1 || i > ds.nb_shapes() {
        return Ok(());
    }
    print!("{}", dump_text(&ds, &ds.shape(i, true).clone())?);
    Ok(())
}

/// `FDSCNX_Dump(HDS)`: `FDSCNX_Dump(HDS, i)` for each shape; `fdscnx_dump(&hds)?`.
pub fn fdscnx_dump(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
    let Some(h) = hds.get() else {
        return Ok(());
    };
    let n = h.ds().nb_shapes();
    for i in 1..=n {
        fdscnx_dump_shape(hds, i)?;
    }
    Ok(())
}
