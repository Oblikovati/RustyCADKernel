//! The 2d ON pieces of `TopOpeBRepBuild_Builder1` (new builder): the pieces of an edge
//! of a face shared by a same domain edge of a same domain face, kept by the relative
//! orientations of the faces, the edges and their adjacent faces.
use super::same_dom_edge::{oriented_nearest_normal, oriented_normal};
use crate::top_ope_brep_build::{TopOpeBRepBuildBuilder, TopOpeBRepBuildTools};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::Direction3;

/// The six shapes of a pair of ON pieces (`aSeq`): the face, the edge and the piece of
/// the object, then of the tool.
#[derive(Clone)]
pub struct TwoPiecesSeq {
    /// The face of the object.
    pub face_obj: Shape,
    /// The edge of the object.
    pub edge_obj: Shape,
    /// The piece of the object.
    pub piece_obj: Shape,
    /// The face of the tool.
    pub face_tool: Shape,
    /// The edge of the tool.
    pub edge_tool: Shape,
    /// The piece of the tool.
    pub piece_tool: Shape,
}

/// The relative frame of two pieces: the faces differently oriented, the edges in
/// reversed senses, the adjacent faces found and the scalar products with them.
struct PiecesFrame {
    faces_dif_oriented: bool,
    edges_rev_sense: bool,
    ad: bool,
    flag1: bool,
    flag2: bool,
    sc_obj: f64,
    sc_tool: f64,
}

/// The pieces kept and their faces.
struct Kept<'a> {
    pieces: &'a mut List<Shape>,
    faces: &'a mut List<Shape>,
}

impl Kept<'_> {
    fn add(&mut self, piece: &Shape, face: &Shape) {
        self.pieces.append(piece.clone());
        self.faces.append(face.clone());
    }
}

/// The unit normal of a face at an edge, reversed for a reversed face.
fn unit_normal(f: &Shape, e: &Shape) -> Result<Direction3, Failure> {
    Direction3::from_vector(oriented_normal(f, e)?)
}

/// The scalar product `(dt ^ dn) * dna`, retried with the normal of the adjacent face
/// moved inside when below the tolerance.
fn adjacent_scalar(
    dt: Direction3,
    dn: Direction3,
    adj: &Shape,
    e: &Shape,
    tol: f64,
) -> Result<f64, Failure> {
    let mut dna = unit_normal(adj, e)?;
    let mut sc = dt.crossed(dn)?.dot(dna);
    if sc.abs() <= tol {
        let n = oriented_nearest_normal(adj, e)?;
        dna.set_xyz(*n.xyz())?;
        sc = dt.crossed(dn)?.dot(dna);
    }
    Ok(sc)
}

impl TopOpeBRepBuildBuilder {
    /// The pieces of the tool edges of one face of the same domain matching the object
    /// piece (all reoriented as the tool edge up to the match, in the structure).
    fn matching_tool_pieces(
        &self,
        piece_obj: &Shape,
        face_tool: &Shape,
        edge_tool: &Shape,
        tool_map_of_tool: bool,
    ) -> Result<Vec<(Shape, Shape)>, Failure> {
        let mut found = Vec::new();
        for e in Explorer::new(face_tool, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !e.is_same(edge_tool) {
                continue;
            }
            let et = edge_tool.oriented(e.orientation());
            let mut ds = self.hds()?.change_ds();
            let map = if tool_map_of_tool {
                ds.change_map_of_shape_with_state_tool()
            } else {
                ds.change_map_of_shape_with_state_obj()
            };
            let Some(part) = map.change_from_key(&et)?.part_mut(State::On) else {
                continue;
            };
            for piece in part.iter_mut() {
                piece.set_orientation(et.orientation());
                let deg = BRepTool::degenerated(&topods::edge(piece_obj)?)?
                    && BRepTool::degenerated(&topods::edge(piece)?)?;
                let same = if deg {
                    TopOpeBRepBuildTools::is_deg_edges_the_same(piece_obj, piece)?
                } else {
                    piece_obj.is_same(piece)
                };
                if same {
                    found.push((et.clone(), piece.clone()));
                    break;
                }
            }
        }
        Ok(found)
    }

    /// `PerformPieceOn2D(aPieceObj, aFaceObj, anEdgeObj, aListOfPieces, aListOfFaces,
    /// aListOfPiecesOut2d)` (Builder1): each piece of the same domain edges in the same
    /// domain faces matching the piece processed by `TwoPiecesON`; returns the number
    /// of matches (-1 when the face has no same domain), the pieces dropped for the
    /// contradictory cases; `b.perform_piece_on2d(&p, &f, &e, &mut lp, &mut lf, &mut lo)?`.
    pub fn perform_piece_on2d(
        &mut self,
        piece_obj: &Shape,
        face_obj: &Shape,
        edge_obj: &Shape,
        pieces: &mut List<Shape>,
        faces: &mut List<Shape>,
        out2d: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        let iref = self.hds()?.ds().ancestor_rank(face_obj);
        if !self.hds()?.has_same_domain(face_obj, true) {
            return Ok(-1);
        }
        let mut flag = 0;
        let mut cases: Map<i32> = Map::default();
        for face_tool in self.hds()?.same_domain(face_obj).iter() {
            let mut edges_tool = IndexedMapOfShape::default();
            topexp::map_shapes_of_type(face_tool, ShapeEnum::Edge, &mut edges_tool)?;
            if !self.hds()?.has_same_domain(edge_obj, true) {
                continue;
            }
            for edge_tool in self.hds()?.same_domain(edge_obj).iter() {
                if !edges_tool.contains(edge_tool) {
                    continue;
                }
                for (et, piece_tool) in
                    self.matching_tool_pieces(piece_obj, face_tool, edge_tool, iref == 1)?
                {
                    let seq = TwoPiecesSeq {
                        face_obj: face_obj.clone(),
                        edge_obj: edge_obj.clone(),
                        piece_obj: piece_obj.clone(),
                        face_tool: face_tool.clone(),
                        edge_tool: et,
                        piece_tool,
                    };
                    flag += 1;
                    cases.add(self.two_pieces_on(&seq, pieces, faces, out2d)?);
                }
            }
        }
        if flag > 1 {
            if cases.contains(&14) && cases.contains(&12) && self.opefus() {
                pieces.clear();
            }
            if cases.contains(&11) && cases.contains(&13) && (self.opec12() || self.opec21()) {
                pieces.clear();
            }
        }
        Ok(flag)
    }

    /// The frame of two pieces (the pieces reversed when against their edges).
    fn pieces_frame(
        s: &mut TwoPiecesSeq,
        map_obj: &IndexedDataMapOfShapeListOfShape,
        map_tool: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<PiecesFrame, Failure> {
        let tol = 1.0e-5;
        let dn_obj = unit_normal(&s.face_obj, &s.piece_obj)?;
        let dn_tool = unit_normal(&s.face_tool, &s.piece_tool)?;
        let faces_dif_oriented = dn_obj.dot(dn_tool) < 0.;
        let (fo, ft) = (topods::face(&s.face_obj)?, topods::face(&s.face_tool)?);
        let (rev_obj, t_obj) = TopOpeBRepBuildTools::get_tangent_to_edge_edge(
            &fo,
            &topods::edge(&s.piece_obj)?,
            &topods::edge(&s.edge_obj)?,
        )?;
        if rev_obj {
            s.piece_obj.reverse();
        }
        let dt_obj = Direction3::from_vector(t_obj)?;
        let (rev_tool, t_tool) = TopOpeBRepBuildTools::get_tangent_to_edge_edge(
            &ft,
            &topods::edge(&s.piece_tool)?,
            &topods::edge(&s.edge_tool)?,
        )?;
        if rev_tool {
            s.piece_tool.reverse();
        }
        let dt_tool = Direction3::from_vector(t_tool)?;
        let edges_rev_sense = dt_obj.dot(dt_tool) < 0.;
        let adj_obj = TopOpeBRepBuildTools::get_adjacent_face(&s.face_obj, &s.edge_obj, map_obj)?;
        let adj_tool =
            TopOpeBRepBuildTools::get_adjacent_face(&s.face_tool, &s.edge_tool, map_tool)?;
        let mut fr = PiecesFrame {
            faces_dif_oriented,
            edges_rev_sense,
            ad: false,
            flag1: false,
            flag2: false,
            sc_obj: 0.,
            sc_tool: 0.,
        };
        if let (Some(ao), Some(at)) = (adj_obj, adj_tool) {
            fr.ad = true;
            fr.sc_obj = adjacent_scalar(dt_obj, dn_tool, &ao, &s.piece_obj, tol)?;
            fr.sc_tool = adjacent_scalar(dt_tool, dn_obj, &at, &s.piece_tool, tol)?;
            fr.flag1 = fr.sc_tool.abs() > tol;
            fr.flag2 = fr.sc_obj.abs() > tol;
        }
        Ok(fr)
    }

    /// `TwoPiecesON(aSeq, aListOfPieces, aListOfFaces, aListOfPiecesOut2d)` (Builder1):
    /// the pieces of the pair kept by the operation and the configuration (faces
    /// differently oriented, edges in reversed senses, sides of the adjacent faces);
    /// returns the case number; `b.two_pieces_on(&seq, &mut lp, &mut lf, &mut lo)?`.
    pub fn two_pieces_on(
        &mut self,
        seq: &TwoPiecesSeq,
        pieces: &mut List<Shape>,
        faces: &mut List<Shape>,
        out2d: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        let iref = self.hds()?.ds().ancestor_rank(&seq.face_obj);
        let mut map_obj = IndexedDataMapOfShapeListOfShape::default();
        let mut map_tool = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.my_shape1,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut map_obj,
        )?;
        topexp::map_shapes_and_ancestors(
            &self.my_shape2,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut map_tool,
        )?;
        let mut s = seq.clone();
        if iref != 1 {
            s = TwoPiecesSeq {
                face_obj: seq.face_tool.clone(),
                edge_obj: seq.edge_tool.clone(),
                piece_obj: seq.piece_tool.clone(),
                face_tool: seq.face_obj.clone(),
                edge_tool: seq.edge_obj.clone(),
                piece_tool: seq.piece_obj.clone(),
            };
        }
        let fr = Self::pieces_frame(&mut s, &map_obj, &map_tool)?;
        let mut kept = Kept { pieces, faces };
        if !fr.ad || !fr.flag1 || !fr.flag2 {
            return self.two_pieces_unsure(&s, &fr, &mut kept);
        }
        self.two_pieces_sure(seq, &s, &fr, &mut kept, out2d)
    }

    /// The pieces kept when no adjacent face (or no side) decides: by the senses and
    /// orientations only.
    fn two_pieces_unsure(
        &self,
        s: &TwoPiecesSeq,
        fr: &PiecesFrame,
        kept: &mut Kept<'_>,
    ) -> Result<i32, Failure> {
        let (rev, dif) = (fr.edges_rev_sense, fr.faces_dif_oriented);
        if rev && !dif {
            if self.opec12()
                && !BRepTool::is_closed_on_face(
                    &topods::edge(&s.piece_obj)?,
                    &topods::face(&s.face_obj)?,
                )?
            {
                kept.add(&s.piece_obj, &s.face_obj);
            }
            return Ok(11);
        }
        if rev && dif {
            if self.opec12() {
                kept.add(&s.piece_obj, &s.face_obj);
            }
            return Ok(if fr.ad { 10 } else { 12 });
        }
        if fr.ad {
            if !self.opecom() {
                if !fr.flag2 {
                    kept.add(&s.piece_obj, &s.face_obj);
                }
                if !fr.flag1 {
                    kept.add(&s.piece_tool, &s.face_tool);
                }
            }
        } else if self.opefus() {
            kept.add(&s.piece_obj, &s.face_obj);
        }
        if !dif {
            return Ok(13);
        }
        Ok(if fr.ad { 10 } else { 14 })
    }

    /// The rejections of the reversed-sense, same-orientation case by the states of the
    /// sides (the fuse IN/IN and OUT/OUT parts processed in 2d once).
    fn rejections_rev_same(
        &mut self,
        seq: &TwoPiecesSeq,
        s: &TwoPiecesSeq,
        so: State,
        st: State,
        out2d: &mut List<Shape>,
    ) -> Result<(bool, bool), Failure> {
        let (mut r1, mut r2) = (true, true);
        if self.opefus() {
            match (so, st) {
                (State::Out, State::In) => r1 = false,
                (State::In, State::Out) => r2 = false,
                _ => self.is_same2d_once(seq, s, out2d)?,
            }
        }
        if self.opecom() {
            match (so, st) {
                (State::Out, State::In) => r2 = false,
                (State::In, State::Out) => r1 = false,
                _ => {}
            }
        }
        if self.opec12() {
            match (so, st) {
                (State::Out, State::Out) => r2 = false,
                (State::In, State::In) => r1 = false,
                (State::Out, State::In) => (r1, r2) = (false, false),
                _ => {}
            }
        }
        Ok((r1, r2))
    }

    /// The rejections of the same-sense, differently oriented case by the states of
    /// the sides (the fuse IN/IN parts processed in 2d once).
    fn rejections_same_dif(
        &mut self,
        seq: &TwoPiecesSeq,
        s: &TwoPiecesSeq,
        so: State,
        st: State,
        out2d: &mut List<Shape>,
    ) -> Result<(bool, bool), Failure> {
        let (mut r1, mut r2) = (true, true);
        if self.opefus() {
            match (so, st) {
                (State::Out, State::Out) => (r1, r2) = (false, false),
                (State::Out, State::In) => r2 = false,
                (State::In, State::Out) => r1 = false,
                _ => self.is_same2d_once(seq, s, out2d)?,
            }
        }
        if self.opecom() {
            match (so, st) {
                (State::In, State::In) => (r1, r2) = (false, false),
                (State::Out, State::In) => r1 = false,
                (State::In, State::Out) => r2 = false,
                _ => {}
            }
        }
        if self.opec12() {
            match (so, st) {
                (State::Out, State::Out) => r1 = false,
                (State::In, State::In) => r2 = false,
                _ => {}
            }
        }
        Ok((r1, r2))
    }

    /// `IsSame2d` of the pair unless its object piece was processed.
    fn is_same2d_once(
        &mut self,
        seq: &TwoPiecesSeq,
        s: &TwoPiecesSeq,
        out2d: &mut List<Shape>,
    ) -> Result<(), Failure> {
        if self.b1d()?.my_processed_parts_on2d.contains(&s.piece_obj) {
            return Ok(());
        }
        self.b1d_mut()?
            .my_processed_parts_on2d
            .add(s.piece_obj.clone());
        Self::is_same2d(seq, out2d)?;
        Ok(())
    }

    /// The pieces kept when the sides of both adjacent faces decide: by the states of
    /// the sides.
    fn two_pieces_sure(
        &mut self,
        seq: &TwoPiecesSeq,
        s: &TwoPiecesSeq,
        fr: &PiecesFrame,
        kept: &mut Kept<'_>,
        out2d: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        let so = if fr.sc_obj > 0. {
            State::In
        } else {
            State::Out
        };
        let st = if fr.sc_tool > 0. {
            State::In
        } else {
            State::Out
        };
        let (rev, dif) = (fr.edges_rev_sense, fr.faces_dif_oriented);
        if rev && dif {
            if self.opec12() {
                kept.add(&s.piece_obj, &s.face_obj);
            }
            return Ok(1);
        }
        if !rev && !dif {
            if !self.opec12() {
                kept.add(&s.piece_obj, &s.face_obj);
            }
            return Ok(3);
        }
        let (r1, r2, case) = if rev {
            let (r1, r2) = self.rejections_rev_same(seq, s, so, st, out2d)?;
            (r1, r2, 2)
        } else {
            let (r1, r2) = self.rejections_same_dif(seq, s, so, st, out2d)?;
            (r1, r2, 4)
        };
        if !r1 {
            kept.add(&s.piece_obj, &s.face_obj);
        }
        if !r2 {
            kept.add(&s.piece_tool, &s.face_tool);
        }
        Ok(case)
    }
}
