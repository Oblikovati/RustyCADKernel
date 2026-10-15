// Rust translation of BiTgte_Blend::ComputeSurfaces of
// ModelingAlgorithms/TKOffset/BiTgte/BiTgte_Blend.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The blend surfaces: the tubes along the cut center lines and the spheres at the
//! vertices where three tubes meet.
use super::statics::{find_created_edge, find_vertex, k_part_curve3d, live, make_curve, make_degenerated_edge, orientation};
use super::BiTgteBlend;
use crate::bi_tgte::BiTgteCurveOnEdge;
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetOffset, PipeOptions};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The support of one side of a center line: an edge (with its 3d curve) or a face
/// (with its surface).
struct Support {
    is_edge: bool,
    edge: Edge,
    curve: Handle<dyn GeomCurve>,
    face: Face,
    surface: Handle<dyn GeomSurface>,
}

/// The support of the shape generating a parallel face (`None` for a vertex: nothing is
/// done natively).
fn support(or: &Shape) -> Result<Option<Support>, Failure> {
    let mut s = Support {
        is_edge: false,
        edge: Edge::default(),
        curve: Handle::default(),
        face: Face::default(),
        surface: Handle::default(),
    };
    match or.shape_type()? {
        ShapeEnum::Edge => {
            s.is_edge = true;
            s.edge = topods::edge(or)?;
            let c = BRepTool::curve_with_location(&s.edge)?;
            s.curve = live(&c.curve, "curve")?.transformed(c.location.transformation())?.cast();
        }
        ShapeEnum::Face => {
            s.face = topods::face(or)?;
            s.surface = BRepTool::surface(&s.face)?;
        }
        _ => {}
    }
    // ----------------------------------------------------------------
    // If a vertex is used in contact, currently nothing is done
    // and the vertexes are not managed (Intersections with sphere);
    // ----------------------------------------------------------------
    if s.face.is_null() && s.edge.is_null() {
        return Ok(None);
    }
    Ok(Some(s))
}

/// The edge of the tube on one support: the curve of the center line projected on the
/// edge, or the p-curve of the center line on the parallel face put on the face.
fn edge_on_support(
    s: &Support,
    cur_cut_e: &Edge,
    (pc, f, l): (&Handle<dyn Geom2dCurve>, f64, f64),
    (map_on_v1f, map_on_v1l): (&MapOfShape, &MapOfShape),
    tol: f64,
) -> Result<(Edge, Vertex, Vertex), Failure> {
    let e;
    let (vf_on_e, vl_on_e);
    if s.is_edge {
        let con_e = BiTgteCurveOnEdge::with_edges(cur_cut_e, &s.edge)?;
        let c = make_curve(&con_e)?;
        let curve = live(&c, "curve")?;
        let p1 = curve.value(curve.first_parameter())?;
        let p2 = curve.value(curve.last_parameter())?;
        let mut vf = find_vertex(p1, map_on_v1f, tol)?;
        if vf.is_null() {
            vf = find_vertex(p1, map_on_v1l, tol)?;
        }
        let mut vl = find_vertex(p2, map_on_v1l, tol)?;
        if vl.is_null() {
            vl = find_vertex(p2, map_on_v1f, tol)?;
        }
        if p1.square_distance(&p2) < tol * tol {
            // BRepOffset_Offset manages degenerated KPart
            // It is REQUIRED that C should be a circle with ZERO radius
            e = make_degenerated_edge(&c, &vf)?;
        } else {
            e = BRepLibMakeEdge::from_curve_vertices(&c, &vf, &vl)?.edge()?;
        }
        vf_on_e = vf;
        vl_on_e = vl;
    } else {
        let pcurve = live(pc, "pcurve")?;
        let surface = live(&s.surface, "surface")?;
        let p2d = pcurve.value(f)?;
        let p1 = surface.value(p2d.x(), p2d.y())?;
        let p2d = pcurve.value(l)?;
        let p2 = surface.value(p2d.x(), p2d.y())?;
        vf_on_e = find_vertex(p1, map_on_v1f, tol)?;
        vl_on_e = find_vertex(p2, map_on_v1l, tol)?;
        let mke = BRepLibMakeEdge::from_pcurve_vertices_params(pc, &s.surface, &vf_on_e, &vl_on_e, f, l)?;
        // The native code reports "Edge Not Done" on the standard output, then the
        // edge raises StdFail_NotDone.
        e = mke.edge()?;

        k_part_curve3d(&e, pc, &s.surface)?;
    }
    Ok((e, vf_on_e, vl_on_e))
}

/// The edge of the map joining the two vertices (null when there is none).
fn joining_edge(map: &MapOfShape, va: &Vertex, vb: &Vertex) -> Result<Edge, Failure> {
    for e in map.iter() {
        let e = topods::edge(e)?;
        if !e.is_null() {
            let (a1, a2) = topexp::vertices(&e, false);
            if (a1.is_same(va) && a2.is_same(vb)) || (a2.is_same(va) && a1.is_same(vb)) {
                return Ok(e);
            }
        }
    }
    Ok(Edge::default())
}

impl BiTgteBlend {
    /// `ComputeSurfaces()`: for each cut part of a center line between two parallel
    /// faces, the tube (BRepOffset_Offset of the center line between the edges on the
    /// two supports) and, when the shape is built, the edges created on the supports and
    /// the cut vertices of the support edges; then the spheres at the vertices of three
    /// center lines.
    pub fn compute_surfaces(&mut self) -> Result<(), Failure> {
        // set in myFaces, the faces actually implied in the connection
        self.my_faces.clear(true);

        // construct
        // 1 - Tubes (True Fillets)
        // 2 - Spheres.

        let tol_angle = 2. * (self.my_tol / (self.my_radius * 0.5).abs()).asin();
        let center_analyse = BRepOffsetAnalyse::with_shape(&self.my_result, tol_angle)?;

        // -----------------------------------------------------
        // Construction of tubes in myResult
        // -----------------------------------------------------
        let b = BRepBuilder::default();
        let mut result = Compound::default();
        b.make_compound(&mut result);
        self.my_result = result.into_shape();

        // --------------------------------------------------------------------
        // Dummy: for construction of spheres:
        // Set in Co the center line, then it there are at least 3
        // center lines sharing the same vertex, Sphere on this vertex.
        // --------------------------------------------------------------------
        let mut co = Compound::default();
        b.make_compound(&mut co);

        // --------------------------------------------------------------------
        // Iteration on the edges lines of center
        // and their valid valid part is taken after cut and tube construction.
        // --------------------------------------------------------------------
        let as_des = self.my_as_des.clone();
        let as_des = live(&as_des, "AsDes")?;
        for i in 1..=self.my_edges.extent() {
            let cur_e = topods::edge(self.my_edges.find_key(i)?)?;

            let l: Vec<Shape> = as_des.ascendant(&cur_e).collect();
            if l.len() != 2 {
                continue;
            }

            // --------------------------------------------------------------
            // F1 and F2 = 2 parallel faces intersecting in CurE.
            // --------------------------------------------------------------
            let f1 = topods::face(&l[0])?;
            let f2 = topods::face(&l[1])?;

            // -----------------------------------------------------
            // find the orientation of edges of intersection
            // in the initial faces.
            // -----------------------------------------------------
            let ld1: Vec<Shape> = as_des.descendant(&f1).collect();
            let ld2: Vec<Shape> = as_des.descendant(&f2).collect();

            let orien1 = orientation(&cur_e, &f1, &ld1);
            let orien2 = orientation(&cur_e, &f2, &ld2);

            // ---------------------------------------------------------
            // Or1 and Or2 : the shapes generators of parallel faces
            // ---------------------------------------------------------
            let or1 = self.my_init_offset_face.image_from(&f1)?;
            let or2 = self.my_init_offset_face.image_from(&f2)?;

            self.my_faces.add(or1.clone());
            self.my_faces.add(or2.clone());

            let Some(s1) = support(&or1)? else {
                continue;
            };
            let Some(s2) = support(&or2)? else {
                continue;
            };

            let cur_l: Vec<Shape> = if !self.my_image_offset.has_image(&cur_e) {
                // the tubes are not unwinded
                if s1.is_edge && s2.is_edge {
                    // if I don't have the image, possibly
                    vec![cur_e.clone().into_shape()] // I'm on intersection tube-tube
                } else {
                    // See comment on the call to MakeLoops
                    continue;
                }
            } else {
                self.my_image_offset.last_image(&cur_e)
            };

            // ---------------------------------------------------------------
            // CurL = List of edges descending from CurE ( = Cuts of CurE)
            // ---------------------------------------------------------------
            for cut in &cur_l {
                let cur_cut_e = topods::edge(cut)?;

                let pc1 = BRepTool::curve_on_face(&cur_cut_e, &f1)?;
                let pc2 = BRepTool::curve_on_face(&cur_cut_e, &f2)?;
                if pc1.pcurve.is_null() || pc2.pcurve.is_null() {
                    continue;
                }

                let mut map_on_v1f = MapOfShape::default();
                let mut map_on_v1l = MapOfShape::default();

                let (v1f, v1l) = topexp::vertices(&cur_cut_e, false);

                // find if the pipe on the tangent edges are soon created.
                // edges generated by V1f and V1l + Maj MapOnV1f/l
                let mut e1f = find_created_edge(
                    &v1f,
                    &cur_cut_e,
                    &self.my_map_sf,
                    &mut map_on_v1f,
                    &center_analyse,
                    self.my_radius,
                    self.my_tol,
                )?;

                let mut e1l = find_created_edge(
                    &v1l,
                    &cur_cut_e,
                    &self.my_map_sf,
                    &mut map_on_v1l,
                    &center_analyse,
                    self.my_radius,
                    self.my_tol,
                )?;

                let maps = (&map_on_v1f, &map_on_v1l);
                let (mut e1, vf_on_e1, vl_on_e1) = edge_on_support(
                    &s1,
                    &cur_cut_e,
                    (&pc1.pcurve, pc1.first, pc1.last),
                    maps,
                    self.my_tol,
                )?;
                let (mut e2, vf_on_e2, vl_on_e2) = edge_on_support(
                    &s2,
                    &cur_cut_e,
                    (&pc2.pcurve, pc2.first, pc2.last),
                    maps,
                    self.my_tol,
                )?;
                // Increment of the Map of Created if reconstruction of the Shape is required
                if self.my_build_shape {
                    let mut by_support = DataMap::default();
                    let mut l1 = List::default();
                    l1.append(e1.clone().into_shape());
                    by_support.bind(or1.clone(), l1);
                    let mut l2 = List::default();
                    l2.append(e2.clone().into_shape());
                    by_support.bind(or2.clone(), l2);
                    self.my_created.bind(cur_cut_e.clone().into_shape(), by_support);
                }

                // ----------------------------------------------------------
                // try to init E1f, E1l, if not found with Analysis.
                // Should happen only if the THEORETICALLY tangent edges
                // are not actually tangent ( Cf: Approximation of lines
                // of intersection that add noise.)
                // ----------------------------------------------------------
                if e1f.is_null() && !vf_on_e1.is_null() && !vf_on_e2.is_null() {
                    e1f = joining_edge(&map_on_v1f, &vf_on_e1, &vf_on_e2)?;
                }
                if e1l.is_null() && !vl_on_e1.is_null() && !vl_on_e2.is_null() {
                    e1l = joining_edge(&map_on_v1l, &vl_on_e1, &vl_on_e2)?;
                }

                e1.set_orientation(orien1);
                e2.set_orientation(orien2);

                let an_offset = BRepOffsetOffset::from_edges_with_connectors(
                    &cur_cut_e,
                    &e1,
                    &e2,
                    -self.my_radius,
                    &e1f,
                    &e1l,
                    PipeOptions {
                        polynomial: self.my_nubs,
                        tol: self.my_tol,
                        conti: Continuity::C2,
                    },
                )?;
                let tuyo = an_offset.face().clone();
                self.my_map_sf.bind(cur_cut_e.clone().into_shape(), an_offset);
                self.my_centers.add(cur_cut_e.clone().into_shape());
                b.add(&mut co, &cur_cut_e)?;

                b.add(&mut self.my_result, &tuyo)?;

                if self.my_build_shape {
                    // method based ONLY on the construction of fillet:
                    // the first edge of the tube is exactly on Shape1.
                    let tuyo_edges: Vec<Shape> =
                        rustycad_tkbrep::topexp::Explorer::new(&tuyo, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
                    if s1.is_edge {
                        // Update CutEdges.
                        let e_on_f1 = topods::edge(tuyo_edges.first().ok_or_else(no_edge)?)?;
                        self.cut_support_edge(&e_on_f1, &s1.curve, &or1)?;
                    }
                    if s2.is_edge {
                        // Update CutEdges.
                        let e_on_f2 = topods::edge(tuyo_edges.get(1).ok_or_else(no_edge)?)?;
                        self.cut_support_edge(&e_on_f2, &s2.curve, &or2)?;
                    }
                }
            }
        }

        // ---------------------------------------------------
        // Construction of spheres,
        // if enough tubes arrive at the vertex
        // ---------------------------------------------------
        let mut map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&co, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;

        for j in 1..=map.extent() {
            let v = topods::vertex(map.find_key(j)?)?;
            let edges: Vec<Shape> = map.find_from_index(j)?.iter().cloned().collect();
            if edges.len() != 3 {
                continue;
            }

            let mut loe = List::default();
            for e in &edges {
                loe.append(self.offset(e)?.generated(&v).reversed());
            }

            let oft = BRepOffsetOffset::from_vertex(
                &v,
                &loe,
                -self.my_radius,
                PipeOptions {
                    polynomial: self.my_nubs,
                    tol: self.my_tol,
                    conti: Continuity::C2,
                },
            )?;
            let face = oft.face().clone();
            self.my_map_sf.bind(v.clone().into_shape(), oft);
            self.my_centers.add(v.into_shape());

            b.add(&mut self.my_result, &face)?;
        }
        Ok(())
    }

    /// The vertices of the edge of a tube on a support edge put on the support edge
    /// (INTERNAL, at their projections) and recorded in `myCutEdges`.
    fn cut_support_edge(&mut self, e_on_f: &Edge, gc: &Handle<dyn GeomCurve>, or: &Shape) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let (v1, v2) = topexp::vertices(e_on_f, false);
        let mut projector = ProjectPointOnCurve::new();

        let p1 = BRepTool::pnt(&v1)?;
        projector.init(p1, gc)?;
        let u1 = projector.lower_distance_parameter()?;

        let p2 = BRepTool::pnt(&v2)?;
        projector.init(p2, gc)?;
        let u2 = projector.lower_distance_parameter()?;

        let or_e = topods::edge(or)?;
        b.update_vertex_on_edge(&topods::vertex(&v1.oriented(Orientation::Internal))?, u1, &or_e, self.my_tol)?;
        b.update_vertex_on_edge(&topods::vertex(&v2.oriented(Orientation::Internal))?, u2, &or_e, self.my_tol)?;

        if !self.my_cut_edges.is_bound(or) {
            self.my_cut_edges.bind(or.clone(), List::default());
        }
        let l = self
            .my_cut_edges
            .change_seek(or)
            .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::ChangeFind"))?;
        l.append(v1.into_shape());
        l.append(v2.into_shape());
        Ok(())
    }
}

/// Standard_NoSuchObject of an exhausted explorer.
fn no_edge() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "TopExp_Explorer::Current")
}
