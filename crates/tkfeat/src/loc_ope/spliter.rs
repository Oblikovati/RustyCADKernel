// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Spliter.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_Spliter`: the split of a shape by the wires lying on it: the vertices of the
//! wires substituted to the shape vertices they touch, the edges cut at the others, the
//! faces cut by the rebuilt wires, then the faces on the left of the wires found.
use super::split_shape::to_vec;
use super::{BuildWires, SplitShape, WiresOnShape};
use crate::errors::{construction_error, not_done, null_object};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;

/// The shapes and their descendants (`myMap`).
type Descendants = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `LocOpe_Spliter`: the shape split by projected wires.
///
/// ```
/// use rustycad_tkfeat::loc_ope::Spliter;
/// let spliter = Spliter::new();
/// assert!(!spliter.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Spliter {
    shape: Shape,
    done: bool,
    res: Shape,
    map: Descendants,
    d_left: List<Shape>,
    left: List<Shape>,
}

/// `Put(S, theMap)`: binds `s` and its sub-shapes to themselves.
fn put(s: &Shape, the_map: &mut Descendants) {
    if the_map.is_bound(s) {
        return;
    }
    let mut l = List::default();
    l.append(s.clone());
    the_map.bind(s.clone(), l);
    for child in ShapeIterator::new(s, true, true) {
        put(&child, the_map);
    }
}

/// `RebuildWires(ledge, PW)`: the edges chained into wires.
fn rebuild_wires(ledge: &List<Shape>, pw: &WiresOnShape) -> Result<List<Shape>, Failure> {
    let the_build = BuildWires::with_edges(ledge, pw)?;
    if !the_build.is_done() {
        return Err(construction_error(
            "LocOpe_Spliter::RebuildWires",
            "the wires were not built",
        ));
    }
    Ok(the_build.result()?.clone())
}

/// The located 3D curve of an edge and its range.
fn located_curve(
    e: &Edge,
) -> Result<
    (
        rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
        f64,
        f64,
    ),
    Failure,
> {
    let located = BRepTool::curve_with_location(e)?;
    let mut c = located.curve;
    if !located.location.is_identity() {
        let curve = c
            .get()
            .ok_or_else(|| null_object("LocOpe_Spliter::Select", "edge without 3D curve"))?;
        c = curve.transformed(located.location.transformation())?.cast();
    }
    Ok((c, located.first, located.last))
}

/// `Select(Ebase, lsubs)`: keeps the edge of `lsubs` nearest to the middle of `ebase`
/// (none when no projection exists).
fn select(ebase: &Edge, lsubs: &mut Vec<Shape>) -> Result<(), Failure> {
    // Choix d`un point
    let (c, f, l) = located_curve(ebase)?;
    let pt = c
        .get()
        .ok_or_else(|| null_object("LocOpe_Spliter::Select", "edge without 3D curve"))?
        .value((f + l) / 2.)?;
    let mut proj = ProjectPointOnCurve::new();
    let (mut dmin, mut imin) = (f64::MAX, 0);
    for (i, s) in lsubs.iter().enumerate() {
        let (c, f, l) = located_curve(&topods::edge(s)?)?;
        proj.init_range(pt, &c, f, l)?;
        if proj.nb_points() > 0 && proj.lower_distance()? < dmin {
            imin = i + 1;
            dmin = proj.lower_distance()?;
        }
    }
    if imin == 0 {
        lsubs.clear();
    } else {
        let kept = lsubs[imin - 1].clone();
        *lsubs = vec![kept];
    }
    Ok(())
}

/// The descendant of `e2`'s pieces matching the wire edge `e1` by its vertices (or, for
/// a closed `e1`, by its tangent at the start), oriented as `e1` runs.
fn matching_pieces(e1: &Edge, pieces: &List<Shape>) -> Result<Vec<Shape>, Failure> {
    let (vf1, vl1) = topexp::vertices(e1, false);
    let mut lsubs = Vec::new();
    for s in pieces.iter() {
        let e2 = topods::edge(s)?;
        let (vf2, vl2) = topexp::vertices(&e2, false);
        if !vl1.is_same(&vf1) {
            if vf1.is_same(&vf2) && vl1.is_same(&vl2) {
                lsubs.push(e2.oriented(Orientation::Forward));
            } else if vf1.is_same(&vl2) && vl1.is_same(&vf2) {
                lsubs.push(e2.oriented(Orientation::Reversed));
            }
        } else if vf2.is_same(&vl2) && vl2.is_same(&vl1) {
            // discrimination sur les tangentes (tout au meme point)
            let d1 = |e: &Edge| -> Result<rustycad_tkmath::gp::Vector3, Failure> {
                let located = BRepTool::curve_with_location(e)?;
                let curve = located.curve.get().ok_or_else(|| {
                    null_object("LocOpe_Spliter::Perform", "edge without 3D curve")
                })?;
                Ok(curve
                    .d1(located.first)?
                    .1
                    .transformed(located.location.transformation()))
            };
            let orient = if d1(e1)?.dot(d1(&e2)?) > 0. {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            lsubs.push(e2.oriented(orient));
        }
    }
    Ok(lsubs)
}

impl Spliter {
    /// `LocOpe_Spliter()`; `Spliter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_Spliter(S)`: the algorithm on `s`; `Spliter::with_shape(&s)`.
    pub fn with_shape(s: &Shape) -> Self {
        Self {
            shape: s.clone(),
            ..Self::default()
        }
    }

    /// `Init(S)`; `spliter.init(&s)`.
    pub fn init(&mut self, s: &Shape) {
        self.shape = s.clone();
        self.done = false;
    }

    /// `IsDone()`; `spliter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ResultingShape()`; StdFail_NotDone before Perform; `spliter.resulting_shape()?`.
    pub fn resulting_shape(&self) -> Result<&Shape, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::ResultingShape"));
        }
        Ok(&self.res)
    }

    /// `Shape()`: the initial shape; `spliter.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// `DirectLeft()`: the faces on the left of the wires; `spliter.direct_left()?`.
    pub fn direct_left(&self) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::DirectLeft"));
        }
        Ok(&self.d_left)
    }

    /// `Left()`: the faces of the left part, grown from `DirectLeft` across the edges
    /// that are not wire edges; `spliter.left()?`.
    pub fn left(&self) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::Left"));
        }
        Ok(&self.left)
    }

    /// `DescendantShapes(S)`: the descendants of `s` (none for an unknown shape);
    /// `spliter.descendant_shapes(&s)?`.
    pub fn descendant_shapes(&self, s: &Shape) -> Result<List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_Spliter::DescendantShapes"));
        }
        Ok(self.map.seek(s).cloned().unwrap_or_default())
    }

    fn first_of(&self, s: &Shape) -> Result<Shape, Failure> {
        self.map
            .seek(s)
            .and_then(|l| l.first().cloned())
            .ok_or_else(|| null_object("LocOpe_Spliter::Perform", "shape without descendant"))
    }

    /// 1ere etape : substitution des vertex: each wire vertex bound to a shape vertex
    /// replaces it (its tolerance covering the distance); the descendants follow.
    fn substitute_vertices(
        &mut self,
        pw: &mut WiresOnShape,
        map_v: &mut MapOfShape,
        map_e: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let mut the_subs = Substitution::new();
        let bb = BRepBuilder::default();
        pw.init_edge_iterator()?;
        while pw.more_edge() {
            let edg = pw.edge()?;
            map_e.add(edg.as_ref().clone());
            for v in Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let vtx = topods::vertex(&v)?;
                if map_v.contains(&v) {
                    continue;
                }
                let Some(vb) = pw.on_vertex(&vtx)? else {
                    continue;
                };
                map_v.add(v.clone());
                if vtx.is_same(&vb) {
                    continue;
                }
                let vsub = topods::vertex(&vtx.oriented(Orientation::Forward))?;
                let d = BRepTool::pnt(&vsub)?.distance(&BRepTool::pnt(&vb)?)
                    + BRepTool::vertex_tolerance(&vb)?;
                bb.update_vertex_tolerance(&vsub, d)?;
                the_subs.substitute(&vb.oriented(Orientation::Forward), &[vsub.into_shape()])?;
            }
            pw.next_edge();
        }
        the_subs.build(&self.shape)?;
        if the_subs.is_copied(&self.shape) {
            // on n`a fait que des substitutions de vertex. Donc chaque element
            // est remplace par lui meme ou par un seul element du meme type.
            for (key, value) in self.map.items_mut() {
                if the_subs.is_copied(key) {
                    let lsub = the_subs.copy(key)?;
                    value.clear();
                    if let Some(first) = lsub.first() {
                        value.append(first.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// Adds every vertex lying on an edge of the shape, and prepares the wires to
    /// rebuild on each face; the wire edges lying on shape edges in `edg_on_edg`.
    fn prepare_faces(
        &self,
        pw: &mut WiresOnShape,
        map_v: &mut MapOfShape,
        the_c_face: &mut SplitShape,
        edg_on_edg: &mut DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(IndexedDataMapOfShapeListOfShape, MapOfShape), Failure> {
        let mut map_fe = IndexedDataMapOfShapeListOfShape::default();
        let mut faces_with_section = MapOfShape::default();
        pw.init_edge_iterator()?;
        while pw.more_edge() {
            let edg = pw.edge()?;
            self.add_vertices_on_edges(pw, &edg, map_v, the_c_face)?;
            if let Some(ebis) = pw.on_edge()? {
                edg_on_edg.bind(edg.into_shape(), ebis.into_shape());
            } else {
                let fac = pw.on_face()?;
                if self.map.is_bound(&fac) {
                    let is_face_with_sec = pw.is_face_with_section(&fac);
                    let fac = self.first_of(&fac)?;
                    if is_face_with_sec {
                        faces_with_section.add(fac.clone());
                    }
                    if !map_fe.contains(&fac) {
                        map_fe.add(fac.clone(), List::default());
                    }
                    map_fe.change_from_key(&fac)?.append(edg.into_shape());
                }
            }
            pw.next_edge();
        }
        Ok((map_fe, faces_with_section))
    }

    /// The vertices of `edg` projected on edges of the shape, added to the descendants
    /// of those edges.
    fn add_vertices_on_edges(
        &self,
        pw: &WiresOnShape,
        edg: &Edge,
        map_v: &mut MapOfShape,
        the_c_face: &mut SplitShape,
    ) -> Result<(), Failure> {
        for v in Explorer::new(edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if map_v.contains(&v) {
                continue;
            }
            map_v.add(v.clone());
            let vtx = topods::vertex(&v)?;
            let (mut ed, mut prm) = (Edge::default(), 0.);
            if !pw.on_edge_of_vertex_from(&vtx, edg, &mut ed, &mut prm)? {
                continue;
            }
            // on devrait verifier que le vtx n`existe pas deja sur l`edge
            if !self.map.is_bound(&ed) {
                continue;
            }
            let ed = topods::edge(&self.first_of(&ed)?)?;
            the_c_face.add_vertex(&vtx, prm, &ed)?;
        }
        Ok(())
    }

    /// The single result: a shell of the pieces of a face, a wire of the pieces of an
    /// edge, or the single descendant; None when there are several of another type.
    fn single_result(&self) -> Result<Option<Shape>, Failure> {
        let lres =
            to_vec(self.map.seek(&self.shape).ok_or_else(|| {
                null_object("LocOpe_Spliter::Perform", "shape without descendant")
            })?);
        let typ_s = self.shape.shape_type()?;
        let b = BRepBuilder::default();
        if typ_s == ShapeEnum::Face && lres.len() >= 2 {
            let mut res = Shell::default();
            b.make_shell(&mut res);
            let mut res = res.into_shape();
            res.set_orientation(Orientation::Forward);
            for s in &lres {
                b.add(&mut res, &s.oriented(self.shape.orientation()))?;
            }
            return Ok(Some(res));
        }
        if typ_s == ShapeEnum::Edge && lres.len() >= 2 {
            let mut res = Wire::default();
            b.make_wire(&mut res);
            let mut res = res.into_shape();
            res.set_orientation(Orientation::Forward);
            for s in &lres {
                b.add(&mut res, &s.oriented(self.shape.orientation()))?;
            }
            return Ok(Some(res));
        }
        if lres.len() != 1 {
            return Ok(None);
        }
        Ok(lres.first().cloned())
    }

    /// The wire edges lying on shape edges replace their matching pieces.
    fn substitute_edges(
        &mut self,
        edg_on_edg: &DataMap<Shape, Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let mut the_subs = Substitution::new();
        for (key, value) in edg_on_edg.items() {
            let e1 = topods::edge(key)?;
            // on recherche dans les descendants de e2 l`edge qui correspont a e1
            let pieces = self
                .map
                .seek(value)
                .ok_or_else(|| null_object("LocOpe_Spliter::Perform", "edge without descendant"))?;
            let mut lsubs = matching_pieces(&e1, pieces)?;
            if lsubs.len() >= 2 {
                // il faut faire un choix
                select(&e1, &mut lsubs)?;
            }
            if lsubs.len() == 1 {
                let ebase = lsubs.remove(0);
                the_subs.substitute(&ebase, &[e1.oriented(ebase.orientation())])?;
            }
        }
        the_subs.build(&self.res)?;
        for (_, ldesc) in self.map.items_mut() {
            let mut newdesc = List::default();
            for s in ldesc.iter() {
                if the_subs.is_copied(s) {
                    if let Some(first) = the_subs.copy(s)?.first() {
                        newdesc.append(first.clone());
                    }
                } else {
                    newdesc.append(s.clone());
                }
            }
            *ldesc = newdesc;
        }
        if the_subs.is_copied(&self.res) {
            self.res = the_subs
                .copy(&self.res)?
                .first()
                .cloned()
                .unwrap_or_default();
        }
        Ok(())
    }

    /// The chains of degenerated edges sharing vertices.
    fn degenerated_wires(&self) -> Result<Vec<Shape>, Failure> {
        let bb = BRepBuilder::default();
        let mut emap = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.res, ShapeEnum::Edge, &mut emap)?;
        let mut deg_edges = Vec::new();
        for i in 1..=emap.extent() {
            let an_edge = topods::edge(emap.find_key(i)?)?;
            if BRepTool::degenerated(&an_edge)? {
                deg_edges.push(an_edge);
            }
        }
        let mut deg_wires = Vec::new();
        while !deg_edges.is_empty() {
            let mut a_deg_wire = Wire::default();
            bb.make_wire(&mut a_deg_wire);
            bb.add(&mut a_deg_wire, &deg_edges.remove(0))?;
            loop {
                let (vfirst, vlast) = topexp::wire_vertices(&a_deg_wire);
                let touching = deg_edges.iter().position(|e| {
                    let (v1, v2) = topexp::vertices(e, false);
                    v1.is_same(&vfirst)
                        || v1.is_same(&vlast)
                        || v2.is_same(&vfirst)
                        || v2.is_same(&vlast)
                });
                let Some(i) = touching else { break };
                bb.add(&mut a_deg_wire, &deg_edges.remove(i))?;
            }
            deg_wires.push(a_deg_wire.into_shape());
        }
        Ok(deg_wires)
    }

    /// Removes the superfluous vertices on degenerated edges: each chain of them keeps
    /// its first vertex.
    fn merge_degenerated_vertices(&mut self) -> Result<(), Failure> {
        let mut the_subs = Substitution::new();
        for w in self.degenerated_wires()? {
            let mut vmap = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(&w, ShapeEnum::Vertex, &mut vmap)?;
            let first = vmap.find_key(1)?.clone();
            let lv = [first.oriented(Orientation::Forward)];
            for j in 2..=vmap.extent() {
                if !vmap.find_key(j)?.is_same(&first) {
                    the_subs.substitute(vmap.find_key(j)?, &lv)?;
                }
            }
        }
        the_subs.build(&self.res)?;
        if the_subs.is_copied(&self.res) {
            self.res = the_subs
                .copy(&self.res)?
                .first()
                .cloned()
                .unwrap_or_default();
        }
        Ok(())
    }

    /// The faces of the result holding a wire edge with its orientation (direct left);
    /// the others are returned.
    fn direct_left_faces(&mut self, map_e: &MapOfShape) -> Result<Vec<Shape>, Failure> {
        let mut others = Vec::new();
        for fac in Explorer::new(&self.res, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut on_left = false;
            for edg in Explorer::new(&fac, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if map_e
                    .iter()
                    .any(|k| k.is_same(&edg) && edg.orientation() == k.orientation())
                {
                    on_left = true;
                    break;
                }
            }
            if on_left {
                self.d_left.append(fac.clone());
                self.left.append(fac);
            } else {
                others.push(fac);
            }
        }
        Ok(others)
    }

    /// Grows the left part across its border edges that are not wire edges.
    fn grow_left(&mut self, map_e: &MapOfShape, mut map_v: MapOfShape) -> Result<(), Failure> {
        // Map des edges ou les connexions sont possibles
        let mut mapebord = MapOfShape::default();
        for f in self.left.iter() {
            for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !map_e.contains(&e) && !mapebord.add(e.clone()) {
                    mapebord.remove(&e);
                }
            }
        }
        while mapebord.extent() != 0 {
            let edg = mapebord.iter().next().cloned().unwrap_or_default();
            let mut face_left = None;
            for fac in map_v.iter() {
                let mut edges = Explorer::new(fac, ShapeEnum::Edge, ShapeEnum::Shape)?;
                if edges.any(|e| e.is_same(&edg)) {
                    face_left = Some(fac.clone());
                    break; // face a gauche
                }
            }
            let Some(fac) = face_left else {
                mapebord.remove(&edg);
                continue;
            };
            for e in Explorer::new(&fac, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !mapebord.add(e.clone()) {
                    mapebord.remove(&e);
                }
            }
            map_v.remove(&fac);
            self.left.append(fac);
        }
        Ok(())
    }

    /// `Perform(PW)`: splits the shape by the wires of `pw`; Standard_NullObject without
    /// shape; `spliter.perform(&mut pw)?`.
    pub fn perform(&mut self, pw: &mut WiresOnShape) -> Result<(), Failure> {
        if self.shape.is_null() {
            return Err(null_object("LocOpe_Spliter::Perform", "no shape to split"));
        }
        self.done = false;
        self.map.clear(false);
        self.res.nullify();
        let shape = self.shape.clone();
        put(&shape, &mut self.map);
        let mut map_v = MapOfShape::default();
        let mut map_e = MapOfShape::default();
        self.substitute_vertices(pw, &mut map_v, &mut map_e)?;
        self.res = self.first_of(&shape)?;
        let mut the_c_face = SplitShape::with_shape(&self.res)?;
        let mut edg_on_edg = DataMap::default();
        let (mut map_fe, faces_with_section) =
            self.prepare_faces(pw, &mut map_v, &mut the_c_face, &mut edg_on_edg)?;
        // Rebuilds wires on each face of the shape
        for i in 1..=map_fe.extent() {
            let fac = topods::face(map_fe.find_key(i)?)?;
            let ledges = rebuild_wires(map_fe.find_from_index(i)?, pw)?;
            *map_fe.change_from_index(i)? = ledges.clone();
            if faces_with_section.contains(&fac) {
                the_c_face.add_wires(&ledges, &fac)?;
            } else {
                for w in ledges.iter() {
                    the_c_face.add_wire(&topods::wire(w)?, &fac)?;
                }
            }
        }
        // Mise a jour des descendants
        let keys: Vec<(Shape, Shape)> = self
            .map
            .items()
            .map(|(k, v)| (k.clone(), v.first().cloned().unwrap_or_default()))
            .collect();
        for (sori, scib) in keys {
            let desc = the_c_face.descendant_shapes(&scib)?.clone();
            if let Some(l) = self.map.change_seek(&sori) {
                *l = desc;
            }
        }
        let Some(res) = self.single_result()? else {
            return Ok(());
        };
        self.res = res;
        self.substitute_edges(&edg_on_edg)?;
        self.merge_degenerated_vertices()?;
        self.d_left.clear();
        self.left.clear();
        let others = self.direct_left_faces(&map_e)?;
        let map_v: MapOfShape = others.into_iter().collect_map();
        self.grow_left(&map_e, map_v)?;
        self.done = true;
        Ok(())
    }
}

/// Collects shapes into a TopTools map.
trait CollectMap {
    fn collect_map(self) -> MapOfShape;
}

impl<I: Iterator<Item = Shape>> CollectMap for I {
    fn collect_map(self) -> MapOfShape {
        let mut map = MapOfShape::default();
        for s in self {
            map.add(s);
        }
        map
    }
}
