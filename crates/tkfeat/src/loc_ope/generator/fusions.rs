// Rust translation of the first part of LocOpe_Generator::Perform of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Generator.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces left of the generating edges, the faces fused with generated ones, the
//! edges fused with generated ones and the edges rebuilt on the fused vertices.
use super::Run;
use super::statics::{new_parameter, to_fuse_at, to_fuse_edges, to_fuse_faces};
use crate::errors::no_such_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// Appends `s` to the list bound to `key`, binding an empty list first.
pub(super) fn append_to(map: &mut super::ShapeLists, key: &Shape, s: Shape) {
    if !map.is_bound(key) {
        map.bind(key.clone(), List::default());
    }
    if let Some(l) = map.change_seek(key) {
        l.append(s);
    }
}

impl Run<'_> {
    /// The generating edges and vertices, and the faces of the shape holding a
    /// generating edge with its orientation (on genere une liste des faces a gauche du
    /// wire, equivalent du LeftOf).
    pub(super) fn collect_left(&mut self) -> Result<(), Failure> {
        let ledges: Vec<Shape> = self.g.generating_edges()?.iter().cloned().collect();
        for edg in &ledges {
            self.g_edg.add(edg.clone());
            for vtx in Explorer::new(edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                self.g_vtx.add(vtx);
            }
            for fac in Explorer::new(self.shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                let mut edges = Explorer::new(&fac, ShapeEnum::Edge, ShapeEnum::Shape)?;
                if edges.any(|e| e.is_same(edg) && e.orientation() == edg.orientation()) {
                    self.the_left.add(fac.clone());
                    if !self.mod_shapes.is_bound(&fac) {
                        self.mod_shapes.bind(fac, List::default());
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// The key of `the_ff_map` other than `fac` whose fused faces hold `facbis`.
    fn fused_with(&self, fac: &Shape, facbis: &Shape) -> Option<Shape> {
        self.the_ff_map
            .items()
            .filter(|(key, _)| !key.is_same(fac))
            .find(|(_, faces)| faces.iter().any(|x| x.is_same(facbis)))
            .map(|(key, _)| key.clone())
    }

    /// Search for face fusions: the face right of each generating edge fuses with the
    /// face the edge generates when they are coplanar.
    pub(super) fn face_fusions(&mut self) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
        let mut the_ef_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            self.shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut the_ef_map,
        )?;
        let g_edg: Vec<Shape> = self.g_edg.iter().cloned().collect();
        for e in &g_edg {
            let edg = topods::edge(e)?;
            let Some(faces) = the_ef_map.seek(e) else {
                continue;
            };
            // edge "interne" au shell, ou bord libre when every face is on the left
            let Some(fac) = faces.iter().find(|x| !self.the_left.contains(x)).cloned() else {
                continue;
            };
            let facbis = self.g.generated_by_edge(&edg)?.into_shape();
            if !to_fuse_faces(&topods::face(&fac)?, &topods::face(&facbis)?)? {
                // face generee par edg : on la marque.
                continue;
            }
            // On recherche si une face a deja fusionne avec facbis
            if let Some(key) = self.fused_with(&fac, &facbis) {
                append_to(&mut self.the_ff_map, &key, fac.clone());
                self.to_remove.add(fac);
                self.to_remove.add(e.clone());
                continue;
            }
            if !self
                .the_ff_map
                .seek(&fac)
                .is_some_and(|l| l.iter().any(|x| x.is_same(&facbis)))
            {
                append_to(&mut self.the_ff_map, &fac, facbis.clone());
            }
            self.to_remove.add(e.clone());
            self.to_remove.add(facbis);
        }
        Ok(the_ef_map)
    }

    /// Adds to `to_remove` the connexity edges between faces to fuse with the same base
    /// face.
    pub(super) fn connexity_edges(&mut self) -> Result<(), Failure> {
        let groups: Vec<Vec<Shape>> = self
            .the_ff_map
            .values()
            .map(|l| l.iter().cloned().collect())
            .collect();
        for faces in &groups {
            for face in faces {
                for ed in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if self.to_remove.contains(&ed) {
                        continue;
                    }
                    for other in faces.iter().filter(|x| !x.is_same(face)) {
                        let mut edges = Explorer::new(other, ShapeEnum::Edge, ShapeEnum::Shape)?;
                        if edges.any(|x| ed.is_same(&x)) {
                            self.to_remove.add(ed.clone());
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The orientation of the face `fac` in which the edge `edg`, closed on it, must
    /// not fuse at `vtx`, from the other edge of the face at `vtx` that is kept.
    fn closed_edge_dont_fuse(
        &mut self,
        edg: &Edge,
        fac: &Face,
        vtx: &Vertex,
    ) -> Result<bool, Failure> {
        let (_, vl) = topexp::vertices(edg, false);
        let connect_last = vl.is_same(vtx);
        for x in Explorer::new(
            &fac.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            if x.is_same(edg) {
                continue;
            }
            let eee = topods::edge(&x)?;
            let orient = eee.orientation();
            let (vf, vl) = topexp::vertices(&eee, false);
            if !((vf.is_same(vtx) || vl.is_same(vtx)) && !self.to_remove.contains(&x)) {
                continue;
            }
            // On recherche celui qu`il ne faut pas fusionner
            let leaving = (vf.is_same(vtx) && orient == Orientation::Forward)
                || (vl.is_same(vtx) && orient == Orientation::Reversed);
            let forward = leaving == connect_last;
            let or = if forward {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            self.dont_fuse.bind(edg.as_ref().clone(), fac.oriented(or));
            return Ok(false);
        }
        Ok(true)
    }

    /// The fusion at `vtx` of the edge `edg` closed on `fac` with the edge `edgbis` it
    /// generates (a voir).
    fn fuse_closed_edge(
        &mut self,
        edg: &Edge,
        fac: &Face,
        vtx: &Vertex,
        edgbis: &Edge,
    ) -> Result<(), Failure> {
        if self.the_ee_map.is_bound(edg) {
            return Ok(());
        }
        append_to(&mut self.the_ee_map, edg, edgbis.as_ref().clone());
        self.to_remove.add(edgbis.as_ref().clone()); // toujours vrai pour edge double
        if self.closed_edge_dont_fuse(edg, fac, vtx)? {
            self.to_remove.add(vtx.as_ref().clone());
        }
        Ok(())
    }

    /// The fusion at `vtx` of the edge `edg` of `fac` with the edge `edgbis` it
    /// generates: the vertex goes when no kept edge of the faces of `edg` meets it.
    fn fuse_open_edge(
        &mut self,
        edg: &Edge,
        fac: &Face,
        vtx: &Vertex,
        edgbis: &Edge,
        the_ef_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<Shape>, Failure> {
        // Native quirk kept: edgbis is appended before the check of its presence,
        // which then always finds it.
        append_to(&mut self.the_ee_map, edg, edgbis.as_ref().clone());
        let faces = the_ef_map.find_from_key(edg)?.clone();
        if !to_fuse_at(edg, fac, vtx, &self.to_remove)? {
            self.dont_fuse
                .bind(edg.as_ref().clone(), fac.as_ref().clone());
            return Ok(None);
        }
        let Some(other) = faces.iter().find(|x| !x.is_same(fac)).cloned() else {
            return Ok(None);
        };
        if !self.the_ff_map.is_bound(&other) {
            // on marque comme face a reconstruire
            self.to_remove.remove(vtx);
            self.dont_fuse.bind(edg.as_ref().clone(), other.clone());
            return Ok(Some(other));
        }
        // edge a fusionner
        if to_fuse_at(edg, &topods::face(&other)?, vtx, &self.to_remove)? {
            self.to_remove.add(vtx.as_ref().clone());
        } else {
            self.to_remove.remove(vtx);
            self.dont_fuse.bind(edg.as_ref().clone(), other);
        }
        Ok(None)
    }

    /// The fusions of the edges of the fused faces with the edges their generating
    /// vertices generate (coaxial lines); the faces to rebuild are returned.
    pub(super) fn edge_fusions(
        &mut self,
        the_ef_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Vec<Shape>, Failure> {
        let mut rebuild_face = Vec::new();
        let mut map_treated = MapOfShape::default();
        let keys: Vec<Shape> = self.the_ff_map.items().map(|(k, _)| k.clone()).collect();
        for key in &keys {
            let fac = topods::face(key)?;
            for e in Explorer::new(&fac, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !map_treated.add(e.clone()) {
                    continue; // on saute l`edge
                }
                let edg = topods::edge(&e)?;
                for v in Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                    if !self.g_vtx.contains(&v) {
                        continue;
                    }
                    let vtx = topods::vertex(&v)?;
                    let edgbis = self.g.generated_by_vertex(&vtx)?;
                    if edgbis.is_null()
                        || BRepTool::degenerated(&edgbis)?
                        || !to_fuse_edges(&edg, &edgbis)?
                    {
                        continue;
                    }
                    if brep_tools::is_really_closed(&edg, &fac)? {
                        self.fuse_closed_edge(&edg, &fac, &vtx, &edgbis)?;
                    } else if let Some(face) =
                        self.fuse_open_edge(&edg, &fac, &vtx, &edgbis, the_ef_map)?
                    {
                        rebuild_face.push(face);
                    }
                }
            }
        }
        Ok(rebuild_face)
    }

    /// The vertex of an edge fused with `edg` other than `vtx`, with the edge, when an
    /// edge fused with `edg` holds `vtx`.
    fn fused_vertex(
        &self,
        edg: &Shape,
        vtx: &Shape,
    ) -> Result<Option<(Shape, Option<Vertex>)>, Failure> {
        let fused = self.the_ee_map.find(edg)?;
        for edgbis in fused.iter() {
            let mut vertices = Explorer::new(edgbis, ShapeEnum::Vertex, ShapeEnum::Shape)?;
            if !vertices.any(|x| x.is_same(vtx)) {
                continue;
            }
            let other = Explorer::new(edgbis, ShapeEnum::Vertex, ShapeEnum::Shape)?
                .find(|x| !x.is_same(vtx));
            let other = match other {
                Some(x) => Some(topods::vertex(&x)?),
                None => None,
            };
            return Ok(Some((edgbis.clone(), other)));
        }
        Ok(None)
    }

    /// Fusion des edges: each edge fused with others is rebuilt with, at each fused
    /// vertex (or vertex inside its range), the far vertex of the fused edge.
    pub(super) fn fuse_edges(&mut self) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let keys: Vec<Shape> = self.the_ee_map.items().map(|(k, _)| k.clone()).collect();
        for key in &keys {
            let edg = topods::edge(key)?;
            let (f, l) = BRepTool::range(&edg)?;
            let mut newedg = topods::edge(&edg.empty_copied()?)?;
            newedg.set_orientation(Orientation::Forward);
            let mut keep_new_edge = false;
            for v in Explorer::new(
                &edg.oriented(Orientation::Forward),
                ShapeEnum::Vertex,
                ShapeEnum::Shape,
            )? {
                let vtx = topods::vertex(&v)?;
                let mut prm = BRepTool::parameter(&vtx, &edg)?;
                let fused = self.fused_vertex(key, &v)?;
                let mut newvtx = None;
                if let Some((_, Some(other))) = &fused {
                    prm = new_parameter(&edg, &vtx, &newedg, other)?;
                    newvtx = Some(other.clone());
                }
                if self.to_remove.contains(&v) || (prm < l && prm > f) {
                    let newvtx = newvtx.ok_or_else(|| {
                        no_such_object(
                            "LocOpe_Generator::Perform",
                            "fused vertex without its far vertex",
                        )
                    })?;
                    b.add(&mut newedg, &newvtx.oriented(vtx.orientation()))?;
                    b.update_vertex_on_edge(
                        &newvtx,
                        prm,
                        &newedg,
                        BRepTool::vertex_tolerance(&newvtx)?,
                    )?;
                    let (edgbis, _) = fused.ok_or_else(|| {
                        no_such_object("LocOpe_Generator::Perform", "no fused edge at the vertex")
                    })?;
                    self.to_remove.add(edgbis); // i-e edgbis
                    keep_new_edge = true;
                } else {
                    b.add(&mut newedg, &vtx.oriented(vtx.orientation()))?;
                    b.update_vertex_on_edge(&vtx, prm, &newedg, BRepTool::vertex_tolerance(&vtx)?)?;
                }
            }
            if keep_new_edge {
                append_to(&mut self.mod_shapes, key, newedg.into_shape());
                self.to_remove.add(key.clone());
            }
        }
        Ok(())
    }
}
