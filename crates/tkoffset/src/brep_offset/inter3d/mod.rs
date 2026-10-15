// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter3d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_Inter3d`: the 3d intersections of offset faces (by pairs of near faces,
//! of faces adjacent through edges or vertices, of faces with the context faces),
//! stored in the shared AsDes as the new edges of each face.
mod connex;
mod context;

use super::{BRepOffsetAnalyse, BRepOffsetOffset, BRepOffsetTool};
use rustycad_tkbo::bop_tools::{BoxSet, PairSelector};
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The offset of each initial face (`MapSF`).
pub type OffsetFaces = DataMap<Shape, BRepOffsetOffset, ShapeMapHasher>;
/// A map of shapes to shapes (`MES`, `Build`).
pub type ShapeShapeMap = DataMap<Shape, Shape, ShapeMapHasher>;

/// `BRepOffset_Inter3d`: the intersections of offset faces stored in a shared AsDes;
/// `BRepOffsetInter3d::new(&as_des, State::Out, 1e-7)`.
pub struct BRepOffsetInter3d {
    my_as_des: Handle<BRepAlgoAsDes>,
    my_touched: IndexedMapOfShape,
    my_done: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    my_new_edges: IndexedMapOfShape,
    my_side: State,
    my_tol: f64,
}

/// `ExtentEdge(F, E, NE)`: an empty copy of the edge over a range a hundred times its
/// length on each side, with new vertices at the ends.
fn extent_edge(e: &Edge) -> Result<Edge, Failure> {
    let mut ne = topods::edge(&e.empty_copied()?)?;
    // Enough for analytic edges; in general the geometry of the edge would be rebuilt
    // from the intersection of the surfaces.
    ne.set_orientation(Orientation::Forward);
    let (mut f, mut l) = BRepTool::range(e)?;
    let length = l - f;
    f -= 100. * length;
    l += 100. * length;
    let builder = BRepBuilder::default();
    builder.range(&ne, f, l, false)?;
    let ce = BRepAdaptorCurve::with_edge(e)?;
    let v1 = BRepLibMakeVertex::new(ce.value(f)?)?.vertex()?;
    let v2 = BRepLibMakeVertex::new(ce.value(l)?)?.vertex()?;
    let mut ne_shape = ne.as_ref().clone();
    builder.add(&mut ne_shape, &v1.oriented(Orientation::Forward))?;
    builder.add(&mut ne_shape, &v2.oriented(Orientation::Reversed))?;
    let mut ne = topods::edge(&ne_shape)?;
    ne.set_orientation(e.orientation());
    Ok(ne)
}

/// The first image of a shape (`Image(S).First()`).
fn first_image(image: &BRepAlgoImage, s: &Shape) -> Result<Shape, Failure> {
    image
        .image(s)
        .first()
        .cloned()
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_List::First"))
}

/// The first interval type of an edge, `None` without intervals.
fn first_type(analyse: &BRepOffsetAnalyse, e: &Edge) -> Result<Option<TypeOfConcavity>, Failure> {
    Ok(match analyse.type_of(e)?.first() {
        Some(interval) => Some(interval.get_type()?),
        None => None,
    })
}

impl BRepOffsetInter3d {
    /// `BRepOffset_Inter3d(AsDes, Side, Tol)`; `BRepOffsetInter3d::new(&as_des, State::Out, 1e-7)`.
    pub fn new(as_des: &Handle<BRepAlgoAsDes>, side: State, tol: f64) -> Self {
        Self {
            my_as_des: as_des.clone(),
            my_touched: IndexedMapOfShape::default(),
            my_done: DataMap::default(),
            my_new_edges: IndexedMapOfShape::default(),
            my_side: side,
            my_tol: tol,
        }
    }

    fn as_des(&self) -> &BRepAlgoAsDes {
        self.my_as_des
            .get()
            .expect("BRepOffset_Inter3d: null AsDes")
    }

    /// `CompletInt(SetOfFaces, InitOffsetFace, Range)`: `FaceInter` of each pair of faces
    /// whose bounding boxes interfere (in the sorted pair order of the box tree);
    /// `inter.complet_int(&faces, &image, &ProgressRange::default())?`.
    pub fn complet_int(
        &mut self,
        set_of_faces: &List<Shape>,
        init_offset_face: &BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        // The bounding boxes of the faces in a box tree.
        let mut a_bb_tree = BoxSet::default();
        a_bb_tree.set_size(set_of_faces.len());
        let mut a_m_faces = IndexedMapOfShape::default();
        for s in set_of_faces.iter() {
            let a_f = topods::face(s)?;
            let mut a_box_f = BoundingBox3::default();
            BRepBndLib::add(a_f.as_ref(), &mut a_box_f, true)?;
            let i = a_m_faces.add(a_f.as_ref().clone());
            a_bb_tree.add(i as i32, box3_to_bvh(&a_box_f)?);
        }
        a_bb_tree.build()?;
        // The pairs.
        let pairs: Vec<(i32, i32)> = {
            let mut a_selector = PairSelector::default();
            a_selector.set_shared_bvh_set(&mut a_bb_tree);
            a_selector.set_same(true);
            a_selector.select()?;
            a_selector.sort();
            a_selector.pairs().iter().map(|p| (p.id1, p.id2)).collect()
        };
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Complete intersection",
            pairs.len() as f64,
            false,
        );
        for (id1, id2) in pairs {
            if !a_ps.more() {
                return Ok(());
            }
            let a_f1 = topods::face(a_m_faces.find_key(id1.min(id2) as usize)?)?;
            let a_f2 = topods::face(a_m_faces.find_key(id1.max(id2) as usize)?)?;
            self.face_inter(&a_f1, &a_f2, init_offset_face)?;
            let _ = a_ps.next(1.);
        }
        Ok(())
    }

    /// `FaceInter(F1, F2, InitOffsetFace)`: the intersection of two offset faces of
    /// different initial shapes (tubes by PipeInter; faces sharing only vertices only
    /// when their initial faces share an edge or nothing); stored, the pair done;
    /// `inter.face_inter(&f1, &f2, &image)?`.
    pub fn face_inter(
        &mut self,
        f1: &Face,
        f2: &Face,
        init_offset_face: &BRepAlgoImage,
    ) -> Result<(), Failure> {
        let (mut l_int1, mut l_int2) = (List::default(), List::default());
        let null_edge = Edge::default();
        let null_face = Face::default();
        if f1.is_same(f2) || self.is_done(f1, f2) {
            return Ok(());
        }
        let init_f1 = init_offset_face.image_from(f1.as_ref())?;
        let init_f2 = init_offset_face.image_from(f2.as_ref())?;
        if init_f1.is_same(&init_f2) {
            return Ok(());
        }
        let inter_pipes =
            init_f2.shape_type()? == ShapeEnum::Edge && init_f1.shape_type()? == ShapeEnum::Edge;
        let inter_faces =
            init_f1.shape_type()? == ShapeEnum::Face && init_f2.shape_type()? == ShapeEnum::Face;
        let (mut le, mut lv) = (List::default(), List::default());
        let side = self.my_side;
        if BRepOffsetTool::find_common_shapes_of_faces(f1, f2, &mut le, &mut lv)?
            || self
                .as_des()
                .has_common_descendant(f1.as_ref(), f2.as_ref(), &mut le)
        {
            // F1 and F2 share shapes.
            if le.is_empty() && !lv.is_empty() {
                if inter_pipes {
                    // The tubes share a vertex.
                    let (ve1a, ve1b) = topexp::vertices(&topods::edge(&init_f1)?, false);
                    let (ve2a, ve2b) = topexp::vertices(&topods::edge(&init_f2)?, false);
                    let mut v = Shape::default();
                    for a in [&ve1a, &ve1b] {
                        for b in [&ve2a, &ve2b] {
                            if a.is_same(b) {
                                v = a.as_ref().clone();
                            }
                        }
                    }
                    if !init_offset_face.has_image(&v) {
                        // No sphere.
                        BRepOffsetTool::pipe_inter(f1, f2, &mut l_int1, &mut l_int2, side)?;
                    }
                } else if inter_faces {
                    // Faces having only common vertices and supports having common
                    // edges: insufficient, but a larger criterion shakes too many
                    // sections.
                    let (if1, if2) = (topods::face(&init_f1)?, topods::face(&init_f2)?);
                    if BRepOffsetTool::find_common_shapes_of_faces(&if1, &if2, &mut le, &mut lv)? {
                        if !le.is_empty() {
                            BRepOffsetTool::inter3d(
                                f1,
                                f2,
                                &mut l_int1,
                                &mut l_int2,
                                side,
                                &null_edge,
                                &null_face,
                                &null_face,
                            )?;
                        }
                    } else {
                        BRepOffsetTool::inter3d(
                            f1,
                            f2,
                            &mut l_int1,
                            &mut l_int2,
                            side,
                            &null_edge,
                            &null_face,
                            &null_face,
                        )?;
                    }
                }
            }
        } else if inter_pipes {
            BRepOffsetTool::pipe_inter(f1, f2, &mut l_int1, &mut l_int2, side)?;
        } else {
            BRepOffsetTool::inter3d(
                f1,
                f2,
                &mut l_int1,
                &mut l_int2,
                side,
                &null_edge,
                &null_face,
                &null_face,
            )?;
        }
        self.store(f1, f2, &l_int1, &l_int2);
        Ok(())
    }

    /// `ConnexIntByArc(SetOfFaces, ShapeInit, Analyse, InitOffsetFace, Range)`: the
    /// intersections of the offset faces across the edges of the offset side (concave
    /// inward, convex outward), then of the tubes at the vertices without sphere with
    /// the other tubes and with the faces of the other edges;
    /// `inter.connex_int_by_arc(&faces, &shape, &analyse, &image, &ProgressRange::default())?`.
    pub fn connex_int_by_arc(
        &mut self,
        _set_of_faces: &List<Shape>,
        shape_init: &Shape,
        analyse: &BRepOffsetAnalyse,
        init_offset_face: &BRepAlgoImage,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        let ot = if self.my_side == State::Out {
            TypeOfConcavity::Convex
        } else {
            TypeOfConcavity::Concave
        };
        let (mut l_int1, mut l_int2) = (List::default(), List::default());
        let null_edge = Edge::default();
        let null_face = Face::default();
        let side = self.my_side;
        let a_ps_outer = ProgressScope::unnamed(the_range, 2., false);
        // Step 1: the faces of the initial faces across an edge of the proper type.
        {
            let a_ps_int_f = ProgressScope::with_literal(
                &a_ps_outer.next(1.),
                "Intersecting offset faces",
                1.,
                true,
            );
            for es in Explorer::new(shape_init, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !a_ps_int_f.more() {
                    return Ok(());
                }
                let e = topods::edge(&es)?;
                if first_type(analyse, &e)? == Some(ot) {
                    let anc = analyse.ancestors(&es)?;
                    if anc.len() == 2 {
                        let init_f1 = topods::face(&anc[0])?;
                        let init_f2 = topods::face(&anc[anc.len() - 1])?;
                        let f1 = topods::face(&first_image(init_offset_face, init_f1.as_ref())?)?;
                        let f2 = topods::face(&first_image(init_offset_face, init_f2.as_ref())?)?;
                        if !self.is_done(&f1, &f2) {
                            BRepOffsetTool::inter3d(
                                &f1,
                                &f2,
                                &mut l_int1,
                                &mut l_int2,
                                side,
                                &e,
                                &init_f1,
                                &init_f2,
                            )?;
                            self.store(&f1, &f2, &l_int1, &l_int2);
                        }
                    }
                }
                let _ = a_ps_int_f.next(1.);
            }
        }
        // Step 2: the tubes sharing a vertex without sphere with the tubes of the other
        // edges at the vertex and with the faces of the edges without tube.
        let a_ps_int_t =
            ProgressScope::with_literal(&a_ps_outer.next(1.), "Intersecting tubes", 1., true);
        for e1s in Explorer::new(shape_init, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !a_ps_int_t.more() {
                return Ok(());
            }
            if init_offset_face.has_image(&e1s) {
                // E1 generated a tube.
                let e1 = topods::edge(&e1s)?;
                let f1 = topods::face(&first_image(init_offset_face, &e1s)?)?;
                let (va, vb) = topexp::vertices(&e1, false);
                let anc_e1 = analyse.ancestors(&e1s)?.to_vec();
                for v in [&va, &vb] {
                    if init_offset_face.has_image(v.as_ref()) {
                        continue;
                    }
                    // The vertex has no sphere.
                    let anc = analyse.ancestors(v.as_ref())?.to_vec();
                    let mut tang_on_v = Vec::new();
                    analyse.tangent_edges(&e1, v, &mut tang_on_v)?;
                    let mtev: Vec<Shape> = tang_on_v;
                    for e2s in &anc {
                        let e2 = topods::edge(e2s)?;
                        let mut is_to_skip = false;
                        if !e1.is_same(&e2) {
                            let t2 = first_type(analyse, &e2)?;
                            is_to_skip = mtev.iter().any(|t| t.is_same(e2s))
                                && (t2.is_none() || t2 != Some(ot));
                        }
                        if e1.is_same(&e2) || is_to_skip {
                            continue;
                        }
                        if init_offset_face.has_image(e2s) {
                            // E2 generated a tube: tube/tube intersection when the
                            // edges are not tangent.
                            let f2 = topods::face(&first_image(init_offset_face, e2s)?)?;
                            if !self.is_done(&f1, &f2) {
                                BRepOffsetTool::pipe_inter(
                                    &f1,
                                    &f2,
                                    &mut l_int1,
                                    &mut l_int2,
                                    side,
                                )?;
                                self.store(&f1, &f2, &l_int1, &l_int2);
                            }
                        } else {
                            // The tube of E1 with the faces of E2 unless they are
                            // tangent to the tube or E2 is a tangent edge.
                            if first_type(analyse, &e2)? == Some(TypeOfConcavity::Tangential) {
                                continue;
                            }
                            let anc_e2 = analyse.ancestors(e2s)?.to_vec();
                            if anc_e2.len() == 2 {
                                for init_f2 in [&anc_e2[0], &anc_e2[1]] {
                                    let tangent_faces = init_f2.is_same(&anc_e1[0])
                                        || init_f2.is_same(&anc_e1[anc_e1.len() - 1]);
                                    if tangent_faces {
                                        continue;
                                    }
                                    let f2 =
                                        topods::face(&first_image(init_offset_face, init_f2)?)?;
                                    if !self.is_done(&f1, &f2) {
                                        BRepOffsetTool::inter3d(
                                            &f1,
                                            &f2,
                                            &mut l_int1,
                                            &mut l_int2,
                                            side,
                                            &null_edge,
                                            &null_face,
                                            &null_face,
                                        )?;
                                        self.store(&f1, &f2, &l_int1, &l_int2);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let _ = a_ps_int_t.next(1.);
        }
        Ok(())
    }

    /// `SetDone(F1, F2)`: records the pair both ways; `inter.set_done(&f1, &f2)`.
    pub fn set_done(&mut self, f1: &Face, f2: &Face) {
        self.my_done
            .try_emplaced(f1.as_ref().clone(), List::default)
            .append(f2.as_ref().clone());
        self.my_done
            .try_emplaced(f2.as_ref().clone(), List::default)
            .append(f1.as_ref().clone());
    }

    /// `IsDone(F1, F2)`: whether the pair was recorded; `inter.is_done(&f1, &f2)`.
    pub fn is_done(&self, f1: &Face, f2: &Face) -> bool {
        self.my_done
            .seek(f1.as_ref())
            .is_some_and(|l| l.iter().any(|s| s.is_same(f2.as_ref())))
    }

    /// `TouchedFaces()`: the faces with stored intersections; `inter.touched_faces()`.
    pub fn touched_faces(&mut self) -> &mut IndexedMapOfShape {
        &mut self.my_touched
    }

    /// `AsDes()`: the shared AsDes; `inter.as_des_handle()`.
    pub fn as_des_handle(&self) -> Handle<BRepAlgoAsDes> {
        self.my_as_des.clone()
    }

    /// `NewEdges()`: the stored intersection edges; `inter.new_edges()`.
    pub fn new_edges(&mut self) -> &mut IndexedMapOfShape {
        &mut self.my_new_edges
    }

    /// `Store(F1, F2, LInt1, LInt2)`: the sections as descendants of each face (the
    /// faces touched, the edges new) when there are any; the pair done.
    fn store(&mut self, f1: &Face, f2: &Face, l_int1: &List<Shape>, l_int2: &List<Shape>) {
        if !l_int1.is_empty() {
            self.my_touched.add(f1.as_ref().clone());
            self.my_touched.add(f2.as_ref().clone());
            self.as_des().add_many_shared(f1.as_ref(), l_int1);
            self.as_des().add_many_shared(f2.as_ref(), l_int2);
            for s in l_int1.iter() {
                self.my_new_edges.add(s.clone());
            }
        }
        self.set_done(f1, f2);
    }
}
