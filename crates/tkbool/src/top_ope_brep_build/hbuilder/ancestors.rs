//! The ancestors of the section edges of `TopOpeBRepBuild_HBuilder`: the edges of the
//! structure they split, the faces of those edges, the curves they lie on and the
//! points of the new vertices.
use super::{HBuilderState, TopOpeBRepBuildHBuilder};
use crate::top_ope_brep_build::builder::curve_indices;
use crate::top_ope_brep_ds::{TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{DataMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The section edges and DS edges/faces of one rank.
type FacesOfEdges = DataMap<i32, List<i32>>;

/// `F` appended to the faces of the edge `gi`.
fn add_face(m: &mut FacesOfEdges, gi: i32, fi: i32) -> Result<(), Failure> {
    if !m.is_bound(&gi) {
        m.bind(gi, List::default());
    }
    m.change_find(&gi)?.append(fi);
    Ok(())
}

/// The faces of `extra` not in `base`, appended to it.
fn merged_faces(mut base: List<i32>, extra: &List<i32>) -> List<i32> {
    let mut moi: Map<i32> = Map::default();
    for &i in base.iter() {
        moi.add(i);
    }
    for &i in extra.iter() {
        if moi.add(i) {
            base.append(i);
        }
    }
    base
}

impl HBuilderState {
    /// The ON splits of the edges of the structure by rank, as section edges of their
    /// DS edge.
    fn map_section_edges(&mut self) -> Result<(), Failure> {
        let hds = self.builder.data_structure();
        let h = hds.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_HBuilder",
            )
        })?;
        let ds = h.ds();
        for (sha_spl, losos) in self.builder.my_split_on.items() {
            let ei = ds.shape_index(sha_spl, true);
            let m = match ds.ancestor_rank(sha_spl) {
                1 => &mut self.my_sect_edge_ds_edges1,
                2 => &mut self.my_sect_edge_ds_edges2,
                _ => continue,
            };
            for sec_edg in losos.list_on_state().iter() {
                if !m.is_bound(sec_edg) {
                    m.bind(sec_edg.clone(), ei);
                }
            }
        }
        Ok(())
    }

    /// `MakeEdgeAncestorMap()` (once): the DS edges of the section edges and the DS
    /// faces with an EDGE interference of each DS edge, by rank.
    pub(crate) fn make_edge_ancestor_map(&mut self) -> Result<(), Failure> {
        if self.my_make_edge_ancestor_is_done {
            return Ok(());
        }
        self.my_sect_edge_ds_edges1.clear(true);
        self.my_sect_edge_ds_edges2.clear(true);
        self.my_ds_edges_ds_faces1.clear(true);
        self.my_ds_edges_ds_faces2.clear(true);
        self.my_make_edge_ancestor_is_done = true;
        self.map_section_edges()?;
        let hds = self.builder.data_structure();
        let h = hds.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_HBuilder",
            )
        })?;
        let ds = h.ds();
        for fi in 1..=ds.nb_shapes() {
            let fds = ds.shape(fi, true).clone();
            if fds.is_null() || fds.shape_type()? != ShapeEnum::Face {
                continue;
            }
            for i in ds.shape_interferences(&fds, true).iter() {
                let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
                let Some(s) = ssi.get() else { continue };
                if s.geometry_type() != TopOpeBRepDSKind::Edge {
                    continue;
                }
                match ds.ancestor_rank(&fds) {
                    1 => add_face(&mut self.my_ds_edges_ds_faces1, s.geometry(), fi)?,
                    2 => add_face(&mut self.my_ds_edges_ds_faces2, s.geometry(), fi)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// `MakeCurveAncestorMap()` (once): the DS curve of each new edge of the kept curves.
    pub(crate) fn make_curve_ancestor_map(&mut self) -> Result<(), Failure> {
        if self.my_make_curve_ancestor_is_done {
            return Ok(());
        }
        self.my_sect_edge_ds_curve.clear(true);
        self.my_make_curve_ancestor_is_done = true;
        let hds = self.builder.data_structure();
        let h = hds.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_HBuilder",
            )
        })?;
        for ic in curve_indices(h, true) {
            for e in self.builder.my_new_edges.find(&ic)?.clone().iter() {
                self.my_sect_edge_ds_curve.bind(e.clone(), ic);
            }
        }
        Ok(())
    }

    /// `GetDSFaceFromDSEdge(indexEdg, rank)`, copied.
    fn ds_faces_of_ds_edge(&mut self, index_edg: i32, rank: i32) -> Result<List<i32>, Failure> {
        self.make_edge_ancestor_map()?;
        let m = match rank {
            1 => &self.my_ds_edges_ds_faces1,
            2 => &self.my_ds_edges_ds_faces2,
            _ => return Ok(self.my_empty_integer_list.clone()),
        };
        Ok(m.find(&index_edg)
            .cloned()
            .unwrap_or_else(|_| self.my_empty_integer_list.clone()))
    }

    /// `GetDSEdgeFromSectEdge(E, rank)`.
    fn ds_edge_of_sect_edge(&mut self, e: &Shape, rank: i32) -> Result<i32, Failure> {
        self.make_edge_ancestor_map()?;
        let m = match rank {
            1 => &self.my_sect_edge_ds_edges1,
            2 => &self.my_sect_edge_ds_edges2,
            _ => return Ok(0),
        };
        Ok(m.find(e).copied().unwrap_or(0))
    }

    /// `GetDSFaceFromDSCurve(indexCur, rank)`.
    fn ds_face_of_ds_curve(&mut self, index_cur: i32, rank: i32) -> Result<i32, Failure> {
        self.make_curve_ancestor_map()?;
        let hds = self.builder.data_structure();
        let h = hds.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_HBuilder",
            )
        })?;
        let ds = h.ds();
        Ok(match rank {
            1 => ds.shape_index(ds.curve(index_cur).shape1(), true),
            2 => ds.shape_index(ds.curve(index_cur).shape2(), true),
            _ => 0,
        })
    }

    /// `GetDSCurveFromSectEdge(SectEdge)`.
    fn ds_curve_of_sect_edge(&mut self, e: &Shape) -> Result<i32, Failure> {
        self.make_curve_ancestor_map()?;
        Ok(self.my_sect_edge_ds_curve.find(e).copied().unwrap_or(0))
    }

    /// The DS shape of an index.
    fn ds_shape(&self, i: i32) -> Result<Shape, Failure> {
        let hds = self.builder.data_structure();
        let h = hds.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_HBuilder",
            )
        })?;
        Ok(h.ds().shape(i, true).clone())
    }
}

impl TopOpeBRepBuildHBuilder {
    /// `MakeEdgeAncestorMap()`; `hb.make_edge_ancestor_map()?`.
    pub fn make_edge_ancestor_map(&self) -> Result<(), Failure> {
        self.my_state.write().make_edge_ancestor_map()
    }

    /// `MakeCurveAncestorMap()`; `hb.make_curve_ancestor_map()?`.
    pub fn make_curve_ancestor_map(&self) -> Result<(), Failure> {
        self.my_state.write().make_curve_ancestor_map()
    }

    /// `GetDSEdgeFromSectEdge(E, rank)`: the DS edge split by the section edge on the
    /// rank (0 when none); `hb.get_ds_edge_from_sect_edge(&e, 1)?`.
    pub fn get_ds_edge_from_sect_edge(&self, e: &Shape, rank: i32) -> Result<i32, Failure> {
        self.my_state.write().ds_edge_of_sect_edge(e, rank)
    }

    /// `GetDSFaceFromDSEdge(indexEdg, rank)`: the DS faces of the rank with an EDGE
    /// interference of the edge, copied (empty when none);
    /// `hb.get_ds_face_from_ds_edge(3, 1)?`.
    pub fn get_ds_face_from_ds_edge(
        &self,
        index_edg: i32,
        rank: i32,
    ) -> Result<List<i32>, Failure> {
        self.my_state.write().ds_faces_of_ds_edge(index_edg, rank)
    }

    /// `GetDSCurveFromSectEdge(SectEdge)`: the DS curve of the new edge (0 when none);
    /// `hb.get_ds_curve_from_sect_edge(&e)?`.
    pub fn get_ds_curve_from_sect_edge(&self, sect_edge: &Shape) -> Result<i32, Failure> {
        self.my_state.write().ds_curve_of_sect_edge(sect_edge)
    }

    /// `GetDSFaceFromDSCurve(indexCur, rank)`: the DS index of the first (second) face
    /// of the curve; `hb.get_ds_face_from_ds_curve(1, 1)?`.
    pub fn get_ds_face_from_ds_curve(&self, index_cur: i32, rank: i32) -> Result<i32, Failure> {
        self.my_state.write().ds_face_of_ds_curve(index_cur, rank)
    }

    /// `GetDSPointFromNewVertex(NewVert)`: the DS point of the new vertex (0 when
    /// none), the map made once; `hb.get_ds_point_from_new_vertex(&v)?`.
    pub fn get_ds_point_from_new_vertex(&self, new_vert: &Shape) -> Result<i32, Failure> {
        let mut st = self.my_state.write();
        if !st.my_make_point_ancestor_is_done {
            st.my_make_point_ancestor_is_done = true;
            let hds = st.builder.data_structure();
            let nb = hds.get().map_or(0, |h| h.ds().nb_points());
            for i in 1..=nb {
                let v = st.builder.new_vertex(i)?.clone();
                if !v.is_null() {
                    st.my_new_vertex_ds_point.bind(v, i);
                }
            }
        }
        Ok(st
            .my_new_vertex_ds_point
            .find(new_vert)
            .copied()
            .unwrap_or(0))
    }

    /// `EdgeCurveAncestors(E, F1, F2, IC)`: the faces and the DS curve of a new edge
    /// (None when it is of no curve); `hb.edge_curve_ancestors(&e)?`.
    pub fn edge_curve_ancestors(&self, e: &Shape) -> Result<Option<(Shape, Shape, i32)>, Failure> {
        let mut st = self.my_state.write();
        let ic = st.ds_curve_of_sect_edge(e)?;
        if ic == 0 {
            return Ok(None);
        }
        let (i_f1, i_f2) = (
            st.ds_face_of_ds_curve(ic, 1)?,
            st.ds_face_of_ds_curve(ic, 2)?,
        );
        Ok(Some((st.ds_shape(i_f1)?, st.ds_shape(i_f2)?, ic)))
    }

    /// `EdgeSectionAncestors(E, LF1, LF2, LE1, LE2)`: the faces of each rank of the DS
    /// edges split by the section edge (merged when both ranks have one) and those
    /// edges; None for a non-edge, else the lists and whether faces of both ranks and
    /// an edge were found; `hb.edge_section_ancestors(&e)?`.
    #[allow(clippy::type_complexity)]
    pub fn edge_section_ancestors(
        &self,
        e: &Shape,
    ) -> Result<Option<([List<Shape>; 4], bool)>, Failure> {
        if e.shape_type()? != ShapeEnum::Edge {
            return Ok(None);
        }
        let mut st = self.my_state.write();
        let (ie1, ie2) = (
            st.ds_edge_of_sect_edge(e, 1)?,
            st.ds_edge_of_sect_edge(e, 2)?,
        );
        let (mut f1, mut f2) = (List::default(), List::default());
        if ie1 != 0 && ie2 != 0 {
            f1 = merged_faces(
                st.ds_faces_of_ds_edge(ie1, 1)?,
                &st.ds_faces_of_ds_edge(ie2, 1)?,
            );
            f2 = merged_faces(
                st.ds_faces_of_ds_edge(ie1, 2)?,
                &st.ds_faces_of_ds_edge(ie2, 2)?,
            );
        } else if ie1 != 0 || ie2 != 0 {
            let ie = if ie1 != 0 { ie1 } else { ie2 };
            f1 = st.ds_faces_of_ds_edge(ie, 1)?;
            f2 = st.ds_faces_of_ds_edge(ie, 2)?;
        }
        let mut lists: [List<Shape>; 4] = Default::default();
        for &i in f1.iter() {
            lists[0].append(st.ds_shape(i)?);
        }
        for &i in f2.iter() {
            lists[1].append(st.ds_shape(i)?);
        }
        if ie1 != 0 {
            lists[2].append(st.ds_shape(ie1)?);
        }
        if ie2 != 0 {
            lists[3].append(st.ds_shape(ie2)?);
        }
        let r = !lists[0].is_empty()
            && !lists[1].is_empty()
            && (!lists[2].is_empty() || !lists[3].is_empty());
        Ok(Some((lists, r)))
    }
}
