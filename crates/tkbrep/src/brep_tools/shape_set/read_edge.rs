// Rust translation of the edge case of BRepTools_ShapeSet::ReadGeometry(T, IS, S)
// (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading an edge record: its tolerance, flags and curve representations.
use super::BRepToolsShapeSet;
use super::read_shape::read_int;
use super::regularity::read_regularity;
use crate::topods::Edge;
use crate::toptools::{ShapeSet, format_version};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::get_real;
use rustycad_tkmath::gp::Point2;

/// The `p-curve(s) regularity surface location first last [UV points]` of records 2
/// and 3.
struct PCurveRecord {
    closed: bool,
    pcurve: i32,
    pcurve2: i32,
    regularity: rustycad_tkmath::geom_abs::Shape,
    surface: i32,
    location: i32,
    range: (f64, f64),
    uv: Option<(Point2, Point2)>,
}

impl BRepToolsShapeSet {
    /// `tolerance same-parameter same-range degenerated`, then the representation
    /// records until a kind of 0 (or a stream failure, where the native loop would not
    /// end).
    pub(super) fn read_edge(&mut self, stream: &mut IoStream) -> Result<Edge, Failure> {
        let mut edge = Edge::default();
        self.builder.make_edge(&mut edge)?;
        let tolerance = get_real(stream);
        let mut flag = 0;
        read_int(stream, &mut flag);
        self.builder.same_parameter(&edge, flag == 1)?;
        read_int(stream, &mut flag);
        self.builder.same_range(&edge, flag == 1)?;
        read_int(stream, &mut flag);
        self.builder.degenerated(&edge, flag == 1)?;
        let mut kind = 0;
        loop {
            read_int(stream, &mut kind);
            self.read_curve_representation(stream, &edge, kind, tolerance)?;
            if kind <= 0 || stream.fail() {
                return Ok(edge);
            }
        }
    }

    /// One representation record of kind 1 to 7; another kind reads nothing more.
    fn read_curve_representation(
        &self,
        stream: &mut IoStream,
        edge: &Edge,
        kind: i32,
        tolerance: f64,
    ) -> Result<(), Failure> {
        match kind {
            1 => self.read_curve3d(stream, edge, tolerance),
            2 | 3 => {
                let record = self.read_pcurve_record(stream, kind == 3);
                self.apply_pcurve_record(edge, &record, tolerance)
            }
            4 => self.read_regularity_record(stream, edge),
            5 => self.read_polygon3d_record(stream, edge),
            6 | 7 => self.read_polygon_on_triangulation_record(stream, edge, kind == 7),
            _ => Ok(()),
        }
    }

    /// `curve location first last`: the 3D curve and its range (only the 3D range),
    /// both skipped for a missing curve.
    fn read_curve3d(
        &self,
        stream: &mut IoStream,
        edge: &Edge,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let (mut c, mut l) = (0, 0);
        read_int(stream, &mut c);
        read_int(stream, &mut l);
        let curve = self.curves.curve(c);
        if !curve.is_null() {
            let location = self.locations().location(l);
            self.builder
                .update_edge_located_curve(edge, &curve, &location, tolerance)?;
        }
        let (first, last) = (get_real(stream), get_real(stream));
        if !curve.is_null() {
            self.builder.range(edge, first, last, true)?;
        }
        Ok(())
    }

    /// The fields of records 2 and 3; the UV points are stored only in version 2.
    fn read_pcurve_record(&self, stream: &mut IoStream, closed: bool) -> PCurveRecord {
        let mut record = PCurveRecord {
            closed,
            pcurve: 0,
            pcurve2: 0,
            regularity: rustycad_tkmath::geom_abs::Shape::C0,
            surface: 0,
            location: 0,
            range: (0.0, 0.0),
            uv: None,
        };
        read_int(stream, &mut record.pcurve);
        if closed {
            read_int(stream, &mut record.pcurve2);
            record.regularity = read_regularity(stream);
        }
        read_int(stream, &mut record.surface);
        read_int(stream, &mut record.location);
        record.range = (get_real(stream), get_real(stream));
        if self.format_nb() == format_version::VERSION_2 {
            let (fx, fy, lx, ly) = (
                get_real(stream),
                get_real(stream),
                get_real(stream),
                get_real(stream),
            );
            record.uv = Some((Point2::new(fx, fy), Point2::new(lx, ly)));
        }
        record
    }

    /// The p-curve (or the seam p-curves with their regularity) and its range; skipped
    /// when a p-curve or the surface is missing. Without stored UV points (versions 1
    /// and 3) the builder computes them.
    fn apply_pcurve_record(
        &self,
        edge: &Edge,
        record: &PCurveRecord,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let pcurve = self.curves2d.curve2d(record.pcurve);
        let pcurve2 = self.curves2d.curve2d(record.pcurve2);
        let surface = self.surfaces.surface(record.surface);
        if pcurve.is_null() || (record.closed && pcurve2.is_null()) || surface.is_null() {
            return Ok(());
        }
        let location = self.locations().location(record.location);
        if record.closed {
            self.builder.update_edge_pcurves(
                edge,
                (&pcurve, &pcurve2),
                &surface,
                &location,
                tolerance,
                record.uv,
            )?;
            self.builder.continuity(
                edge,
                (&surface, &surface),
                (&location, &location),
                record.regularity,
            )?;
        } else {
            self.builder
                .update_edge_pcurve(edge, &pcurve, &surface, &location, tolerance, record.uv)?;
        }
        let (first, last) = record.range;
        self.builder
            .range_on_surface(edge, &surface, &location, first, last)
    }

    /// `regularity surface location surface2 location2`, skipped for a missing surface.
    fn read_regularity_record(&self, stream: &mut IoStream, edge: &Edge) -> Result<(), Failure> {
        let regularity = read_regularity(stream);
        let (mut s, mut l, mut s2, mut l2) = (0, 0, 0, 0);
        for value in [&mut s, &mut l, &mut s2, &mut l2] {
            read_int(stream, value);
        }
        let (surface, surface2) = (self.surfaces.surface(s), self.surfaces.surface(s2));
        if surface.is_null() || surface2.is_null() {
            return Ok(());
        }
        let (location, location2) = (self.locations().location(l), self.locations().location(l2));
        self.builder.continuity(
            edge,
            (&surface, &surface2),
            (&location, &location2),
            regularity,
        )
    }

    /// `polygon location`, skipped for an index out of the polygon table.
    fn read_polygon3d_record(&self, stream: &mut IoStream, edge: &Edge) -> Result<(), Failure> {
        let (mut c, mut l) = (0, 0);
        read_int(stream, &mut c);
        read_int(stream, &mut l);
        if c > 0 && c <= self.polygons3d.extent() {
            let polygon = self.polygons3d.get(c)?;
            self.builder
                .update_edge_polygon3d(edge, &polygon, &self.locations().location(l))?;
        }
        Ok(())
    }

    /// `nodes [nodes2] triangulation location`, skipped for an index out of its table.
    fn read_polygon_on_triangulation_record(
        &self,
        stream: &mut IoStream,
        edge: &Edge,
        closed: bool,
    ) -> Result<(), Failure> {
        let (mut pt, mut pt2, mut t, mut l) = (0, 0, 0, 0);
        read_int(stream, &mut pt);
        if closed {
            read_int(stream, &mut pt2);
        }
        read_int(stream, &mut t);
        read_int(stream, &mut l);
        let nodes_count = self.nodes.extent();
        let in_range = |index: i32| index > 0 && index <= nodes_count;
        let valid = t > 0
            && t <= self.triangulations.extent()
            && in_range(pt)
            && (!closed || in_range(pt2));
        if !valid {
            return Ok(());
        }
        let (triangulation, _) = self.triangulations.get(t)?;
        let location = self.locations().location(l);
        let nodes = self.nodes.get(pt)?;
        if closed {
            let nodes2 = self.nodes.get(pt2)?;
            return self.builder.update_edge_polygons_on_triangulation(
                edge,
                (&nodes, &nodes2),
                &triangulation,
                &location,
            );
        }
        self.builder
            .update_edge_polygon_on_triangulation(edge, &nodes, &triangulation, &location)
    }
}
