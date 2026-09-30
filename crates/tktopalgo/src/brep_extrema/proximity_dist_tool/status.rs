// Rust translation of IsNodeOnBorder, IsEdgeOnBorder and DefineStatusProxPnt of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_ProximityDistTool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The border statuses of the proximity points.
use super::{BRepExtremaProxPntStatus, BRepExtremaProximityDistTool};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::tools::PrjStateInTriangle;
use rustycad_tkmath::bvh::{BvhArray3d, BvhVec3d, array};
use rustycad_tkmath::poly::{Connect, Triangulation, mesh_purpose};

/// `NCollection_DynamicArray::operator()` of a shape list: unchecked natively (an index
/// outside the list reads out of bounds); Standard_OutOfRange here.
fn shape_at(list: &[Shape], index: i32) -> Result<&Shape, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|i| list.get(i))
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "BRepExtrema_ProximityDistTool: shape index {index} outside [0, {})",
                list.len()
            ))
        })
}

/// `Border` or `Middle`.
fn border_if(is_border: bool) -> BRepExtremaProxPntStatus {
    if is_border {
        BRepExtremaProxPntStatus::Border
    } else {
        BRepExtremaProxPntStatus::Middle
    }
}

/// The last vertex index of a set, and whether its first and last vertices coincide
/// within Confusion (a closed polygon).
fn closed_polygon(vertices: &BvhArray3d) -> Result<(i32, bool), Failure> {
    let last_idx: i32 = array::size(vertices) - 1;
    let gap: BvhVec3d = *array::value(vertices, 0)? - *array::value(vertices, last_idx)?;
    Ok((last_idx, gap.modulus() < precision::confusion()))
}

/// The triangulation of a face of a shape list.
fn face_triangulation(shape: &Shape) -> Result<Handle<Triangulation>, Failure> {
    let (triangulation, _) = BRepTool::triangulation(&topods::face(shape)?, mesh_purpose::NONE)?;
    Ok(triangulation)
}

impl BRepExtremaProximityDistTool {
    /// `IsNodeOnBorder(theNodeIdx, theTr)`: a triangle around the node (from 1) has a
    /// free link through it; `BRepExtremaProximityDistTool::is_node_on_border(1, &tr)?`.
    pub fn is_node_on_border(node_idx: i32, tr: &Handle<Triangulation>) -> Result<bool, Failure> {
        let triangulation = tr.get().ok_or_else(|| {
            null_object("BRepExtrema_ProximityDistTool::IsNodeOnBorder: a null triangulation")
        })?;
        let mut poly_connect = Connect::new(tr)?;
        poly_connect.initialize(node_idx)?;
        while poly_connect.more() {
            let cont_trg: i32 = poly_connect.value();
            let (n0, n1, n2) = triangulation.triangle(cont_trg)?.get();
            let cont_trg_nodes: [i32; 3] = [n0, n1, n2];
            let (a0, a1, a2) = poly_connect.triangles(cont_trg)?;
            for (j, adj_trg) in [a0, a1, a2].into_iter().enumerate() {
                let k: usize = (j + 1) % 3;
                // A free link of the triangle whose ends are on the border.
                if adj_trg == 0 && (cont_trg_nodes[j] == node_idx || cont_trg_nodes[k] == node_idx)
                {
                    return Ok(true);
                }
            }
            poly_connect.next()?;
        }
        Ok(false)
    }

    /// `IsEdgeOnBorder(theTrgIdx, theFirstEdgeNodeIdx, theSecondEdgeNodeIdx, theTr)`: the
    /// link `(first, second)` (nodes of the triangle from 0) of the triangle (from 1) is
    /// free; `BRepExtremaProximityDistTool::is_edge_on_border(1, 0, 1, &tr)?`.
    pub fn is_edge_on_border(
        trg_idx: i32,
        first_edge_node_idx: i32,
        second_edge_node_idx: i32,
        tr: &Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        let poly_connect = Connect::new(tr)?;
        let (a0, a1, a2) = poly_connect.triangles(trg_idx)?;
        for (j, adj_trg) in [a0, a1, a2].into_iter().enumerate() {
            let k: i32 = (j as i32 + 1) % 3;
            if adj_trg == 0 && j as i32 == first_edge_node_idx && k == second_edge_node_idx {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `DefineStatusProxPnt()` (protected): the statuses of the two proximity points;
    /// `tool.define_status_prox_pnt()?`.
    pub fn define_status_prox_pnt(&mut self) -> Result<(), Failure> {
        self.define_status_prox_pnt1()?;
        self.define_status_prox_pnt2()
    }

    /// `defineStatusProxPnt1`: an additional vertex keeps its status; a vertex of an edge
    /// polygon is on the border at the ends of an open polygon; a vertex of a face is on
    /// the border when its node is.
    fn define_status_prox_pnt1(&mut self) -> Result<(), Failure> {
        if self.my_is_prox_vtx1_from_add_set {
            let status = usize::try_from(self.my_prox_vtx_idx1)
                .ok()
                .and_then(|i| self.my_add_status1.get(i));
            self.my_pnt_status1 = *status.ok_or_else(|| {
                Failure::out_of_range("BRepExtrema_ProximityDistTool: additional status index")
            })?;
            return Ok(());
        }
        let face_id1: i32 = self.my_set1.get_shape_id_of_vtx(self.my_prox_vtx_idx1)?;
        let shape1: &Shape = shape_at(&self.my_shape_list1, face_id1)?;
        match shape1.shape_type()? {
            ShapeEnum::Edge => {
                let (last_idx, closed) = closed_polygon(&self.my_set1.vertices())?;
                self.my_pnt_status1 = match closed {
                    true => BRepExtremaProxPntStatus::Middle,
                    false => {
                        border_if(self.my_prox_vtx_idx1 == 0 || self.my_prox_vtx_idx1 == last_idx)
                    }
                };
            }
            ShapeEnum::Face => {
                let node_idx: i32 = self.my_set1.get_vtx_idx_in_shape(self.my_prox_vtx_idx1)? + 1;
                let tr = face_triangulation(shape1)?;
                self.my_pnt_status1 = border_if(Self::is_node_on_border(node_idx, &tr)?);
            }
            _ => {}
        }
        Ok(())
    }

    /// `defineStatusProxPnt2`: from the projection state of the proximity point on the
    /// second set (an inner projection on an edge segment leaves the status unchanged).
    fn define_status_prox_pnt2(&mut self) -> Result<(), Failure> {
        let trg_idx: i32 = self.my_prox_prj_state.get_trg_idx();
        let face_id2: i32 = self.my_set2.get_face_id(trg_idx)?;
        let shape2: Shape = shape_at(&self.my_shape_list2, face_id2)?.clone();
        match shape2.shape_type()? {
            ShapeEnum::Edge => self.edge_status2(trg_idx),
            ShapeEnum::Face => self.face_status2(trg_idx, &shape2),
            _ => Ok(()),
        }
    }

    /// The edge case of `defineStatusProxPnt2`: a projection on a polygon end (of the
    /// whole second set) is on the border unless the polygon is closed; a projection on
    /// a link only when the link is degenerate at an end.
    fn edge_status2(&mut self, trg_idx: i32) -> Result<(), Failure> {
        let state: PrjStateInTriangle = self.my_prox_prj_state.get_prj_state();
        if state == PrjStateInTriangle::Inner {
            return Ok(());
        }
        let (last_idx, closed) = closed_polygon(&self.my_set2.vertices())?;
        if closed {
            self.my_pnt_status2 = BRepExtremaProxPntStatus::Middle;
            return Ok(());
        }
        let vtx_indices: [i32; 3] = self.my_set2.get_vtx_indices(trg_idx)?;
        let first_vtx_idx: i32 = node_of(
            &vtx_indices,
            self.my_prox_prj_state.get_number_of_first_node(),
        )?;
        if state == PrjStateInTriangle::Vertex {
            self.my_pnt_status2 = border_if(first_vtx_idx == 0 || first_vtx_idx == last_idx);
        } else if state == PrjStateInTriangle::Edge {
            let last_vtx_idx: i32 = node_of(
                &vtx_indices,
                self.my_prox_prj_state.get_number_of_last_node(),
            )?;
            // It can be the border only for a projection on a degenerated edge.
            self.my_pnt_status2 = border_if(
                first_vtx_idx == last_vtx_idx && (first_vtx_idx == 0 || first_vtx_idx == last_idx),
            );
        }
        Ok(())
    }

    /// The face case of `defineStatusProxPnt2`: inside a triangle is the middle; a
    /// projection on a node or a link is on the border when the node or the link is.
    fn face_status2(&mut self, trg_idx: i32, face: &Shape) -> Result<(), Failure> {
        let state: PrjStateInTriangle = self.my_prox_prj_state.get_prj_state();
        if state == PrjStateInTriangle::Inner {
            self.my_pnt_status2 = BRepExtremaProxPntStatus::Middle;
            return Ok(());
        }
        let tr = face_triangulation(face)?;
        let vtx_indices: [i32; 3] = self.my_set2.get_vtx_indices(trg_idx)?;
        if state == PrjStateInTriangle::Vertex {
            let node_num: i32 = self.my_prox_prj_state.get_number_of_first_node();
            let node_idx: i32 = self
                .my_set2
                .get_vtx_idx_in_shape(node_of(&vtx_indices, node_num)?)?
                + 1;
            self.my_pnt_status2 = border_if(Self::is_node_on_border(node_idx, &tr)?);
        } else if state == PrjStateInTriangle::Edge {
            let trg_idx_in_shape: i32 = self.my_set2.get_trg_idx_in_shape(trg_idx)? + 1;
            let first: i32 = self.my_prox_prj_state.get_number_of_first_node();
            let last: i32 = self.my_prox_prj_state.get_number_of_last_node();
            self.my_pnt_status2 =
                border_if(Self::is_edge_on_border(trg_idx_in_shape, first, last, &tr)?);
        }
        Ok(())
    }
}

/// `aVtxIndicesOfTrg[theNodeNum]` (an array of bounds 0..2, checked natively).
fn node_of(vtx_indices: &[i32; 3], node_num: i32) -> Result<i32, Failure> {
    usize::try_from(node_num)
        .ok()
        .and_then(|i| vtx_indices.get(i))
        .copied()
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "NCollection_Array1::operator[]: node {node_num} outside [0, 2]"
            ))
        })
}
