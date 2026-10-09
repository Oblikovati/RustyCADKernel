// Rust translation of BRepFill_CompatibleWires::SameNumberByPolarMethod (first part)
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SameNumberByPolarMethod`: closed sections cut so that they have the same number
//! of edges, by carrying their vertices from plane to plane and cutting the next
//! section where the ray from its center meets it.
use super::BRepFillCompatibleWires;
use super::helpers::{
    EdgeNewEdges, VertexLists, nth, plane_of_wire, search_root, search_vertex, seq_of_vertices,
};
use super::intersect::{edge_intersect_on_wire, transform};
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_check::{BRepCheckStatus, BRepCheckWire};

/// The state of the polar method: the planes of the sections (center and normal, the
/// origin and a null vector when a section has no plane), the range of the
/// non-punctual sections and the cuts made so far.
pub(super) struct Polar {
    pub(super) pos: Vec<Point3>,
    pub(super) axe: Vec<Vector3>,
    pub(super) ideb: usize,
    pub(super) ifin: usize,
    pub(super) nb_max_v: usize,
    pub(super) with_rotation: bool,
    pub(super) percent: f64,
    pub(super) edge_new_edges: EdgeNewEdges,
}

impl Polar {
    /// The plane of section `i` (1-based).
    pub(super) fn plane(&self, i: usize) -> (Point3, Vector3) {
        (self.pos[i - 1], self.axe[i - 1])
    }
}

/// Whether the vertex counts as a ray end (`Pnew` away from the center of the plane).
fn away_from(pnew: Point3, center: Point3) -> bool {
    pnew.distance(&center) > precision::confusion()
}

impl BRepFillCompatibleWires {
    /// `SameNumberByPolarMethod(WithRotation)`.
    pub(super) fn same_number_by_polar_method(
        &mut self,
        with_rotation: bool,
    ) -> Result<(), Failure> {
        if !self.all_closed_by_check()? {
            self.my_status = ThruSectionErrorStatus::NotSameTopology;
            return Ok(());
        }
        let nb_sects: usize = self.my_work.len();
        let ideb: usize = if self.my_degen1 { 2 } else { 1 };
        let ifin: usize = if self.my_degen2 {
            nb_sects - 1
        } else {
            nb_sects
        };
        let v_closed: bool = !self.my_degen1
            && !self.my_degen2
            && self.my_work[ideb - 1].is_same(&self.my_work[ifin - 1]);
        self.remove_degenerated_edges(ideb, ifin)?;
        let mut polar = self.section_planes(ideb, ifin, with_rotation)?;
        let mut rmap = VertexLists::default();
        if !self.report_backwards(&mut polar, &mut rmap)? {
            return Ok(());
        }
        let mut map_vlv: VertexLists = self.vertex_chains(&rmap, ideb)?;
        let size_map: usize = seq_of_vertices(&self.my_work[ideb - 1])?.len();
        if !self.report_forwards(&mut polar, &mut map_vlv, size_map)? {
            return Ok(());
        }
        if !self.regularize(&mut polar, &map_vlv, v_closed)? {
            return Ok(());
        }
        self.finish_polar(&mut polar, v_closed)
    }

    /// Every section is closed for BRepCheck_Wire.
    fn all_closed_by_check(&self) -> Result<bool, Failure> {
        let mut all_closed: bool = true;
        for section in &self.my_work {
            let checker = BRepCheckWire::new(&topods::wire(section)?)?;
            all_closed = all_closed && checker.closed(false)? == BRepCheckStatus::NoError;
        }
        Ok(all_closed)
    }

    /// The sections of `ideb..=ifin` rebuilt without their degenerated edges.
    fn remove_degenerated_edges(&mut self, ideb: usize, ifin: usize) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for i in ideb..=ifin {
            let section: Shape = self.my_work[i - 1].clone();
            let mut has_deg_edge: bool = false;
            let mut iterator = ShapeIterator::new(&section, true, true);
            while iterator.more() {
                if BRepTool::degenerated(&topods::edge(iterator.value()?)?)? {
                    has_deg_edge = true;
                    break;
                }
                iterator.next();
            }
            if !has_deg_edge {
                continue;
            }
            let mut new_wire = Wire::default();
            builder.make_wire(&mut new_wire);
            iterator.initialize(&section, true, true);
            while iterator.more() {
                let edge = topods::edge(iterator.value()?)?;
                if !BRepTool::degenerated(&edge)? {
                    builder.add(&mut new_wire, &edge)?;
                }
                iterator.next();
            }
            self.my_work[i - 1] = new_wire.into_shape();
        }
        Ok(())
    }

    /// The planes of the sections; a punctual end takes its vertex and the normal of
    /// its neighbour.
    fn section_planes(
        &self,
        ideb: usize,
        ifin: usize,
        with_rotation: bool,
    ) -> Result<Polar, Failure> {
        let nb_sects: usize = self.my_work.len();
        let mut nb_max_v: usize = 0;
        for section in &self.my_work {
            let mut explorer = WireExplorer::with_wire(&topods::wire(section)?)?;
            while explorer.more() {
                nb_max_v += 1;
                explorer.next()?;
            }
        }
        let mut pos: Vec<Point3> = vec![Point3::default(); nb_sects];
        let mut axe: Vec<Vector3> = vec![Vector3::default(); nb_sects];
        for i in ideb..=ifin {
            if let Some(plane) = plane_of_wire(&topods::wire(&self.my_work[i - 1])?)? {
                pos[i - 1] = plane.location();
                axe[i - 1] = Vector3::from(plane.axis().direction());
            }
        }
        if self.my_degen1 {
            pos[0] = BRepTool::pnt(&topods::vertex(nth(
                &seq_of_vertices(&self.my_work[0])?,
                1,
                "vertex",
            )?)?)?;
            axe[0] = axe[ideb - 1];
        }
        if self.my_degen2 {
            pos[nb_sects - 1] = BRepTool::pnt(&topods::vertex(nth(
                &seq_of_vertices(&self.my_work[nb_sects - 1])?,
                1,
                "vertex",
            )?)?)?;
            axe[nb_sects - 1] = axe[ifin - 1];
        }
        Ok(Polar {
            pos,
            axe,
            ideb,
            ifin,
            nb_max_v,
            with_rotation,
            percent: self.my_percent,
            edge_new_edges: EdgeNewEdges::default(),
        })
    }

    /// The vertices of each section, from the last to the first, reported on the
    /// previous section (`RMap`); false (with the Failed status) when a section has
    /// more vertices than all the edges together.
    fn report_backwards(
        &mut self,
        polar: &mut Polar,
        rmap: &mut VertexLists,
    ) -> Result<bool, Failure> {
        let mut i: usize = polar.ifin;
        while i > polar.ideb {
            let seq_v: Vec<Shape> = seq_of_vertices(&self.my_work[i - 1])?;
            if seq_v.len() > polar.nb_max_v {
                self.my_status = ThruSectionErrorStatus::Failed;
                return Ok(false);
            }
            for vi in &seq_v {
                rmap.bind(vi.clone(), List::default());
                let pi: Point3 = BRepTool::pnt(&topods::vertex(vi)?)?;
                let pnew: Point3 =
                    transform(polar.with_rotation, pi, polar.plane(i), polar.plane(i - 1))?;
                if !away_from(pnew, polar.pos[i - 2]) {
                    continue;
                }
                let target: Wire = topods::wire(&self.my_work[i - 2])?;
                let found = edge_intersect_on_wire(
                    polar.pos[i - 2],
                    pnew,
                    polar.percent,
                    rmap,
                    &target,
                    &mut polar.edge_new_edges,
                )?;
                if found.new_vertex {
                    self.my_work[i - 2] = found.new_wire.map(Wire::into_shape).unwrap_or_default();
                }
                rmap.change_find(vi)?.append(found.vsol.into_shape());
            }
            i -= 1;
        }
        Ok(true)
    }

    /// `MapVLV`: for each vertex of the first section, the chain of its reports
    /// through `RMap` (bounded by the number of sections, for looping sections).
    fn vertex_chains(&self, rmap: &VertexLists, ideb: usize) -> Result<VertexLists, Failure> {
        let mut map_vlv = VertexLists::default();
        for vi in seq_of_vertices(&self.my_work[ideb - 1])? {
            let mut chain: List<Shape> = List::default();
            chain.append(vi.clone());
            let mut nb_v: usize = 1;
            let mut next: Option<Vertex> = search_root(&vi, rmap);
            while let Some(v1) = next {
                chain.append(v1.as_ref().clone());
                nb_v += 1;
                // The test on NbV is required for looping sections.
                next = if v1.is_same(&vi) || nb_v >= self.my_work.len() {
                    None
                } else {
                    search_root(&v1, rmap)
                };
            }
            map_vlv.bind(vi, chain);
        }
        Ok(map_vlv)
    }

    /// The vertices of each section, from the first to the last, reported on the next
    /// section when their chain does not reach it yet.
    fn report_forwards(
        &mut self,
        polar: &mut Polar,
        map_vlv: &mut VertexLists,
        size_map: usize,
    ) -> Result<bool, Failure> {
        for i in polar.ideb..polar.ifin {
            let seq_v: Vec<Shape> = seq_of_vertices(&self.my_work[i - 1])?;
            if seq_v.len() > polar.nb_max_v || seq_v.len() > size_map {
                self.my_status = ThruSectionErrorStatus::Failed;
                return Ok(false);
            }
            for vi in &seq_v {
                self.report_vertex_forward(polar, map_vlv, vi, i)?;
            }
        }
        Ok(true)
    }

    /// One vertex of section `i` reported on section `i + 1`.
    fn report_vertex_forward(
        &mut self,
        polar: &mut Polar,
        map_vlv: &mut VertexLists,
        vi: &Shape,
        i: usize,
    ) -> Result<(), Failure> {
        let root: Option<Vertex> = search_root(vi, map_vlv);
        if let Some(vroot) = &root {
            let lvi: &List<Shape> = map_vlv.find(vroot)?;
            if search_vertex(lvi, &self.my_work[i])?.is_some() {
                return Ok(());
            }
        }
        let pi: Point3 = BRepTool::pnt(&topods::vertex(vi)?)?;
        let pnew: Point3 = transform(polar.with_rotation, pi, polar.plane(i), polar.plane(i + 1))?;
        if !away_from(pnew, polar.pos[i]) {
            return Ok(());
        }
        let target: Wire = topods::wire(&self.my_work[i])?;
        let found = edge_intersect_on_wire(
            polar.pos[i],
            pnew,
            polar.percent,
            map_vlv,
            &target,
            &mut polar.edge_new_edges,
        )?;
        // `MapVLV(VRoot)` with a null root raises Standard_NoSuchObject natively.
        let key: Shape = root.map(Vertex::into_shape).unwrap_or_default();
        map_vlv.change_find(&key)?.append(found.vsol.into_shape());
        if found.new_vertex {
            self.my_work[i] = found.new_wire.map(Wire::into_shape).unwrap_or_default();
        }
        Ok(())
    }
}
