// Rust translation of the steps of LocOpe_Spliter::Perform of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Spliter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps of the splitting: the vertices and edges of the wires substituted in the
//! shape, the faces split, the degenerated wires merged, and the faces on the left of
//! the wires grown across the edges that are not cut.
use super::super::split_shape::to_vec;
use super::super::{SplitShape, WiresOnShape};
use super::{Spliter, matching_pieces, select};
use crate::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Shell, Wire};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl Spliter {
    /// 1ere etape : substitution des vertex: each wire vertex bound to a shape vertex
    /// replaces it (its tolerance covering the distance); the descendants follow.
    pub(super) fn substitute_vertices(
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
    pub(super) fn prepare_faces(
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
    pub(super) fn add_vertices_on_edges(
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
    pub(super) fn single_result(&self) -> Result<Option<Shape>, Failure> {
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
    pub(super) fn substitute_edges(
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
    pub(super) fn degenerated_wires(&self) -> Result<Vec<Shape>, Failure> {
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
    pub(super) fn merge_degenerated_vertices(&mut self) -> Result<(), Failure> {
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
    pub(super) fn direct_left_faces(&mut self, map_e: &MapOfShape) -> Result<Vec<Shape>, Failure> {
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
    pub(super) fn grow_left(
        &mut self,
        map_e: &MapOfShape,
        mut map_v: MapOfShape,
    ) -> Result<(), Failure> {
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
}
