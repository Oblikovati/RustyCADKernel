//! `BOPAlgo_PaveFiller_6.cxx`: vertices at the ends of section curves, the filtering of
//! paves put on several curves, and the existing-vertex check of section points.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::{CurveRef, VertexTolerances};
use crate::bop_ds::{Pave, PaveBlockHandle, ShapeInfo};
use crate::bop_tools::{
    compute_vertex_point, d_tolerance, make_new_vertex_point, update_vertex_curve,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, IndexedDataMap, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::{RangeEnd, find_valid_range};

/// Native local `PaveBlockDist`: a section pave block and the vertex's distance to it.
struct PaveBlockDistance {
    block: PaveBlockHandle,
    square_distance: f64,
    sine: f64,
    tolerance: f64,
}

impl BOPAlgoPaveFiller {
    /// True when a vertex On/In both faces lies within `tolerance` of the point
    /// (`IsExistingVertex`).
    pub(super) fn is_existing_vertex(
        &self,
        point: Point3,
        tolerance: f64,
        vertices_on_in: &Map<i32>,
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let check = tolerance + self.options.fuzzy_value();
        let mut point_box = BoundingBox3::default();
        point_box.add_point(point);
        point_box.enlarge(tolerance);
        for vertex in vertices_on_in.iter().copied() {
            let info = structure.shape_info(vertex)?;
            if point_box.is_out_box(info.box_bounds()) {
                continue;
            }
            let shape = Vertex::unchecked(info.shape().clone());
            if compute_vertex_point(&shape, point, check)? == 0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Creates vertices at the free ends of a bounded section curve that lie on both faces
    /// (`PutBoundPaveOnCurve`); returns their indices.
    pub(super) fn put_bound_pave_on_curve(
        &mut self,
        faces: &[Face; 2],
        curve: CurveRef,
    ) -> Result<Vec<i32>, Failure> {
        let mut created = Vec::new();
        let (block, section) = self.curve_first_block(curve)?;
        let (mut t1, mut t2, mut p1, mut p2) = (0., 0., Point3::default(), Point3::default());
        section.bounds(&mut t1, &mut t2, &mut p1, &mut p2)?;
        let stored = self.ff_curve(curve)?;
        let tolerance = stored.tolerance().max(stored.tangential_tolerance());
        let bounds = self.bound_paves(curve)?;
        let closed = p2.is_equal(&p1, precision::confusion());
        if closed && (bounds[0] > 0 || bounds[1] > 0) {
            return Ok(created);
        }
        for (end, (parameter, point)) in [(t1, p1), (t2, p2)].into_iter().enumerate() {
            if bounds[end] >= 0 || (end == 1 && closed) {
                continue;
            }
            if !self
                .context_ref()?
                .is_valid_point_for_faces(point, &faces[0], &faces[1], tolerance)?
            {
                continue;
            }
            let mut vertex = Vertex::default();
            make_new_vertex_point(point, tolerance, &mut vertex)?;
            update_vertex_curve(&section, parameter, &vertex)?;
            let index = self.append_vertex_with_box(&vertex)?;
            block
                .lock()
                .expect("pave-block lock")
                .append_ext_pave(Pave::new(index, parameter));
            created.push(index);
        }
        Ok(created)
    }

    /// Appends a vertex with its box (`BRepBndLib::Add` plus Confusion gap) to the DS.
    pub(super) fn append_vertex_with_box(&mut self, vertex: &Vertex) -> Result<i32, Failure> {
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Vertex);
        info.set_shape(&vertex.clone().into_shape());
        let bounds = info.change_box();
        BRepBndLib::add(vertex, bounds, true)?;
        bounds.set_gap(bounds.gap() + precision::confusion());
        Ok(self.structure_mut()?.append_shape_info(&info))
    }

    /// Removes a vertex from section curves it is far from (compared with the closest
    /// curve), unless it projects at a large angle, then lowers its saved tolerance to the
    /// largest kept distance (`FilterPavesOnCurves`).
    pub(super) fn filter_paves_on_curves(
        &self,
        ff: usize,
        tolerances: &VertexTolerances,
    ) -> Result<(), Failure> {
        let by_vertex = self.pave_distances(ff)?;
        for index in 1..=by_vertex.extent() {
            let vertex = *by_vertex.find_key(index)?;
            let list = by_vertex.find_from_index(index)?;
            let mut minimum = f64::MAX;
            for item in list {
                if item.square_distance < minimum {
                    minimum = item.square_distance;
                }
            }
            let mut kept = -1.0_f64;
            let mut removed = false;
            for item in list {
                let check = 100. * (item.tolerance * item.tolerance).max(minimum);
                if item.square_distance > check && item.sine < 0.5 {
                    item.block
                        .lock()
                        .expect("pave-block lock")
                        .remove_ext_pave(vertex);
                    removed = true;
                } else if item.square_distance > kept {
                    kept = item.square_distance;
                }
            }
            if removed
                && kept > 0.
                && let Some(saved) = tolerances.seek(&vertex)
            {
                let real = saved.max(kept.sqrt() + precision::confusion());
                Self::set_vertex_tolerance(&self.vertex_shape(vertex)?, real)?;
            }
        }
        Ok(())
    }

    fn pave_distances(
        &self,
        ff: usize,
    ) -> Result<IndexedDataMap<i32, Vec<PaveBlockDistance>, DefaultHashPolicy>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut by_vertex: IndexedDataMap<i32, Vec<PaveBlockDistance>, DefaultHashPolicy> =
            IndexedDataMap::with_hasher(DefaultHashPolicy, 1);
        for curve in structure.interf_ff_records()[ff].curves().iter() {
            let tolerance = curve.tolerance().max(curve.tangential_tolerance());
            let adaptor = GeomAdaptorCurve::with_curve(curve.curve().curve())?;
            let block = curve.pave_blocks()[0].clone();
            let paves: Vec<Pave> = block.lock().expect("pave-block lock").ext_paves().to_vec();
            for pave in paves {
                let point =
                    BRepTool::pnt(&Vertex::unchecked(structure.shape(pave.index())?.clone()))?;
                let (on_curve, derivative) = adaptor.d1(pave.parameter())?;
                let projection = Vector3::between(point, on_curve);
                let square_distance = projection.square_magnitude();
                let derivative_square = derivative.square_magnitude();
                let mut sine = projection.cross_square_magnitude(derivative);
                if square_distance > gp::resolution() && derivative_square > gp::resolution() {
                    sine = (sine / square_distance / derivative_square).sqrt();
                }
                let item = PaveBlockDistance {
                    block: block.clone(),
                    square_distance,
                    sine,
                    tolerance,
                };
                if let Some(list) = by_vertex.change_seek(&pave.index()) {
                    list.push(item);
                } else {
                    by_vertex.add(pave.index(), vec![item]);
                }
            }
        }
        Ok(by_vertex)
    }

    /// Adds the closing pave of a section curve whose end vertex also covers its other end,
    /// when a valid range remains (`PutClosingPaveOnCurve`).
    pub(super) fn put_closing_pave_on_curve(&mut self, curve: CurveRef) -> Result<(), Failure> {
        let (block, section) = self.curve_first_block(curve)?;
        if section.curve().is_null() || !section.has_bounds() {
            return Ok(());
        }
        let (mut t, mut p) = ([0.; 2], [Point3::default(); 2]);
        {
            let (first, rest) = t.split_at_mut(1);
            let (p_first, p_rest) = p.split_at_mut(1);
            section.bounds(&mut first[0], &mut rest[0], &mut p_first[0], &mut p_rest[0])?;
        }
        let Some((mut vertex, opposite, opposite_point)) = end_pave(&block, t, p) else {
            return Ok(());
        };
        let shape = self.vertex_shape(vertex)?;
        let vertex_tolerance = BRepTool::vertex_tolerance(&shape)?;
        let stored = self.ff_curve(curve)?;
        let point_tolerance =
            stored.tolerance().max(stored.tangential_tolerance()) + precision::confusion();
        let distance = BRepTool::pnt(&shape)?.distance(&opposite_point);
        if distance > vertex_tolerance + point_tolerance {
            return Ok(());
        }
        let new_tolerance = vertex_tolerance.max(distance + d_tolerance());
        let adaptor = GeomAdaptorCurve::with_curve(section.curve())?;
        let ends = [
            RangeEnd {
                parameter: t[0],
                point: p[0],
                tolerance: new_tolerance,
            },
            RangeEnd {
                parameter: t[1],
                point: p[1],
                tolerance: new_tolerance,
            },
        ];
        if !find_valid_range(&adaptor, section.tolerance(), ends[0], ends[1])?.is_valid {
            return Ok(());
        }
        if new_tolerance > vertex_tolerance {
            vertex = self.update_vertex(vertex, new_tolerance)?;
        }
        block
            .lock()
            .expect("pave-block lock")
            .change_ext_paves()
            .push(Pave::new(vertex, opposite));
        Ok(())
    }

    /// Whether the vertex lies on the section within its own tolerance (`EstimatePaveOnCurve`).
    pub(super) fn estimate_pave_on_curve(
        &self,
        vertex: i32,
        curve: CurveRef,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let section = self.ff_curve(curve)?.curve().clone();
        let mut parameter = 0.;
        self.context_ref()?.is_vertex_on_line(
            &self.vertex_shape(vertex)?,
            &section,
            tolerance,
            &mut parameter,
        )
    }
}

/// The first extra pave at a curve end: its vertex, the opposite end parameter and point.
fn end_pave(block: &PaveBlockHandle, t: [f64; 2], p: [Point3; 2]) -> Option<(i32, f64, Point3)> {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    for pave in locked.ext_paves() {
        for end in 0..2 {
            if (pave.parameter() - t[end]).abs() < precision::p_confusion() {
                return Some((pave.index(), t[1 - end], p[1 - end]));
            }
        }
    }
    None
}
