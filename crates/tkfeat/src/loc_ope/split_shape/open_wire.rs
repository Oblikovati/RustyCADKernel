// Rust translation of LocOpe_SplitShape::AddOpenWire of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_SplitShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AddOpenWire`: a wire joining two points of the boundary of a face. Between two
//! points of one boundary wire the face is cut in two (the boundary chained from the
//! end of the wire, turning left at each vertex); between two boundary wires they are
//! merged into one.
use super::statics::{check_overlapping, choix_uv, get_direction, is_inside, same_uv};
use super::{SplitShape, to_list, to_vec};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Vector2};

/// The boundary wires of a descendant face holding the ends of the open wire.
struct EndWires {
    wfirst: Wire,
    wlast: Wire,
}

/// The wires of `fac` (FORWARD) holding `vfirst` and `vlast`, when one wire scan finds
/// both.
fn end_wires(fac: &Face, vfirst: &Vertex, vlast: &Vertex) -> Result<Option<EndWires>, Failure> {
    let (mut ffound, mut lfound) = (false, false);
    let (mut wfirst, mut wlast) = (Wire::default(), Wire::default());
    for w in Explorer::new(fac, ShapeEnum::Wire, ShapeEnum::Shape)? {
        let wir = topods::wire(&w)?;
        for v in Explorer::new(&wir, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if !ffound && v.is_same(vfirst) {
                ffound = true;
                wfirst = wir.clone();
            } else if !lfound && v.is_same(vlast) {
                lfound = true;
                wlast = wir.clone();
            }
            if ffound && lfound {
                return Ok(Some(EndWires { wfirst, wlast }));
            }
        }
    }
    Ok(None)
}

/// The first edge of the FORWARD wire `w` holding `v` (skipping `skip` when the wire
/// has more than one edge), with its orientation.
fn edge_holding(
    w: &Wire,
    v: &Vertex,
    skip: Option<&Edge>,
    nb_e: usize,
) -> Result<Option<Edge>, Failure> {
    for e in Explorer::new(
        &w.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )? {
        let edg = topods::edge(&e)?;
        if nb_e > 1 && skip.is_some_and(|s| edg.is_same(s)) {
            continue;
        }
        let mut vertices = Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if vertices.any(|x| x.is_same(v)) {
            return Ok(Some(edg));
        }
    }
    Ok(None)
}

/// The edges already chained, told apart by orientation (`NCollection_Map<TopoDS_Shape>`
/// with the default hasher, unlike the TopTools maps).
type OrientedEdges = Map<Shape>;

/// The state of the chaining of the boundary from the end of the open wire.
struct Chain {
    vlast: Vertex,
    last_edge: Edge,
    plast: Point2,
    dlast: Vector2,
}

/// The edges of a wire (with their orientation) that are not in `map_e`, the first two
/// returned for the overlapping check.
fn edges_of(newwire: &Wire) -> Vec<Shape> {
    ShapeIterator::new(newwire, false, true)
        .filter(|e| e.shape_type().ok() == Some(ShapeEnum::Edge))
        .collect()
}

/// Whether a new wire of exactly two overlapping edges (or fewer than two edges) makes
/// the cut impossible.
fn degenerate_wire(edges: &[Shape], face_ref: &Face) -> Result<bool, Failure> {
    if edges.len() < 2 {
        return Ok(true);
    }
    if edges.len() == 2 {
        return check_overlapping(
            &topods::edge(&edges[0])?,
            &topods::edge(&edges[1])?,
            face_ref,
        );
    }
    Ok(false)
}

impl SplitShape {
    /// The descendant of `f` holding the open wire and both its ends on its wires,
    /// removed from the descendants (as natively before any further check), with the
    /// wires of the ends.
    fn take_face_with_ends(
        &mut self,
        f: &Face,
        w: &Wire,
        vfirst: &Vertex,
        vlast: &Vertex,
    ) -> Result<Option<(Face, EndWires)>, Failure> {
        let mut lf = to_vec(self.list_of(f)?);
        for k in 0..lf.len() {
            let mut fac = topods::face(&lf[k])?;
            if !is_inside(&fac, w)? {
                continue;
            }
            fac.set_orientation(Orientation::Forward);
            if let Some(ends) = end_wires(&fac, vfirst, vlast)? {
                let mut face_ref = topods::face(&lf.remove(k))?;
                face_ref.set_orientation(Orientation::Forward);
                *self.list_mut(f)? = to_list(lf);
                return Ok(Some((face_ref, ends)));
            }
        }
        Ok(None)
    }

    /// `AddOpenWire(W, F)`: the cut of the descendant of `f` holding `w` (both its ends
    /// on the boundary); false when none does or the cut is impossible.
    pub(super) fn add_open_wire(&mut self, w: &Wire, f: &Face) -> Result<bool, Failure> {
        // On cherche la face descendante de F qui continent le wire
        brep_tools::update_face(f)?;
        let (vfirst, vlast) =
            topexp::wire_vertices(&topods::wire(&w.oriented(Orientation::Forward))?);
        let Some((face_ref, ends)) = self.take_face_with_ends(f, w, &vfirst, &vlast)? else {
            return Ok(false);
        };
        if ends.wfirst.is_same(&ends.wlast) {
            self.split_in_two(w, f, &face_ref, &ends.wfirst, (&vfirst, &vlast))
        } else {
            self.merge_wires(w, f, &face_ref, ends)
        }
    }

    /// The edges of `wfirst` (the seams of the face marked double) then the edges of
    /// the open wire in both orientations (marked double).
    fn first_wires(
        &mut self,
        w: &Wire,
        wfirst: &Wire,
        face_ref: &Face,
    ) -> Result<(Vec<Shape>, usize), Failure> {
        let mut wires_first = Vec::new();
        for e in Explorer::new(wfirst, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if BRepTool::is_closed_on_face(&topods::edge(&e)?, face_ref)? {
                self.dbl_e.add(e.clone());
            }
            wires_first.push(e);
        }
        let mut nb_e = 0;
        for e in Explorer::new(
            &w.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            nb_e += 1;
            let orient = e.orientation();
            wires_first.push(e.clone());
            wires_first.push(e.oriented(top_abs::reverse(orient)));
            self.dbl_e.add(e);
        }
        Ok((wires_first, nb_e))
    }

    /// The chain from the end of the open wire: the start point (at `vfirst`) and the
    /// end point and direction (at `vlast`) on the face oriented as `wfirst`.
    fn start_chain(
        w: &Wire,
        wface: &Face,
        nb_e: usize,
        (vfirst, vlast): (&Vertex, &Vertex),
    ) -> Result<(Point2, Chain), Failure> {
        // On recherche l`edge contenant Vlast
        let first_edge = edge_holding(w, vfirst, None, nb_e)?.unwrap_or_default();
        let pc = BRepTool::curve_on_face(&first_edge, wface)?;
        let c2d = pc.pcurve.get().ok_or_else(|| {
            crate::errors::null_object("LocOpe_SplitShape::AddOpenWire", "edge without p-curve")
        })?;
        let pfirst = if first_edge.orientation() == Orientation::Forward {
            c2d.value(pc.first)?
        } else {
            c2d.value(pc.last)?
        };
        let last_edge = edge_holding(w, vlast, Some(&first_edge), nb_e)?.unwrap_or(first_edge);
        let (plast, dlast) = get_direction(&last_edge, wface, false)?;
        Ok((
            pfirst,
            Chain {
                vlast: vlast.clone(),
                last_edge,
                plast,
                dlast,
            },
        ))
    }

    /// The edges of `wires_first` leaving `vlast` along their orientation.
    fn possible_edges(wires_first: &[Shape], vlast: &Vertex) -> Result<IndexedMapOfShape, Failure> {
        let mut poss = IndexedMapOfShape::default();
        for e in wires_first {
            let edg = topods::edge(e)?;
            let orient = edg.orientation();
            let (vdeb, vfin) = topexp::vertices(&edg, false);
            if (orient == Orientation::Forward && vlast.is_same(&vdeb))
                || (orient == Orientation::Reversed && vlast.is_same(&vfin))
            {
                poss.add(e.clone());
            }
        }
        Ok(poss)
    }

    /// Chains the first new wire from the end of the open wire until it closes; false
    /// when the chain breaks (the loop is not closed: split is not possible).
    fn chain_first_wire(
        chain: &mut Chain,
        wires_first: &[Shape],
        wface: &Face,
        new_w1: &mut Shape,
        map_e: &mut OrientedEdges,
    ) -> Result<bool, Failure> {
        let b = BRepBuilder::default();
        loop {
            // On enchaine par la fin
            let poss = Self::possible_edges(wires_first, &chain.vlast)?;
            let next = match poss.extent() {
                0 => return Ok(true),
                1 => {
                    let next = topods::edge(poss.find_key(1)?)?;
                    (chain.plast, chain.dlast) = get_direction(&next, wface, false)?;
                    next
                }
                // Faire choix en U,V...
                _ => match choix_uv(
                    &chain.last_edge,
                    wface,
                    &poss,
                    &mut chain.plast,
                    &mut chain.dlast,
                )? {
                    Some(next) => next,
                    None => return Ok(false),
                },
            };
            if map_e.contains(&next) {
                return Ok(true);
            }
            b.add(new_w1, &next)?;
            map_e.add(next.as_ref().clone());
            chain.vlast = if next.orientation() == Orientation::Forward {
                topexp::last_vertex(&next, false)
            } else {
                topexp::first_vertex(&next, false)
            };
            chain.last_edge = next;
        }
    }

    /// Two faces in place of the descendant: the boundary chained from the end of the
    /// open wire, and the rest; the other wires of the face go to the one holding them.
    fn split_in_two(
        &mut self,
        w: &Wire,
        f: &Face,
        face_ref: &Face,
        wfirst: &Wire,
        ends: (&Vertex, &Vertex),
    ) -> Result<bool, Failure> {
        let b = BRepBuilder::default();
        let bas = BRepAdaptorSurface::with_face(face_ref, false)?;
        let is_periodic = bas.is_u_periodic()? || bas.is_v_periodic()?;
        // on cree 2 faces en remplacement de itl.Value()
        // Essai JAG
        let (wires_first, nb_e) = self.first_wires(w, wfirst, face_ref)?;
        let mut new_w1 = Wire::default();
        b.make_wire(&mut new_w1);
        let mut new_w1 = new_w1.into_shape();
        let mut new_w2 = Wire::default();
        b.make_wire(&mut new_w2);
        let mut new_w2 = new_w2.into_shape();
        let wface = topods::face(&face_ref.oriented(wfirst.orientation()))?;
        let (pfirst, mut chain) = Self::start_chain(w, &wface, nb_e, ends)?;
        let same_end = ends.0.is_same(&chain.vlast);
        let cond = if is_periodic {
            !(same_end && same_uv(pfirst, chain.plast, &bas)?)
        } else {
            !same_end
        };
        let mut map_e = OrientedEdges::default();
        if cond
            && !Self::chain_first_wire(&mut chain, &wires_first, &wface, &mut new_w1, &mut map_e)?
        {
            return Ok(false);
        }
        let mut rest = Vec::new();
        for e in &wires_first {
            if !map_e.contains(e) {
                b.add(&mut new_w2, e)?;
                map_e.add(e.clone());
                rest.push(e.clone());
            }
        }
        // check overlapping edges for second face
        if degenerate_wire(&rest, face_ref)? {
            return Ok(false);
        }
        // check overlapping edges for first face
        if degenerate_wire(&edges_of(&topods::wire(&new_w1)?), face_ref)? {
            return Ok(false);
        }
        let (new_w1, new_w2) = (topods::wire(&new_w1)?, topods::wire(&new_w2)?);
        self.two_faces(f, face_ref, wfirst, &new_w1, &new_w2)?;
        Ok(true)
    }

    /// The faces on the two new wires, the other wires of the face given to the face
    /// holding them, and the wire descendants updated.
    fn two_faces(
        &mut self,
        f: &Face,
        face_ref: &Face,
        wfirst: &Wire,
        new_w1: &Wire,
        new_w2: &Wire,
    ) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let mut new_f1 = topods::face(&face_ref.empty_copied()?)?;
        new_f1.set_orientation(Orientation::Forward);
        let mut new_f2 = topods::face(&face_ref.empty_copied()?)?;
        new_f2.set_orientation(Orientation::Forward);
        // modifs JAG 97.05.28
        b.add(&mut new_f1, new_w1)?;
        b.add(&mut new_f2, new_w2)?;
        for x in Explorer::new(
            &face_ref.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            let wir = topods::wire(&x)?;
            if wir.is_same(wfirst) {
                continue;
            }
            if is_inside(&new_f1, &wir)? {
                b.add(&mut new_f1, &wir)?;
            } else if is_inside(&new_f2, &wir)? {
                b.add(&mut new_f2, &wir)?;
            }
            // Ce wire est ni dans newF2 ni dans newF1: the native code warns only.
        }
        let lf = self.list_mut(f)?;
        lf.append(new_f1.into_shape());
        lf.append(new_f2.into_shape());
        // Mise a jour des descendants des wires
        for x in Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let mut ls = to_vec(self.list_of(&x)?);
            if let Some(k) = ls.iter().position(|s| s.is_same(wfirst)) {
                // on a trouve le wire
                ls.remove(k);
                ls.push(new_w1.as_ref().clone());
                ls.push(new_w2.as_ref().clone());
                *self.list_mut(&x)? = to_list(ls);
            }
        }
        Ok(())
    }

    /// One face in place of the descendant: the wires of the two ends merged with the
    /// open wire in both orientations (on ne cree qu`une seule face).
    fn merge_wires(
        &mut self,
        w: &Wire,
        f: &Face,
        face_ref: &Face,
        ends: EndWires,
    ) -> Result<bool, Failure> {
        let b = BRepBuilder::default();
        let outer_w = brep_tools::outer_wire(face_ref)?;
        let mut new_wire = Wire::default();
        b.make_wire(&mut new_wire);
        let mut new_wire = new_wire.into_shape();
        let or_relat = if ends.wfirst.orientation() == ends.wlast.orientation() {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        let (mut wfirst, mut wlast) = (ends.wfirst, ends.wlast);
        if wlast.is_same(&outer_w) {
            wlast = wfirst;
            wfirst = outer_w;
        }
        // Edges de wfirst
        for e in Explorer::new(
            &wfirst.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            b.add(&mut new_wire, &e)?;
        }
        // Edges de wlast
        for e in Explorer::new(
            &wlast.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            let orient = top_abs::compose(e.orientation(), or_relat);
            b.add(&mut new_wire, &e.oriented(orient))?;
        }
        // Edges du wire ajoute, et dans les 2 sens
        for e in Explorer::new(
            &w.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            ShapeEnum::Shape,
        )? {
            let orient = e.orientation();
            b.add(&mut new_wire, &e.oriented(orient))?;
            b.add(&mut new_wire, &e.oriented(top_abs::reverse(orient)))?;
            self.dbl_e.add(e.oriented(orient));
        }
        // on refait une face
        let mut new_face = face_ref.empty_copied()?;
        for x in Explorer::new(
            &face_ref.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )? {
            if x.is_same(&wfirst) {
                b.add(&mut new_face, &new_wire.oriented(x.orientation()))?;
            } else if !x.is_same(&wlast) {
                b.add(&mut new_face, &x)?;
            }
        }
        self.list_mut(f)?.append(new_face);
        // Mise a jour des descendants des wires
        for x in Explorer::new(f, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let mut ls = to_vec(self.list_of(&x)?);
            let before = ls.len();
            ls.retain(|s| !s.is_same(&wfirst) && !s.is_same(&wlast));
            if ls.len() != before {
                ls.push(new_wire.clone());
                *self.list_mut(&x)? = to_list(ls);
            }
        }
        Ok(true)
    }
}
