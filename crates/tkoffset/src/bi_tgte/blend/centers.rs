// Rust translation of BiTgte_Blend::Perform, BiTgte_Blend::ComputeCenters and
// BiTgte_Blend::Intersect of ModelingAlgorithms/TKOffset/BiTgte/BiTgte_Blend.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The center lines of the blend: the offsets of the faces and the tubes on the edges
//! by the radius, intersected and unwound.
use super::statics::{add, is_in_face, is_on_restriction, live};
use super::BiTgteBlend;
use crate::brep_offset::{CoincidentVertices, EdgeIntersections};
use crate::brep_offset::BRepOffsetInter3d;
use crate::brep_offset::{
    BRepOffsetInter2d, BRepOffsetMakeLoops, BRepOffsetOffset, BRepOffsetStatus, BRepOffsetTool, CreatedShapes,
    EnlargeFaceOptions, PipeOptions,
};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::{JoinType, Shape as Continuity};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
use rustycad_tktopalgo::brep_lib;

/// The bounding boxes of the offset faces (`MapSBox`).
type BoxMap = DataMap<Shape, BoundingBox3, ShapeMapHasher>;

impl BiTgteBlend {
    /// `Perform(BuildShape)`: the shape sewed (to remove false free borders), then the
    /// center lines, the blend surfaces and, when asked, the blended shape;
    /// Standard_Failure when the sewing gives no shape; `blend.perform(true)?`.
    pub fn perform(&mut self, build_shape: bool) -> Result<(), Failure> {
        self.my_build_shape = build_shape;

        // Try cutting to avoid tubes on free borders
        // that are not actually free.
        let mut sew = BRepBuilderAPISewing::new(self.my_tol, true, true, true, false);
        brep_lib::build_curves3d_default(&self.my_shape)?;
        for f in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            sew.add(&f)?;
        }
        sew.perform(&ProgressRange::default())?;
        let mut sewed_shape = sew.sewed_shape().clone();
        if sewed_shape.is_null() {
            return Err(Failure::with_message("Standard_Failure", "Sewing aux fraises"));
        }

        // Check if the sewing modified the orientation.
        let mut faces = Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)?;
        let first_face: Shape = Iterator::next(&mut faces).unwrap_or_default();
        let mut face_ref = topods::face(&first_face)?;
        let ori_ref = face_ref.orientation();
        if sew.is_modified(&face_ref) {
            face_ref = topods::face(&sew.modified(&face_ref))?;
        }
        for ff in Explorer::new(&sewed_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if face_ref.is_same(&ff) && ff.orientation() != ori_ref {
                sewed_shape.reverse();
                break;
            }
        }

        // Make SameParameter if Sew does not do it (Detect that edges
        // are not sameparameter but it does nothing.)
        for sec in Explorer::new(&sewed_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let sec = topods::edge(&sec)?;
            brep_lib::same_parameter_edge(&sec, BRepTool::edge_tolerance(&sec)?)?;
        }

        topexp::map_shapes_and_ancestors(&sewed_shape, ShapeEnum::Edge, ShapeEnum::Face, &mut self.my_ancestors)?;

        // Extend myFaces with the faces of the sewed shape.
        for f in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.my_faces.contains(&f) && sew.is_modified(&f) {
                self.my_faces.remove_key(&f);
                self.my_faces.add(sew.modified(&f));
            }
        }

        self.my_shape = sewed_shape;
        // end Sewing for false free borders.

        // ----------------------------------------------------------------
        // place faces with the proper orientation in the initial shape
        // ----------------------------------------------------------------
        let shape = self.my_shape.clone();
        for f in Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.my_faces.contains(&f) {
                self.my_faces.remove_key(&f);
                self.my_faces.add(f);
            } else if self.my_stop_faces.contains(&f) {
                self.my_stop_faces.remove(&f);
                self.my_stop_faces.add(f);
            }
        }

        // ----------------------------------------------
        // Calculate lines of centers and of surfaces
        // ----------------------------------------------
        self.compute_centers()?;

        // -----------------------------
        // Calculate connection Surfaces
        // -----------------------------
        self.compute_surfaces()?;

        // ----------------------------------
        // Calculate the generated shape if required
        // ----------------------------------
        if self.my_build_shape {
            self.compute_shape()?;
        }

        // Finally construct curves 3d from edges to be transferred
        // since the partition is provided ( A Priori);
        brep_lib::build_curves3d(&self.my_result, precision::confusion(), Continuity::C1, 14, 0)?;

        self.my_done = true;
        Ok(())
    }

    /// `ComputeCenters()`: the offsets of the faces of the edges to blend and of the
    /// faces they touch, the tubes on their convex (concave) edges, all intersected
    /// with each other until no new face is touched; then the center lines limited by
    /// the 2d intersections and unwound by MakeLoops into `myResult`.
    pub fn compute_centers(&mut self) -> Result<(), Failure> {
        // ------------
        // Preanalyze.
        // ------------
        let tol_angle = 2. * (self.my_tol / (self.my_radius * 0.5).abs()).asin();
        let shape = self.my_shape.clone();
        self.my_analyse.perform(&shape, tol_angle)?;

        // ------------------------------------------
        // calculate faces touched by caps
        // ------------------------------------------
        // `Touched` does nothing natively: no face is touched by a cap.
        let touched_by_cork = MapOfShape::default();

        // -----------------------
        // init of the intersector
        // -----------------------
        let side = if self.my_radius < 0. { State::Out } else { State::In };
        let mut inter = BRepOffsetInter3d::new(&self.my_as_des, side, self.my_tol);

        let mut map_s_box = BoxMap::default();

        let b = BRepBuilder::default();
        let mut co = Compound::default(); // to only know on which edges the tubes are made
        b.make_compound(&mut co);

        // ----------------------------------------
        // Calculate Sections Face/Face + Propagation
        // ----------------------------------------
        let mut jen_rajoute = true;

        while jen_rajoute {
            jen_rajoute = false;

            let mut fini = false;

            let mut edge_tgt = CreatedShapes::default();

            while !fini {
                // -------------------------------------------------
                // locate in myFaces the Faces connected to myEdges.
                // -------------------------------------------------
                fini = true;
                for i in 1..=self.my_edges.extent() {
                    let e = topods::edge(self.my_edges.find_key(i)?)?;
                    if BRepTool::degenerated(&e)? {
                        continue;
                    }

                    let l: Vec<Shape> = self.my_ancestors.find_from_key(&e)?.iter().cloned().collect();
                    if l.len() == 1 {
                        // So this is a free border onwhich the ball should roll.
                        self.my_faces.add(e.clone().into_shape());

                        // set in myStopFaces to not propagate the tube on free border.
                        self.my_stop_faces.add(e.into_shape());
                    } else {
                        for sh in &l {
                            if !self.my_stop_faces.contains(sh) {
                                self.my_faces.add(sh.clone());
                            }
                        }
                    }
                }
                self.my_edges.clear(true);

                // --------------------------------------------
                // Construction of Offsets of all faces.
                // --------------------------------------------
                let mut i = 1;
                while i <= self.my_faces.extent() {
                    let as_ = self.my_faces.find_key(i)?.clone();
                    i += 1;
                    if self.my_map_sf.is_bound(&as_) {
                        continue;
                    }

                    let mut of1 = BRepOffsetOffset::new();

                    if as_.shape_type()? == ShapeEnum::Face {
                        let f = topods::face(&as_)?;
                        if touched_by_cork.contains(&f) {
                            let mut big_f = Face::default();
                            BRepOffsetTool::en_large_face(&f, &mut big_f, EnlargeFaceOptions::with_change_geom(true))?;
                            of1.init_face_created(&big_f, self.my_radius, &edge_tgt, true, JoinType::Arc)?;
                        } else {
                            of1.init_face_created(&f, self.my_radius, &edge_tgt, true, JoinType::Arc)?;
                        }
                    } else {
                        // So this is a Free Border edge on which the ball rolls.
                        of1.init_edge(&topods::edge(&as_)?, self.my_radius)?;
                    }

                    // ------------------------------------
                    // Increment the map of created tangents
                    // ------------------------------------
                    let mut let_ = Vec::new();
                    if as_.shape_type()? == ShapeEnum::Face {
                        self.my_analyse.edges_on_face(&topods::face(&as_)?, TypeOfConcavity::Tangential, &mut let_)?;
                    }

                    for cur_s in &let_ {
                        if edge_tgt.is_bound(cur_s) {
                            continue;
                        }
                        let cur = topods::edge(cur_s)?;
                        let ote = topods::edge(&of1.generated(cur_s))?;
                        edge_tgt.bind(cur_s.clone(), of1.generated(cur_s));
                        let (v1, v2) = topexp::vertices(&cur, false);
                        let (ov1, ov2) = topexp::vertices(&ote, false);
                        let mut le = Vec::new();
                        if !edge_tgt.is_bound(&v1) {
                            self.my_analyse.edges_on_vertex(&v1, TypeOfConcavity::Tangential, &mut le)?;
                            let la = self.my_analyse.ancestors(&v1)?;
                            if le.len() == la.len() {
                                edge_tgt.bind(v1.clone().into_shape(), ov1.into_shape());
                            }
                        }
                        if !edge_tgt.is_bound(&v2) {
                            le.clear();
                            self.my_analyse.edges_on_vertex(&v2, TypeOfConcavity::Tangential, &mut le)?;
                            let la = self.my_analyse.ancestors(&v2)?;
                            if le.len() == la.len() {
                                edge_tgt.bind(v2.clone().into_shape(), ov2.into_shape());
                            }
                        }
                    }
                    // end of map created tangent

                    if of1.status() == BRepOffsetStatus::Reversed || of1.status() == BRepOffsetStatus::Degenerated {
                        continue;
                    }

                    let f1 = of1.face().clone();

                    // increment S D
                    self.my_init_offset_face.set_root(&as_);
                    self.my_init_offset_face.bind(&as_, &f1)?;

                    let mut box1 = BoundingBox3::default();
                    BRepBndLib::add(&f1, &mut box1, true)?;
                    map_s_box.bind(f1.clone().into_shape(), box1);

                    // ---------------------------------------------
                    // intersection with all already created faces.
                    // ---------------------------------------------
                    fini = !self.intersect(&as_, &f1, &map_s_box, &of1, &mut inter)?;

                    if as_.shape_type()? == ShapeEnum::Face {
                        b.add(&mut co, &as_)?;
                    }

                    self.my_map_sf.bind(as_, of1);
                }
            } // end of : while ( !Fini)

            //--------------------------------------------------------
            // so the offsets were created and intersected.
            // now the tubes are constructed.
            //--------------------------------------------------------
            // Construction of tubes on edge.
            //--------------------------------------------------------
            let ot = if self.my_radius < 0. { TypeOfConcavity::Concave } else { TypeOfConcavity::Convex };

            let mut map = IndexedDataMapOfShapeListOfShape::default();
            topexp::map_shapes_and_ancestors(&co, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
            topexp::map_shapes_and_ancestors(&co, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;

            for e in Explorer::new(&co, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let e = topods::edge(&e)?;
                if self.my_map_sf.is_bound(&e) {
                    continue;
                }

                let anc: Vec<Shape> = map.find_from_key(&e)?.iter().cloned().collect();
                if anc.len() != 2 {
                    continue;
                }
                let l = self.my_analyse.type_of(&e)?;
                if l.is_empty() || l[0].get_type()? != ot {
                    continue;
                }
                let e_on1 = topods::edge(&self.offset(&anc[0])?.generated(&e))?;
                let e_on2 = topods::edge(&self.offset(&anc[1])?.generated(&e))?;
                // find if exits tangent edges in the original shape
                let (v1f, v1l) = topexp::vertices(&e, false);
                let e1f = self.created_tangent_edge(&e, &v1f)?;
                let e1l = self.created_tangent_edge(&e, &v1l)?;
                let of1 = BRepOffsetOffset::from_edges_with_connectors(
                    &e,
                    &e_on1,
                    &e_on2,
                    self.my_radius,
                    &e1f,
                    &e1l,
                    PipeOptions::default(),
                )?;
                let f1 = of1.face().clone();

                // maj S D
                self.my_init_offset_face.set_root(&e);
                self.my_init_offset_face.bind(&e, &f1)?;

                let mut box1 = BoundingBox3::default();
                BRepBndLib::add(&f1, &mut box1, true)?;
                map_s_box.bind(f1.clone().into_shape(), box1);

                // ---------------------------------------------
                // intersection with all already created faces.
                // ---------------------------------------------
                let is_on_rest = self.intersect(&e, &f1, &map_s_box, &of1, &mut inter)?;
                jen_rajoute = jen_rajoute || is_on_rest;

                self.my_map_sf.bind(e.into_shape(), of1);
            }
        } // end while JenRajoute

        self.my_edges.clear(true);
        self.my_edges = inter.new_edges().clone();

        // -------------------------------------------------------------------
        // now it is necessary to limit edges on the neighbors (otherwise one
        // will go too far and will not be able to construct faces).
        // -------------------------------------------------------------------

        // Proceed with MakeLoops
        let mut a_dmvv = CoincidentVertices::default();
        let ot = if self.my_radius < 0. { TypeOfConcavity::Convex } else { TypeOfConcavity::Concave };

        let as_des = self.my_as_des.clone();
        let as_des_ref = live(&as_des, "AsDes")?;
        let mut lof = List::default();
        for i in 1..=self.my_faces.extent() {
            let cur_s = self.my_faces.find_key(i)?.clone();

            // tube on free border, it is undesirable.
            if self.my_stop_faces.contains(&cur_s) {
                continue;
            }

            let Some(of) = self.my_map_sf.seek(&cur_s) else {
                continue; // inverted or degenerated
            };

            let cur_of = of.face().clone();
            lof.append(cur_of.clone().into_shape());

            if cur_s.shape_type()? == ShapeEnum::Face {
                let cur_f = topods::face(&cur_s)?;
                for cur_e in Explorer::new(&cur_f.oriented(Orientation::Forward), ShapeEnum::Edge, ShapeEnum::Shape)? {
                    // --------------------------------------------------------------
                    // set in myAsDes the edges generated by limitations of the
                    // initial square if the type is correct (The edges that will
                    // disappear are not set)
                    // --------------------------------------------------------------
                    let cur_e = topods::edge(&cur_e)?;
                    let l = self.my_analyse.type_of(&cur_e)?;
                    if !l.is_empty() && l[0].get_type()? != ot {
                        // a priori doe s not disappear, so it is set
                        let cur_oe = of.generated(&cur_e);
                        as_des_ref.add_shared(&cur_of, &cur_oe.oriented(cur_e.orientation()));
                    } else {
                        let lanc = self.my_analyse.ancestors(&cur_e)?;
                        let (first, last) = (&lanc[0], lanc.last().unwrap_or(&lanc[0]));
                        if !self.my_faces.contains(first)
                            || !self.my_faces.contains(last)
                            || self.my_stop_faces.contains(first)
                            || self.my_stop_faces.contains(last)
                        {
                            let cur_oe = of.generated(&cur_e);
                            as_des_ref.add_shared(&cur_of, &cur_oe.oriented(cur_e.orientation()));
                        }
                    }
                }
                let an_empty_map = EdgeIntersections::default();
                BRepOffsetInter2d::compute(
                    as_des_ref,
                    &cur_of,
                    &self.my_edges,
                    self.my_tol,
                    &an_empty_map,
                    &mut a_dmvv,
                    &ProgressRange::default(),
                )?;
            }
        }

        // ----------------------------------------------------------------
        // It is also required to make 2D intersections with generated tubes
        // (Useful for unwinding)
        // ----------------------------------------------------------------
        let tubes: Vec<(Shape, Face)> = self
            .my_map_sf
            .items()
            .map(|(k, v)| (k.clone(), v.face().clone()))
            .collect();
        for (cur_s, cur_of) in tubes {
            if cur_s.shape_type()? == ShapeEnum::Face {
                continue;
            }

            // no unwinding by tubes on free border.
            if self.my_stop_faces.contains(&cur_s) {
                continue;
            }

            lof.append(cur_of.clone().into_shape());

            // --------------------------------------------------------------
            // set in myAsDes the edge restrictions of the square
            // --------------------------------------------------------------
            for cur_oe in Explorer::new(&cur_of.oriented(Orientation::Forward), ShapeEnum::Edge, ShapeEnum::Shape)? {
                as_des_ref.add_shared(&cur_of, &cur_oe);
            }

            let an_empty_map = EdgeIntersections::default();
            BRepOffsetInter2d::compute(
                as_des_ref,
                &cur_of,
                &self.my_edges,
                self.my_tol,
                &an_empty_map,
                &mut a_dmvv,
                &ProgressRange::default(),
            )?;
        }

        // fuse vertices on edges stored in AsDes
        let mut an_empty_image = BRepAlgoImage::new();
        BRepOffsetInter2d::fuse_vertices(&a_dmvv, as_des_ref, &mut an_empty_image)?;
        // ------------
        // unwinding
        // ------------
        let mut make_loops = BRepOffsetMakeLoops::new();
        make_loops.build(&lof, as_des_ref, &mut self.my_image_offset, &mut an_empty_image, &ProgressRange::default())?;

        // ------------------------------------------------------------
        // It is possible to unwind edges at least one ancestor which of
        // is a face of the initial shape, so:
        // the edges generated by intersection tube-tube are missing
        // ------------------------------------------------------------

        // --------------------------------------------------------------
        // Currently set the unwinded surfaces in <myResult>
        // --------------------------------------------------------------
        let mut result = Compound::default();
        b.make_compound(&mut result);
        for cur_lof in lof.iter() {
            if !self.my_image_offset.has_image(cur_lof) {
                continue;
            }

            let lim = self.my_image_offset.last_image(cur_lof);
            for cur_lim in &lim {
                // If a face is its own image, it is not set
                if cur_lim.is_same(cur_lof) {
                    break;
                }
                b.add(&mut result, cur_lim)?;
            }
        }
        self.my_result = result.into_shape();
        Ok(())
    }

    /// The offset of a shape (Standard_NoSuchObject when it has none).
    pub(super) fn offset(&self, s: &Shape) -> Result<&BRepOffsetOffset, Failure> {
        self.my_map_sf
            .seek(s)
            .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))
    }

    /// The edge generated at the vertex by the tube on a tangent edge of `E` already
    /// built (null when there is none).
    fn created_tangent_edge(&self, e: &Edge, v: &Vertex) -> Result<Edge, Failure> {
        let mut tang_e = Vec::new();
        self.my_analyse.tangent_edges(e, v, &mut tang_e)?;
        // find if the pipe on the tangent edges are soon created.
        for t in &tang_e {
            if let Some(of) = self.my_map_sf.seek(t) {
                return topods::edge(&of.generated(v));
            }
        }
        Ok(Edge::default())
    }

    /// `Intersect(Init, Face, MapSBox, OF1, Inter)`: the intersections of the face with
    /// the faces already built whose box meets its own (not the faces of the edge of a
    /// tube, nor two tubes on free borders); the restrictions touched at the ends of the
    /// sections are added to `myEdges` for propagation; true when one is added.
    fn intersect(
        &mut self,
        init: &Shape,
        face: &Face,
        map_s_box: &BoxMap,
        of1: &BRepOffsetOffset,
        inter: &mut BRepOffsetInter3d,
    ) -> Result<bool, Failure> {
        let mut jen_rajoute = false;

        let box1 = map_s_box
            .seek(face)
            .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))?;

        // -----------------------------------------------
        // intersection with all already created faces.
        // -----------------------------------------------
        let init_shape1 = of1.initial_shape();
        let f1_sur_bord_libre =
            init_shape1.shape_type()? == ShapeEnum::Edge && self.my_stop_faces.contains(init_shape1);

        let mut done = MapOfShape::default();
        let others: Vec<(Shape, BRepOffsetOffset)> =
            self.my_map_sf.items().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (it_key, of2) in &others {
            let f2 = of2.face().clone();

            let box2 = map_s_box
                .seek(&f2)
                .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))?;
            if box1.is_out_box(box2) {
                continue;
            }

            if inter.is_done(face, &f2) {
                continue;
            }

            // 2 tubes created on free border are not intersected.
            let init_shape2 = of2.initial_shape();
            let f2_sur_bord_libre =
                init_shape2.shape_type()? == ShapeEnum::Edge && self.my_stop_faces.contains(init_shape2);

            if f1_sur_bord_libre && f2_sur_bord_libre {
                continue;
            }

            // -------------------------------------------------------
            // Tubes are not intersected with neighbor faces.
            // -------------------------------------------------------
            if init.shape_type()? == ShapeEnum::Edge
                && it_key.shape_type()? == ShapeEnum::Face
                && is_in_face(&topods::edge(init)?, &topods::face(it_key)?)?
            {
                continue;
            }

            inter.face_inter(face, &f2, &self.my_init_offset_face)?;

            // ------------------------------------------
            // an edge of F1 or F2 has been touched ?
            // if yes, add faces in myFaces
            //   ==> JenRajoute = True
            // ------------------------------------------
            let mut l_int = List::default();
            done.clear(true);
            let common = live(&self.my_as_des, "AsDes")?.has_common_descendant(face, &f2, &mut l_int);
            if common {
                for cur_e in l_int.iter() {
                    let cur_e = topods::edge(cur_e)?;
                    let (v1, v2) = topexp::vertices(&cur_e, false);

                    for v in [v1, v2] {
                        if !done.add(v.clone().into_shape()) {
                            continue;
                        }
                        let (is_on_r1, e1) = is_on_restriction(&v, &cur_e, face)?;
                        let (is_on_r2, e2) = is_on_restriction(&v, &cur_e, &topods::face(&f2)?)?;
                        if is_on_r1 && !self.my_stop_faces.contains(init) {
                            add(&e1, &mut self.my_edges, init, of1, &self.my_analyse, is_on_r1 && is_on_r2)?;
                            jen_rajoute = true;
                        }
                        if is_on_r2 && !self.my_stop_faces.contains(it_key) {
                            add(&e2, &mut self.my_edges, it_key, of2, &self.my_analyse, is_on_r1 && is_on_r2)?;
                            jen_rajoute = true;
                        }
                    }
                }
            }
        }

        Ok(jen_rajoute)
    }
}
