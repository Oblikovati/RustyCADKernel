// Rust translation of BRepCheck_Wire::Orientation and its helpers (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Wire::Orientation`: the chain of the oriented edges from vertex to
//! vertex, choosing in 2d (`ChoixUV`) among several candidates on a face.
use super::errors::out_of_range;
use super::explore::{is_oriented, sub_shapes};
use super::status::BRepCheckStatus;
use super::wire::BRepCheckWire;
use super::wire_choix_uv::choix_uv;
use super::wire_closed::record;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The state of the chain walk of `Orientation`.
struct Chain {
    vf: Vertex,
    vl: Vertex,
    map_s: MapOfShape,
    the_edge: Edge,
    the_ref: Edge,
    // `ortmp`: the stored orientation of the last edge found in `map_s`.
    ortmp: Orientation,
}

/// What one step of the walk decided.
enum Step {
    /// Go on with the next index (the index change is applied by the caller).
    Continue,
    /// Leave `Orientation` with this status.
    Leave(BRepCheckStatus),
}

impl Chain {
    /// The pivot of the walk: VL when set, else VF.
    fn pivot(&self) -> Option<&Vertex> {
        if !self.vl.is_null() {
            return Some(&self.vl);
        }
        if !self.vf.is_null() {
            return Some(&self.vf);
        }
        None
    }

    /// Whether `edge` holds the pivot with the orientation that continues the chain
    /// (FORWARD on VL, REVERSED on VF).
    fn starts_at_pivot(&self, edge: &Shape) -> Result<bool, Failure> {
        let (wanted, pivot) = if self.vl.is_null() {
            (Orientation::Reversed, &self.vf)
        } else {
            (Orientation::Forward, &self.vl)
        };
        Ok(sub_shapes(edge, ShapeEnum::Vertex)?
            .iter()
            .any(|v| v.orientation() == wanted && pivot.is_same(v)))
    }

    /// The candidates to continue the chain among the edges at the pivot.
    fn candidates(&mut self, edges: &List<Shape>, face: &Face) -> Result<List<Shape>, Failure> {
        let mut ledge: List<Shape> = List::default();
        for edg in edges {
            let orient = edg.orientation();
            let contained = self.map_s.contains(edg);
            if contained {
                self.ortmp = get_orientation(&self.map_s, edg);
            }
            // The native list of already passed outgoing edges is filled but never read.
            // In 2d (a face) or for an edge that is not degenerated.
            if (!contained || self.ortmp != orient)
                && self.starts_at_pivot(edg)?
                && (!face.is_null() || !BRepTool::degenerated(&Edge::unchecked(edg.clone()))?)
            {
                ledge.append(edg.clone());
            }
        }
        Ok(ledge)
    }

    /// Moves the pivot to the other end of the chosen edge (VL to its REVERSED vertex,
    /// VF to its FORWARD one), nullified when there is none.
    fn advance(&mut self, chosen: &Shape) -> Result<(), Failure> {
        let going_forward = !self.vl.is_null();
        let wanted = if going_forward {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let next = sub_shapes(chosen, ShapeEnum::Vertex)?
            .into_iter()
            .find(|v| v.orientation() == wanted);
        let slot = if going_forward {
            &mut self.vl
        } else {
            &mut self.vf
        };
        match next {
            Some(v) => *slot = Vertex::unchecked(v),
            None => slot.nullify(),
        }
        self.map_s.add(chosen.clone());
        self.the_edge = Edge::unchecked(chosen.clone());
        Ok(())
    }
}

/// `GetOrientation(mapE, edg)`: the orientation stored in `map_e` for `edge`.
fn get_orientation(map_e: &MapOfShape, edge: &Shape) -> Orientation {
    map_e
        .contained(edge)
        .map_or(Orientation::Forward, |stored| stored.orientation())
}

impl BRepCheckWire {
    /// `Orientation(F, Update)`: after `Closed` (a closure defect other than
    /// `NotClosed` is returned as is), walks the oriented edges from vertex to vertex:
    /// `InvalidDegeneratedFlag` for a first edge without oriented vertices,
    /// `BadOrientationOfSubshape` when no or several edges continue the chain (chosen in
    /// 2d on a face), `NotClosed` when the chain cannot close on the face;
    /// `checker.orientation(&face, false)?`.
    pub fn orientation(&self, face: &Face, update: bool) -> Result<BRepCheckStatus, Failure> {
        let closed = self.closed(false)?;
        let list = self.base_own_list()?;
        if closed != BRepCheckStatus::NotClosed && closed != BRepCheckStatus::NoError {
            record(&list, update, closed);
            return Ok(closed);
        }
        let status = self.walk_orientation(face)?;
        record(&list, update, status);
        Ok(status)
    }

    /// The first oriented edge sets the chain; then the walk.
    fn walk_orientation(&self, face: &Face) -> Result<BRepCheckStatus, Failure> {
        let mut chain = Chain {
            vf: Vertex::default(),
            vl: Vertex::default(),
            map_s: MapOfShape::default(),
            the_edge: Edge::default(),
            the_ref: Edge::default(),
            ortmp: Orientation::Forward,
        };
        let first = sub_shapes(&self.wire(), ShapeEnum::Edge)?
            .into_iter()
            .find(is_oriented);
        if let Some(edg) = first {
            chain.map_s.add(edg.clone());
            chain.the_edge = Edge::unchecked(edg.clone());
            chain.the_ref = Edge::unchecked(edg.clone());
            first_vertices(&edg, &mut chain)?;
            if chain.vf.is_null() && chain.vl.is_null() {
                return Ok(BRepCheckStatus::InvalidDegeneratedFlag);
            }
        }
        self.walk_chain(face, &mut chain)
    }

    /// The loop over `Index < nbOriNoDegen`.
    fn walk_chain(&self, face: &Face, chain: &mut Chain) -> Result<BRepCheckStatus, Failure> {
        let (nb_ori_no_degen, cstat) = {
            let fields = self.fields.read();
            (fields.map_ve.extent(), fields.cstat)
        };
        let is_go_fwd: bool = !chain.vl.is_null();
        let mut index: i64 = 1;
        while index < nb_ori_no_degen as i64 {
            let Some(pivot) = chain.pivot().cloned() else {
                return Ok(BRepCheckStatus::InvalidDegeneratedFlag);
            };
            let edges = self.edges_at(&pivot)?;
            let mut ledge = chain.candidates(&edges, face)?;
            let mut changed_direction = false;
            if ledge.is_empty() {
                if cstat != BRepCheckStatus::NotClosed {
                    return Ok(BRepCheckStatus::BadOrientationOfSubshape);
                }
                if chain.vl.is_null() {
                    return Ok(BRepCheckStatus::NoError);
                }
                // No chain on VL: the chain on VF is forced (the index stays).
                index -= 1;
                chain.vl.nullify();
                chain.the_edge = chain.the_ref.clone();
                changed_direction = true;
            } else if !face.is_null() {
                choix_uv(&pivot, &chain.the_edge, face, &mut ledge)?;
            }
            match self.step(chain, &ledge, changed_direction)? {
                Step::Leave(status) => return Ok(status),
                Step::Continue => {}
            }
            if let Step::Leave(status) =
                self.check_closure(chain, face, (index, is_go_fwd, cstat))?
            {
                return Ok(status);
            }
            index += 1;
        }
        Ok(BRepCheckStatus::NoError)
    }

    /// The decision on the candidates: several is a bad orientation, one moves the
    /// pivot, none (without a change of direction) is `NotClosed`.
    fn step(&self, chain: &mut Chain, ledge: &List<Shape>, changed: bool) -> Result<Step, Failure> {
        let count = ledge.extent();
        if count >= 2 {
            return Ok(Step::Leave(BRepCheckStatus::BadOrientationOfSubshape));
        }
        if count == 1 {
            let chosen = ledge.first().expect("one candidate").clone();
            chain.advance(&chosen)?;
            return Ok(Step::Continue);
        }
        if !changed {
            return Ok(Step::Leave(BRepCheckStatus::NotClosed));
        }
        Ok(Step::Continue)
    }

    /// The 2d closure check of the first step (not done in `Closed`): an edge must come
    /// back to the reference vertex of the walk direction.
    fn check_closure(
        &self,
        chain: &Chain,
        face: &Face,
        (index, is_go_fwd, cstat): (i64, bool, BRepCheckStatus),
    ) -> Result<Step, Failure> {
        let reference = if is_go_fwd && !chain.vf.is_null() {
            Some(&chain.vf)
        } else if !is_go_fwd && !chain.vl.is_null() {
            Some(&chain.vl)
        } else {
            None
        };
        let Some(v_ref) = reference else {
            return Ok(Step::Continue);
        };
        if index != 1 || cstat == BRepCheckStatus::NotClosed || face.is_null() {
            return Ok(Step::Continue);
        }
        let mut ledge: List<Shape> = List::default();
        for edg in &self.edges_at(v_ref)? {
            if chain.the_ref.is_same(edg) {
                continue;
            }
            let back = sub_shapes(edg, ShapeEnum::Vertex)?
                .iter()
                .any(|v| v.orientation() == Orientation::Reversed && v_ref.is_same(v));
            if back {
                ledge.append(edg.clone());
            }
        }
        choix_uv(v_ref, &chain.the_ref, face, &mut ledge)?;
        if ledge.is_empty() {
            return Ok(Step::Leave(BRepCheckStatus::NotClosed));
        }
        Ok(Step::Continue)
    }

    /// `myMapVE(myMapVE.FindIndex(vertex))`: Standard_OutOfRange for a vertex that is
    /// not in the map, as the native index 0 raises.
    fn edges_at(&self, vertex: &Vertex) -> Result<List<Shape>, Failure> {
        let fields = self.fields.read();
        let index = fields.map_ve.find_index(vertex);
        if index == 0 {
            return Err(out_of_range(&format!(
                "BRepCheck_Wire::Orientation: vertex index 0 in a map of {} vertices",
                fields.map_ve.extent()
            )));
        }
        Ok(fields.map_ve.find_from_index(index)?.clone())
    }
}

/// VF and VL of the first oriented edge: its first FORWARD and REVERSED vertices.
fn first_vertices(edge: &Shape, chain: &mut Chain) -> Result<(), Failure> {
    for v in sub_shapes(edge, ShapeEnum::Vertex)? {
        match v.orientation() {
            Orientation::Forward => chain.vf = Vertex::unchecked(v),
            Orientation::Reversed => chain.vl = Vertex::unchecked(v),
            _ => {}
        }
        if !chain.vf.is_null() && !chain.vl.is_null() {
            break;
        }
    }
    Ok(())
}
