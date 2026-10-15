// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_BuildShape.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_BuildShape`: shells and solids built from a list of faces. The faces are
//! grouped by edge connexity, each group oriented from a reference face into a shell; a
//! closed manifold shell gives a solid, the shells inside another become its cavities.
use crate::errors::construction_error;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape, Shell, Solid};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// The shells and the shells nested in each of them.
type NestedShells = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `LocOpe_BuildShape`: the shape built from the faces of a list.
///
/// ```
/// use rustycad_tkfeat::loc_ope::BuildShape;
/// use rustycad_tkernel::ncollection::List;
/// let build = BuildShape::with_faces(&List::default())?;
/// assert!(build.shape().is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct BuildShape {
    res: Shape,
}

/// `Add`: marks the edge `ind` and adds its faces to `map_f`, then recursively the
/// edges of each new face.
fn add(
    ind: usize,
    map_i: &mut Map<i32>,
    map_f: &mut IndexedMapOfShape,
    map_ef: &IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    if !map_i.add(ind as i32) {
        return Err(construction_error(
            "LocOpe_BuildShape::Add",
            "edge visited twice",
        ));
    }
    for face in map_ef.find_from_index(ind)?.iter() {
        if map_f.contains(face) {
            continue;
        }
        map_f.add(face.clone());
        for edg in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let indedg = map_ef.find_index(&edg);
            if indedg == 0 {
                return Err(construction_error(
                    "LocOpe_BuildShape::Add",
                    "face edge missing from the edge map",
                ));
            }
            if !map_i.contains(&(indedg as i32)) {
                add(indedg, map_i, map_f, map_ef)?;
            }
        }
    }
    Ok(())
}

/// The first face of `map_if` holding `edg`, with the orientation of the edge in it.
fn face_sharing(
    edg: &Shape,
    map_f: &IndexedMapOfShape,
    map_if: &Map<i32>,
) -> Result<Option<(i32, Orientation)>, Failure> {
    for &key in map_if.iter() {
        let new_f = map_f.find_key(key as usize)?;
        for e in Explorer::new(new_f, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if e.is_same(edg) {
                return Ok(Some((key, e.orientation())));
            }
        }
    }
    Ok(None)
}

/// `Propagate`: adds to the shell `sh` the faces of `map_if` sharing an edge with `f`,
/// oriented against `f` along it, then propagates from each of them.
fn propagate(
    f: &Shape,
    sh: &mut Shape,
    map_f: &IndexedMapOfShape,
    map_if: &mut Map<i32>,
) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let indf = map_f.find_index(f) as i32;
    if !map_if.contains(&indf) {
        return Ok(());
    }
    map_if.remove(&indf);
    if map_if.is_empty() {
        return Ok(());
    }
    for edg in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let ored1 = edg.orientation();
        if ored1 == Orientation::Internal || ored1 == Orientation::External {
            continue;
        }
        let Some((key, ored2)) = face_sharing(&edg, map_f, map_if)? else {
            continue;
        };
        let mut fto_add = map_f.find_key(key as usize)?.clone();
        let added = if ored2 == ored1 {
            fto_add.reverse();
            true
        } else {
            ored2 == top_abs::reverse(ored1)
        };
        if added {
            b.add(sh, &fto_add)?;
            propagate(&fto_add, sh, map_f, map_if)?;
        }
    }
    Ok(())
}

/// `IsInside`: the first vertex of `s1` classified IN or OUT of the solid `s2` decides;
/// true when none is.
fn is_inside(s1: &Shape, s2: &Shape) -> Result<bool, Failure> {
    let mut class = BRepClass3dSolidClassifier::with_shape(s2)?;
    for v in Explorer::new(s1, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vtx = topods::vertex(&v)?;
        class.perform(&BRepTool::pnt(&vtx)?, BRepTool::vertex_tolerance(&vtx)?)?;
        match class.state() {
            State::In => return Ok(true),
            State::Out => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}

/// A solid of `shells`.
fn make_solid(shells: &[Shape]) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut solid = Solid::default();
    b.make_solid(&mut solid);
    let mut solid = solid.into_shape();
    for shell in shells {
        b.add(&mut solid, shell)?;
    }
    Ok(solid)
}

/// The faces of `l` once each, in a compound.
fn distinct_faces(l: &List<Shape>) -> Result<(IndexedMapOfShape, Shape), Failure> {
    let b = BRepBuilder::default();
    let mut c = Compound::default();
    b.make_compound(&mut c);
    let mut c = c.into_shape();
    let mut map_f = IndexedMapOfShape::default();
    for s in l.iter() {
        if s.shape_type()? == ShapeEnum::Face && !map_f.contains(s) {
            map_f.add(s.clone());
            b.add(&mut c, s)?;
        }
    }
    Ok((map_f, c))
}

/// The shell of the connected faces `map_f`, oriented from the first FORWARD or
/// REVERSED face (the INTERNAL and EXTERNAL faces added as they are), and whether every
/// face is FORWARD or REVERSED.
fn oriented_shell(map_f: &IndexedMapOfShape) -> Result<(Shape, bool), Failure> {
    let b = BRepBuilder::default();
    let mut manifold = true;
    let mut face_ref = Shape::default();
    let mut map_if: Map<i32> = Map::default();
    for j in 1..=map_f.extent() {
        let orient = map_f.find_key(j)?.orientation();
        if orient == Orientation::Internal || orient == Orientation::External {
            manifold = false;
        } else if face_ref.is_null() {
            face_ref = map_f.find_key(j)?.clone();
        }
        map_if.add(j as i32);
    }
    let mut new_sh = Shell::default();
    b.make_shell(&mut new_sh);
    let mut new_sh = new_sh.into_shape();
    if !manifold && face_ref.is_null() {
        // on a un paquet de faces. pas d'orientation possible ?
        for j in 1..=map_f.extent() {
            b.add(&mut new_sh, map_f.find_key(j)?)?;
        }
    } else {
        // orienter ce qu`on peut
        if !manifold {
            for j in 1..=map_f.extent() {
                let face = map_f.find_key(j)?;
                if face.orientation() == Orientation::Internal
                    || face.orientation() == Orientation::External
                {
                    b.add(&mut new_sh, face)?;
                    map_if.remove(&(j as i32));
                }
            }
        }
        b.add(&mut new_sh, &face_ref)?;
        propagate(&face_ref, &mut new_sh, map_f, &mut map_if)?;
    }
    new_sh.set_closed(BRepTool::is_closed(&new_sh)?)?;
    Ok((new_sh, manifold))
}

/// Whether every FORWARD or REVERSED edge of the shell has two faces, or one when it is
/// degenerated.
fn is_closed_manifold(new_sh: &Shape) -> Result<bool, Failure> {
    let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(new_sh, ShapeEnum::Edge, ShapeEnum::Face, &mut map_ef)?;
    for k in 1..=map_ef.extent() {
        let ed = topods::edge(map_ef.find_key(k)?)?;
        let ori_ed = ed.orientation();
        if ori_ed == Orientation::Internal || ori_ed == Orientation::External {
            continue;
        }
        let nbfac = map_ef.find_from_index(k)?.len();
        if nbfac > 2 {
            // peu probable
            return Ok(false);
        } else if nbfac == 1 && !BRepTool::degenerated(&ed)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A closed shell oriented to bound a finite solid (reversed when the infinite point
/// is IN).
fn outward_shell(new_sh: &Shape) -> Result<Shape, Failure> {
    let new_so = make_solid(std::slice::from_ref(new_sh))?; // newSh est FORWARD
    let mut class = BRepClass3dSolidClassifier::with_shape(&new_so)?;
    class.perform_infinite_point(precision::confusion())?;
    if class.state() == State::In {
        return Ok(new_sh.oriented(Orientation::Reversed));
    }
    Ok(new_sh.clone())
}

/// The shells of each connex group of faces: the closed manifold ones in `lresult`, the
/// others INTERNAL in `lshell`.
fn connex_shells(
    c: &Shape,
    lresult: &mut List<Shape>,
    lshell: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(c, ShapeEnum::Edge, ShapeEnum::Face, &mut map_ef)?;
    let mut map_i: Map<i32> = Map::default();
    let nbedges = map_ef.extent();
    loop {
        // Recherche du premier edge non traite
        if let Some(i) = (1..=nbedges).find(|&i| !map_i.contains(&(i as i32))) {
            let mut map_f = IndexedMapOfShape::default();
            add(i, &mut map_i, &mut map_f, &map_ef)?;
            let (new_sh, manifold) = oriented_shell(&map_f)?;
            if manifold && is_closed_manifold(&new_sh)? {
                lresult.append(outward_shell(&new_sh)?);
            } else {
                lshell.append(new_sh.oriented(Orientation::Internal));
            }
        }
        if map_i.extent() >= nbedges {
            return Ok(());
        }
    }
}

/// The shells of `lresult` and those inside each, the nested ones dropped as keys.
fn nested_shells(lresult: &List<Shape>) -> Result<NestedShells, Failure> {
    let mut imb_sh: NestedShells = DataMap::default();
    let mut lintern: Vec<Shape> = Vec::new();
    for sh in lresult.iter() {
        let tempo = make_solid(std::slice::from_ref(sh))?;
        let inside = imb_sh.bound(sh.clone(), List::default());
        for sh2 in lresult.iter() {
            if !sh2.is_same(sh) && is_inside(sh2, &tempo)? {
                lintern.push(sh2.clone());
                inside.append(sh2.clone());
            }
        }
    }
    // LPA 07/10/98: on vire les shells imbriques comme
    // etant aussi des solides a part entiere.
    for sh in &lintern {
        imb_sh.unbind(sh);
    }
    Ok(imb_sh)
}

/// The solids of the nested shells: one per shell with cavities first, then one per
/// remaining shell.
fn nested_solids(mut imb_sh: NestedShells) -> Result<List<Shape>, Failure> {
    let mut lsolid = List::default();
    while imb_sh.extent() != 0 {
        let with_cavities =
            imb_sh
                .items()
                .find(|(_, inside)| !inside.is_empty())
                .map(|(key, inside)| {
                    (
                        key.clone(),
                        inside.iter().map(|s| s.reversed()).collect::<Vec<_>>(),
                    )
                });
        if let Some((key, cavities)) = with_cavities {
            let mut shells = vec![key.clone()];
            shells.extend(cavities);
            lsolid.append(make_solid(&shells)?);
            imb_sh.unbind(&key);
        } else {
            for (key, _) in imb_sh.items() {
                lsolid.append(make_solid(std::slice::from_ref(key))?);
            }
            imb_sh.clear(false);
        }
    }
    Ok(lsolid)
}

impl BuildShape {
    /// `LocOpe_BuildShape()`: a null shape; `BuildShape::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_BuildShape(L)`: performed on the faces of `l`;
    /// `BuildShape::with_faces(&faces)?`.
    pub fn with_faces(l: &List<Shape>) -> Result<Self, Failure> {
        let mut build = Self::new();
        build.perform(l)?;
        Ok(build)
    }

    /// `Perform(L)`: the solid, the shell or the compound of the solids and open shells
    /// built from the faces of `l` (only the faces are used); null without face;
    /// `build.perform(&faces)?`.
    pub fn perform(&mut self, l: &List<Shape>) -> Result<(), Failure> {
        self.res.nullify();
        let (map_f, c) = distinct_faces(l)?;
        if map_f.extent() == 0 {
            return Ok(()); // no face
        }
        let mut lshell = List::default();
        let mut lresult = List::default();
        connex_shells(&c, &mut lresult, &mut lshell)?;
        // on a une list de shells dans lresult. on suppose qu`ils ne s`intersectent pas.
        // il faut classifier les shells orientes pour en faire des solides...
        // on n`accepte qu`1 niveau d'imbrication
        let lsolid = nested_solids(nested_shells(&lresult)?)?;
        self.res = Self::result_of(&lsolid, &lshell)?;
        Ok(())
    }

    /// The single solid, the single open shell, or a compound of all of them.
    fn result_of(lsolid: &List<Shape>, lshell: &List<Shape>) -> Result<Shape, Failure> {
        if lsolid.len() == 1 && lshell.is_empty() {
            return Ok(lsolid.first().cloned().unwrap_or_default());
        }
        if lsolid.is_empty() && lshell.len() == 1 {
            return Ok(lshell.first().cloned().unwrap_or_default());
        }
        let b = BRepBuilder::default();
        let mut res = Compound::default();
        b.make_compound(&mut res);
        let mut res = res.into_shape();
        for s in lsolid.iter().chain(lshell.iter()) {
            b.add(&mut res, s)?;
        }
        Ok(res)
    }

    /// `Shape()`: the result; `build.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.res
    }
}
