// Rust translation of the static helper FindFromVEdge
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The second part of the new bound of a face at a corner: the section edges between
//! the new vertices of the bound edge (pairs found apart first, then the continuous
//! walk), else the splits of the bound edge.
use super::ds_queries::{find_next_vertex, find_vertex, get_pave, remove_edges, split_between};
use super::from_u_edge::BoundSearch;
use rustycad_tkbo::bop_ds::Pave;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Compound, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::Point2;

/// The pairs of paves joined by a chain of section edges in the first pass.
struct FoundPairs {
    first_paves: Vec<Pave>,
    last_paves: Vec<Pave>,
    flags: Vec<i32>,
    lists: Vec<List<Shape>>,
    pair_counter: i32,
}

impl FoundPairs {
    /// The empty pairs with one list slot per pave of the bound edge.
    fn new(nbpave: usize) -> Self {
        Self {
            first_paves: Vec::new(),
            last_paves: Vec::new(),
            flags: Vec::new(),
            lists: (0..nbpave).map(|_| List::default()).collect(),
            pair_counter: 1,
        }
    }

    /// `anArrayOfListOfSec(apaircounter++).Append(theList)` and the pair recorded;
    /// Standard_OutOfRange past the pave count.
    fn push(
        &mut self,
        mut chain: List<Shape>,
        (first, last): (Pave, Pave),
        flag: i32,
    ) -> Result<(), Failure> {
        let slot: usize = usize::try_from(self.pair_counter - 1).unwrap_or(usize::MAX);
        let nb: usize = self.lists.len();
        let Some(list) = self.lists.get_mut(slot) else {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "BRepFill_TrimShellCorner FindFromVEdge: pair {}; expected 1..={nb}",
                    self.pair_counter
                ),
                "NCollection_Array1",
            ));
        };
        list.append_list(&mut chain);
        self.pair_counter += 1;
        self.first_paves.push(first);
        self.last_paves.push(last);
        self.flags.push(flag);
        Ok(())
    }

    /// The 0-based index of the first pair holding the paves `(a_pave1, a_pave2)`: it
    /// starts at or before `a_pave1` and (flag 1) ends at or after `a_pave2`, or (flag
    /// 0) satisfies `ends_on_u_edge`.
    fn holding(
        &self,
        a_pave1: &Pave,
        a_pave2: &Pave,
        ends_on_u_edge: &dyn Fn(&Pave) -> Result<bool, Failure>,
    ) -> Result<Option<usize>, Failure> {
        for (k, ((fp, lp), flag)) in self
            .first_paves
            .iter()
            .zip(&self.last_paves)
            .zip(&self.flags)
            .enumerate()
        {
            let bfin: bool = a_pave1.is_equal(fp) || a_pave1.parameter() > fp.parameter();
            let blin: bool = if *flag != 0 {
                a_pave2.is_equal(lp) || a_pave2.parameter() < lp.parameter()
            } else {
                ends_on_u_edge(lp)?
            };
            if bfin && blin {
                return Ok(Some(k));
            }
        }
        Ok(None)
    }

    /// The chain of a pair moved out of its slot (`Append` empties the source list).
    fn take(&mut self, k: usize, list: &mut List<Shape>) {
        if let Some(chain) = self.lists.get_mut(k) {
            list.append_list(chain);
        }
    }
}

/// The walk state of FindFromVEdge.
struct VEdgeWalk<'s, 'a> {
    search: &'s BoundSearch<'a>,
    comp: Compound,
    sec_found: bool,
    nbpave: i32,
}

/// A position on the bound edge: the pave, its vertex and its point on the face.
#[derive(Clone)]
struct BoundPos {
    pave: Pave,
    vertex: Vertex,
    point: Point2,
}

impl VEdgeWalk<'_, '_> {
    /// `FillGap` in the remaining section edges; the chain found is taken out of them.
    fn fill_and_take(
        &mut self,
        ends: (&Vertex, &Vertex),
        points: (Point2, Point2),
    ) -> Result<Option<List<Shape>>, Failure> {
        let Some(ordered) = self.search.fill(ends, points, &self.comp)? else {
            return Ok(None);
        };
        self.comp = remove_edges(&self.comp, &ordered)?;
        self.sec_found = true;
        Ok(Some(ordered))
    }

    /// The start position: the previous pave, or the first pave of the bound edge when
    /// the previous chain ended on the u-edge; `None` when the bound edge has no pave.
    fn start(&self, the_prev_pave: &Pave, is_on_u_edge: bool) -> Result<Option<BoundPos>, Failure> {
        let pave: Pave = if is_on_u_edge {
            let Some(first) = get_pave(self.search.bound_edge_index, true, self.search.ds)? else {
                return Ok(None);
            };
            first
        } else {
            *the_prev_pave
        };
        let point: Point2 = self.bound_point(&pave)?;
        let vertex: Vertex = topods::vertex(self.search.ds.shape(pave.index())?)?;
        Ok(Some(BoundPos {
            pave,
            vertex,
            point,
        }))
    }

    /// The point of a pave on the bound edge.
    fn bound_point(&self, pave: &Pave) -> Result<Point2, Failure> {
        self.search
            .point_on(self.search.bound_edge, pave.parameter())
    }

    /// The cut end of the second u-edge, with its pave and point; `None` when it has no
    /// new edge.
    fn u_edge_end(&self) -> Result<Option<BoundPos>, Failure> {
        let mut pave = Pave::default();
        let Some(vertex) = find_vertex(
            self.search.ue2_old,
            self.search.rank,
            self.search.ds,
            self.search.hist,
            &mut pave,
        )?
        else {
            return Ok(None);
        };
        let point: Point2 = self
            .search
            .point_on(self.search.ue2_old, pave.parameter())?;
        Ok(Some(BoundPos {
            pave,
            vertex,
            point,
        }))
    }

    /// The split of the bound edge between two paves, appended when there is one.
    fn append_split(
        &self,
        (a_pave1, a_pave2): (&Pave, &Pave),
        list: &mut List<Shape>,
    ) -> Result<(), Failure> {
        if let Some(a_e_split) = split_between(
            self.search.ds,
            self.search.bound_edge_index,
            (a_pave1, a_pave2),
        )? {
            list.append(a_e_split);
        }
        Ok(())
    }

    /// The flag-0 end test of the continuous walk: the pave is the (positive) end pave
    /// of the pair and their points on the second u-edge are within the vertex
    /// tolerance.
    fn ends_near(&self, a_pave2: &Pave, p2: Point2, lp: &Pave) -> Result<bool, Failure> {
        if a_pave2.index() != lp.index() || a_pave2.index() <= 0 {
            return Ok(false);
        }
        let p3: Point2 = self.search.point_on(self.search.ue2_old, lp.parameter())?;
        let a_v: Vertex = topods::vertex(self.search.ds.shape(a_pave2.index())?)?;
        let a_bas = BRepAdaptorSurface::with_face(&self.search.face_f, false)?;
        let a_tolerance: f64 = BRepTool::vertex_tolerance(&a_v)?;
        let (utol, vtol) = (
            a_bas.u_resolution(a_tolerance)?,
            a_bas.v_resolution(a_tolerance)?,
        );
        Ok(p2.distance(p3) < if utol > vtol { utol } else { vtol })
    }

    /// One round of the first pass from `first`: the chains between consecutive new
    /// vertices of the bound edge (the point of the start stays the round's one), then
    /// a chain to the cut end of the second u-edge; true when that last chain is found.
    fn pairs_round(
        &mut self,
        first: &mut BoundPos,
        pairs: &mut FoundPairs,
    ) -> Result<(bool, bool), Failure> {
        let (mut a_pave1, p1, mut a_prev_vertex) = (first.pave, first.point, first.vertex.clone());
        let (mut bfound, mut pit) = (false, 0);
        while let Some((a_next_vertex, a_pave2)) =
            find_next_vertex(self.search.bound_edge_index, &a_pave1, self.search.ds)?
        {
            if pit >= self.nbpave {
                break;
            }
            let p2: Point2 = self.bound_point(&a_pave2)?;
            if let Some(ordered) = self.fill_and_take((&a_prev_vertex, &a_next_vertex), (p1, p2))? {
                pairs.push(ordered, (first.pave, a_pave2), 1)?;
                *first = BoundPos {
                    pave: a_pave2,
                    vertex: a_next_vertex.clone(),
                    point: p2,
                };
                a_prev_vertex = a_next_vertex;
                bfound = true;
            }
            a_pave1 = a_pave2;
            pit += 1;
        }
        if let Some(end) = self.u_edge_end()?
            && let Some(ordered) =
                self.fill_and_take((&a_prev_vertex, &end.vertex), (p1, end.point))?
        {
            pairs.push(ordered, (first.pave, end.pave), 0)?;
            return Ok((bfound, true));
        }
        Ok((bfound, false))
    }

    /// The first pass ("by pairs non continuously"): the chains found between the new
    /// vertices of the bound edge, the start moving to the next new vertex after a
    /// round without chain.
    fn pairs_pass(&mut self, start: &BoundPos) -> Result<FoundPairs, Failure> {
        let mut pairs = FoundPairs::new(usize::try_from(self.nbpave).unwrap_or(0));
        let mut first: BoundPos = start.clone();
        for _ in 0..self.nbpave {
            let (bfound, ended) = self.pairs_round(&mut first, &mut pairs)?;
            if ended {
                break;
            }
            if !bfound {
                let Some((a_next_vertex, a_pave2)) =
                    find_next_vertex(self.search.bound_edge_index, &first.pave, self.search.ds)?
                else {
                    break;
                };
                first = BoundPos {
                    pave: a_pave2,
                    point: self.bound_point(&a_pave2)?,
                    vertex: a_next_vertex,
                };
            }
        }
        Ok(pairs)
    }

    /// One step of the continuous walk from `prev` to the next pave: the chain of the
    /// first pass holding the step, else a new chain, else the split of the bound edge
    /// (up to its last pave at the u-edge end); false when the bound edge has no pave.
    fn walk_step(
        &mut self,
        prev: &BoundPos,
        next: (&Vertex, &Pave, Point2),
        pairs: &mut FoundPairs,
        list: &mut List<Shape>,
        at_u_edge: bool,
    ) -> Result<bool, Failure> {
        let (a_next_vertex, a_pave2, p2) = next;
        let ends_test = |lp: &Pave| -> Result<bool, Failure> {
            if at_u_edge {
                Ok(true)
            } else {
                self.ends_near(a_pave2, p2, lp)
            }
        };
        if let Some(k) = pairs.holding(&prev.pave, a_pave2, &ends_test)? {
            pairs.take(k, list);
            return Ok(true);
        }
        if let Some(mut ordered) =
            self.fill_and_take((&prev.vertex, a_next_vertex), (prev.point, p2))?
        {
            list.append_list(&mut ordered);
            return Ok(true);
        }
        if !at_u_edge {
            self.append_split((&prev.pave, a_pave2), list)?;
            return Ok(true);
        }
        let Some(last) = get_pave(self.search.bound_edge_index, false, self.search.ds)? else {
            return Ok(false);
        };
        self.append_split((&prev.pave, &last), list)?;
        Ok(true)
    }

    /// The second pass ("by pairs continuously"): the walk over the new vertices of the
    /// bound edge from `prev`, then to the cut end of the second u-edge (or the split up
    /// to the last pave); false when the bound edge has no last pave.
    fn continuous_pass(
        &mut self,
        mut prev: BoundPos,
        pairs: &mut FoundPairs,
        list: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let mut pit = 0;
        while let Some((a_next_vertex, a_pave2)) =
            find_next_vertex(self.search.bound_edge_index, &prev.pave, self.search.ds)?
        {
            if pit >= self.nbpave {
                break;
            }
            let p2: Point2 = self.bound_point(&a_pave2)?;
            if !self.walk_step(&prev, (&a_next_vertex, &a_pave2, p2), pairs, list, false)? {
                return Ok(false);
            }
            prev = BoundPos {
                pave: a_pave2,
                vertex: a_next_vertex,
                point: p2,
            };
            pit += 1;
        }
        if let Some(end) = self.u_edge_end()? {
            return self.walk_step(
                &prev,
                (&end.vertex, &end.pave, end.point),
                pairs,
                list,
                true,
            );
        }
        let Some(last) = get_pave(self.search.bound_edge_index, false, self.search.ds)? else {
            return Ok(false);
        };
        self.append_split((&prev.pave, &last), list)?;
        Ok(true)
    }
}

/// `FindFromVEdge(thePrevPave, isOnUEdge, theUE1Old, theUE2Old, theFace, theSecEdges,
/// theRank, theBoundEdge, theBoundEdgeIndex, theDS, theHistMap, theListOfWireEdges,
/// isSectionFound)`: the edges of the new bound from the previous pave (or the first
/// pave of the bound edge) and whether a section chain was found; `None` when the bound
/// edge has no pave to start or end at. (The native p-curve of the first u-edge is
/// read and never used.)
pub(super) fn find_from_v_edge(
    search: &BoundSearch,
    the_prev_pave: &Pave,
    is_on_u_edge: bool,
    the_sec_edges: &Compound,
) -> Result<Option<(List<Shape>, bool)>, Failure> {
    if is_on_u_edge {
        let mut a_pave_of_e2 = Pave::default();
        if find_vertex(
            search.ue2_old,
            search.rank,
            search.ds,
            search.hist,
            &mut a_pave_of_e2,
        )?
        .is_some()
            && the_prev_pave.is_equal(&a_pave_of_e2)
        {
            return Ok(Some((List::default(), false)));
        }
    }
    let mut walk = VEdgeWalk {
        search,
        comp: the_sec_edges.clone(),
        sec_found: false,
        nbpave: 0,
    };
    let Some(start) = walk.start(the_prev_pave, is_on_u_edge)? else {
        return Ok(None);
    };
    walk.nbpave = search.ds.paves(search.bound_edge_index)?.len() as i32;
    let mut pairs: FoundPairs = walk.pairs_pass(&start)?;
    let Some(prev) = walk.start(the_prev_pave, is_on_u_edge)? else {
        return Ok(None);
    };
    let mut a_list_of_wire_edges: List<Shape> = List::default();
    if !walk.continuous_pass(prev, &mut pairs, &mut a_list_of_wire_edges)? {
        return Ok(None);
    }
    Ok(Some((a_list_of_wire_edges, walk.sec_found)))
}
