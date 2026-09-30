// Rust translation of BRepBuilderAPI_Sewing::FaceAnalysis (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analysis of the input faces: edges smaller than the minimal tolerance become
//! degenerated (their vertices glued into one), and faces made only of such edges are
//! removed.
use super::{BRepBuilderAPISewing, live_reshape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Xyz};

/// The number of samples of the compactness test.
const NB_COMPACTNESS_SAMPLES: i32 = 5;

/// The state shared by the faces of the analysis.
struct Analysis<'r> {
    reshape: &'r dyn ReShape,
    small_edges: MapOfShape,
    glued_vertices: IndexedDataMapOfShapeListOfShape,
}

/// The counts of one face and whether it changed.
#[derive(Default)]
struct FaceCounts {
    nb_edges: i32,
    nb_small: i32,
    changed: bool,
}

/// Whether the 3d curve of an edge stays within half the minimal tolerance of the
/// middle of its ends (5 samples); false without 3d curve.
fn is_compact(edge: &Edge, min_tolerance: f64) -> Result<bool, Failure> {
    let (c3d, first, last) = BRepTool::curve(edge)?;
    let Some(c) = c3d.get() else {
        return Ok(false);
    };
    let middle: Xyz = (c.value(first)?.xyz() + c.value(last)?.xyz()) * 0.5;
    let cp = Point3::new(middle.x(), middle.y(), middle.z());
    let mut maxdist: f64 = 0.0;
    let delta: f64 = (last - first) / f64::from(NB_COMPACTNESS_SAMPLES - 1);
    for idx in 0..NB_COMPACTNESS_SAMPLES {
        let dist = cp.distance(&c.value(first + f64::from(idx) * delta)?);
        if maxdist < dist {
            maxdist = dist;
        }
    }
    Ok(2. * maxdist <= min_tolerance)
}

impl Analysis<'_> {
    /// Glues the vertices of a small edge: into the group of the first when it is
    /// glued (merging the group of the second), into the group of the second, or into
    /// a new vertex when both are free and distinct.
    fn glue_small_edge_vertices(&mut self, edge: &Edge) -> Result<(), Failure> {
        let (v1, v2) = topexp::vertices(edge, false);
        let nv1 = self.reshape.apply(&v1, ShapeEnum::Shape)?;
        let nv2 = self.reshape.apply(&v2, ShapeEnum::Shape)?;
        if !nv1.is_same(&v1) {
            // First vertex was already glued: its group must exist.
            self.glued_vertices.change_from_key(&nv1)?;
            if !nv2.is_same(&v2) {
                if nv1.is_same(&nv2) {
                    return Ok(());
                }
                // Merge lists of glued vertices.
                let moved: Vec<Shape> = self
                    .glued_vertices
                    .find_from_key(&nv2)?
                    .iter()
                    .cloned()
                    .collect();
                for v in &moved {
                    self.reshape.replace(v, &nv1.oriented(v.orientation()))?;
                    self.glued_vertices.change_from_key(&nv1)?.append(v.clone());
                }
                self.glued_vertices.remove_key(&nv2);
                return Ok(());
            }
            self.glued_vertices
                .change_from_key(&nv1)?
                .append(v2.clone().into_shape());
            return self.reshape.replace(&v2, &nv1.oriented(v2.orientation()));
        }
        if !nv2.is_same(&v2) {
            self.glued_vertices
                .change_from_key(&nv2)?
                .append(v1.clone().into_shape());
            return self.reshape.replace(&v1, &nv2.oriented(v1.orientation()));
        }
        if v1.is_same(&v2) {
            return Ok(());
        }
        let mut nv = Vertex::default();
        BRepBuilder::default().make_vertex(&mut nv);
        let mut vlist = List::default();
        vlist.append(v1.clone().into_shape());
        vlist.append(v2.clone().into_shape());
        self.glued_vertices.add(nv.clone().into_shape(), vlist);
        self.reshape.replace(&v1, &nv.oriented(v1.orientation()))?;
        self.reshape.replace(&v2, &nv.oriented(v2.orientation()))
    }

    /// The degenerated edge that replaces a small edge (its p-curve on the face, the
    /// glued vertices), added to the new wire; nothing when the edge has no p-curve.
    fn degenerated_replacement(
        &self,
        edge: &Edge,
        face: &Face,
        nwire: &mut Shape,
    ) -> Result<Option<Edge>, Failure> {
        let fedge = Edge::unchecked(edge.oriented(Orientation::Forward));
        let found = BRepTool::curve_on_face(&fedge, face)?;
        if found.pcurve.is_null() {
            return Ok(None);
        }
        let b = BRepBuilder::default();
        let mut nedge = Edge::default();
        b.make_edge(&mut nedge)?;
        b.update_edge_pcurve_on_face(&nedge, &found.pcurve, face, precision::confusion())?;
        b.range(&nedge, found.first, found.last, false)?;
        b.degenerated(&nedge, true)?;
        let (v1, v2) = topexp::vertices(&fedge, false);
        b.add(
            &mut nedge,
            &self
                .reshape
                .apply(&v1, ShapeEnum::Shape)?
                .oriented(v1.orientation()),
        )?;
        b.add(
            &mut nedge,
            &self
                .reshape
                .apply(&v2, ShapeEnum::Shape)?
                .oriented(v2.orientation()),
        )?;
        b.add(nwire, &nedge.oriented(edge.orientation()))?;
        Ok(Some(nedge))
    }
}

impl BRepBuilderAPISewing {
    /// `FaceAnalysis(theProgress)`: the context shape becomes the input when none was
    /// added; in each face, a degenerated edge is kept, an edge within the minimal
    /// tolerance becomes degenerated with its vertices glued; a face whose edges are all
    /// such is removed, a changed face replaced. The glued vertices get the mean point
    /// and a tolerance covering them, and the inputs their new images.
    pub(crate) fn face_analysis(&mut self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        if !self.my_shape.is_null() && self.my_old_shapes.is_empty() {
            let shape = self.my_shape.clone();
            self.add(&shape)?;
            self.my_shape.nullify();
        }
        let reshape_handle = self.my_re_shape.clone();
        let mut analysis = Analysis {
            reshape: live_reshape(&reshape_handle)?,
            small_edges: MapOfShape::default(),
            glued_vertices: IndexedDataMapOfShapeListOfShape::default(),
        };
        let scope = ProgressScope::with_literal(
            progress,
            "Shape analysis",
            self.my_old_shapes.extent() as f64,
            false,
        );
        let mut i = 1;
        while i <= self.my_old_shapes.extent() && scope.more() {
            let shape = self.my_old_shapes.find_from_index(i)?.clone();
            for face in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
                self.analyse_face(&mut analysis, &topods::face(&face)?)?;
            }
            i += 1;
            scope.next(1.0);
        }
        update_glued_vertices(&analysis.glued_vertices)?;
        for i in 1..=self.my_old_shapes.extent() {
            let image = analysis
                .reshape
                .apply(self.my_old_shapes.find_from_index(i)?, ShapeEnum::Shape)?;
            *self.my_old_shapes.change_from_index(i)? = image;
        }
        Ok(())
    }

    /// One face: its wires rebuilt with the small edges degenerated, then the face
    /// removed or replaced.
    fn analyse_face(&mut self, analysis: &mut Analysis<'_>, face: &Face) -> Result<(), Failure> {
        let mut nface = face.empty_copied()?.oriented(Orientation::Forward);
        let mut counts = FaceCounts::default();
        let b = BRepBuilder::default();
        let forward_face = face.oriented(Orientation::Forward);
        let mut witer = ShapeIterator::new(&forward_face, true, true);
        while let Ok(child) = witer.value() {
            let child = child.clone();
            witer.next();
            if child.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            let wire = topods::wire(&child)?;
            let (nwire, wire_changed) = self.analyse_wire(analysis, face, &wire, &mut counts)?;
            if wire_changed {
                b.add(&mut nface, &nwire.oriented(wire.orientation()))?;
                counts.changed = true;
            } else {
                b.add(&mut nface, &wire)?;
            }
        }
        if counts.nb_small == counts.nb_edges {
            self.my_little_face.add(face.clone().into_shape());
            return analysis.reshape.remove(face);
        }
        if counts.changed {
            analysis
                .reshape
                .replace(face, &nface.oriented(face.orientation()))?;
        }
        Ok(())
    }

    /// One wire: its edges copied into a new wire, the small ones replaced.
    fn analyse_wire(
        &mut self,
        analysis: &mut Analysis<'_>,
        face: &Face,
        wire: &Wire,
        counts: &mut FaceCounts,
    ) -> Result<(Shape, bool), Failure> {
        let b = BRepBuilder::default();
        let mut nwire = wire.empty_copied()?.oriented(Orientation::Forward);
        let mut wire_changed = false;
        let forward_wire = wire.oriented(Orientation::Forward);
        let mut eiter = ShapeIterator::new(&forward_wire, true, true);
        while let Ok(child) = eiter.value() {
            let edge = topods::edge(child)?;
            eiter.next();
            counts.nb_edges += 1;
            if BRepTool::degenerated(&edge)? {
                // Old edge kept.
                b.add(&mut nwire, &edge)?;
                self.my_degenerated.add(edge.into_shape());
                counts.nb_small += 1;
                continue;
            }
            let mut is_small = analysis.small_edges.contains(&edge);
            if !is_small {
                is_small = is_compact(&edge, self.min_tolerance())?;
                if is_small {
                    analysis.small_edges.add(edge.clone().into_shape());
                    analysis.glue_small_edge_vertices(&edge)?;
                }
            }
            if !is_small {
                // Old edge kept.
                b.add(&mut nwire, &edge)?;
                continue;
            }
            counts.nb_small += 1;
            if let Some(nedge) = analysis.degenerated_replacement(&edge, face, &mut nwire)? {
                self.my_degenerated.add(nedge.into_shape());
            }
            wire_changed = true;
        }
        Ok((nwire, wire_changed))
    }
}

/// Each glued vertex takes the mean point of its vertices and a tolerance covering
/// them (the largest distance plus the largest tolerance).
fn update_glued_vertices(glued: &IndexedDataMapOfShapeListOfShape) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    for i in 1..=glued.extent() {
        let vnew = topods::vertex(glued.find_key(i)?)?;
        let vlist = glued.find_from_index(i)?;
        let mut coord = Xyz::new(0., 0., 0.);
        let mut nb_points: i32 = 0;
        for v in vlist.iter() {
            coord += BRepTool::pnt(&topods::vertex(v)?)?.xyz();
            nb_points += 1;
        }
        if nb_points == 0 {
            continue;
        }
        let mean = coord / f64::from(nb_points);
        let vp = Point3::new(mean.x(), mean.y(), mean.z());
        let (mut tol, mut mtol): (f64, f64) = (0.0, 0.0);
        for v in vlist.iter() {
            let vertex = topods::vertex(v)?;
            let mut vtol = BRepTool::vertex_tolerance(&vertex)?;
            if mtol < vtol {
                mtol = vtol;
            }
            vtol = vp.distance(&BRepTool::pnt(&vertex)?);
            if tol < vtol {
                tol = vtol;
            }
        }
        b.update_vertex_point(&vnew, vp, tol + mtol)?;
    }
    Ok(())
}
