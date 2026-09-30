// Rust translation of the TopAbs_EDGE case of BinTools_ShapeSet::ReadShape
// (BinTools_ShapeSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading the geometry record of one edge: the tolerance, the flags and the curve
//! representations, rebuilt with `BRep_Builder`.
use super::format_version;
use super::raw_enums::continuity_from_raw;
use super::scalars::{get_bool, get_integer, get_real, stream_get};
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use crate::topods::{Edge, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Point2;

/// The fields of a p-curve record (`-2-` or `-3-`) in reading order.
struct PCurveRecord {
    closed: bool,
    pcurve: i32,
    pcurve2: i32,
    continuity: i32,
    surface: i32,
    location: i32,
    range: (f64, f64),
    uv: Option<(Point2, Point2)>,
}

impl BinShapeSet {
    /// The edge record: tolerance, SameParameter, SameRange and Degenerated, then the
    /// representations up to a 0 type. An unknown type raises "Unexpected Curve
    /// Representation =<type>".
    pub(super) fn read_edge(
        &self,
        stream: &mut IoStream,
        shape: &mut Shape,
    ) -> Result<(), Failure> {
        let mut edge = Edge::default();
        self.builder.make_edge(&mut edge)?;
        *shape = edge.clone().into_shape();
        let tolerance = get_real(stream)?;
        self.builder.same_parameter(&edge, get_bool(stream))?;
        self.builder.same_range(&edge, get_bool(stream))?;
        self.builder.degenerated(&edge, get_bool(stream))?;
        loop {
            match stream_get(stream) {
                0 => return Ok(()),
                1 => self.read_curve3d_record(&edge, tolerance, stream)?,
                kind @ (2 | 3) => self.read_pcurve_record(&edge, tolerance, kind == 3, stream)?,
                4 => self.read_regularity_record(&edge, stream)?,
                5 => self.read_polygon3d_record(&edge, stream)?,
                kind @ (6 | 7) => self.read_triangulation_record(&edge, kind == 7, stream)?,
                other => {
                    return Err(Failure::new(&format!(
                        "Unexpected Curve Representation ={other}\n"
                    )));
                }
            }
        }
    }

    /// `-1-`: the curve and location indices, then the range; a null curve (index 0)
    /// consumes the record and changes nothing.
    fn read_curve3d_record(
        &self,
        edge: &Edge,
        tolerance: f64,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let (curve_index, location_index) = (get_integer(stream)?, get_integer(stream)?);
        let curve = self.curves.curve(curve_index)?;
        if !curve.is_null() {
            let location = self.locations.location(location_index)?;
            self.builder
                .update_edge_located_curve(edge, &curve, &location, tolerance)?;
        }
        let (first, last) = (get_real(stream)?, get_real(stream)?);
        if !self.curves.curve(curve_index)?.is_null() {
            self.builder.range(edge, first, last, true)?;
        }
        Ok(())
    }

    fn read_pcurve_fields(
        &self,
        closed: bool,
        stream: &mut IoStream,
    ) -> Result<PCurveRecord, Failure> {
        let pcurve = get_integer(stream)?;
        let (pcurve2, continuity) = if closed {
            (get_integer(stream)?, stream_get(stream))
        } else {
            (0, 0)
        };
        let (surface, location) = (get_integer(stream)?, get_integer(stream)?);
        let range = (get_real(stream)?, get_real(stream)?);
        let uv = if matches!(
            self.format_nb(),
            format_version::VERSION_2 | format_version::VERSION_3
        ) {
            let first = Point2::new(get_real(stream)?, get_real(stream)?);
            Some((first, Point2::new(get_real(stream)?, get_real(stream)?)))
        } else {
            None
        };
        Ok(PCurveRecord {
            closed,
            pcurve,
            pcurve2,
            continuity,
            surface,
            location,
            range,
            uv,
        })
    }

    /// `-2-` or `-3-` (seam): the p-curves, surface, location, range and (versions 2 and
    /// 3) UV points; skipped when a p-curve or the surface index names a null geometry.
    /// A seam also gets the regularity of its surface with itself.
    fn read_pcurve_record(
        &self,
        edge: &Edge,
        tolerance: f64,
        closed: bool,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let record = self.read_pcurve_fields(closed, stream)?;
        let pcurve = self.curves2d.curve2d(record.pcurve)?;
        if pcurve.is_null() || (record.closed && self.curves2d.curve2d(record.pcurve2)?.is_null()) {
            return Ok(());
        }
        let surface = self.surfaces.surface(record.surface)?;
        if surface.is_null() {
            return Ok(());
        }
        let location = self.locations.location(record.location)?;
        if record.closed {
            let pcurve2 = self.curves2d.curve2d(record.pcurve2)?;
            let pcurves = (&pcurve, &pcurve2);
            self.builder
                .update_edge_pcurves(edge, pcurves, &surface, &location, tolerance, record.uv)?;
            let continuity = continuity_from_raw(record.continuity)?;
            self.builder.continuity(
                edge,
                (&surface, &surface),
                (&location, &location),
                continuity,
            )?;
        } else {
            self.builder
                .update_edge_pcurve(edge, &pcurve, &surface, &location, tolerance, record.uv)?;
        }
        let (first, last) = record.range;
        self.builder
            .range_on_surface(edge, &surface, &location, first, last)
    }

    /// `-4-`: the continuity, then two surface and location indices; skipped when a
    /// surface index names a null surface.
    fn read_regularity_record(&self, edge: &Edge, stream: &mut IoStream) -> Result<(), Failure> {
        let continuity = stream_get(stream);
        let (surface1, location1) = (get_integer(stream)?, get_integer(stream)?);
        let (surface2, location2) = (get_integer(stream)?, get_integer(stream)?);
        let first = self.surfaces.surface(surface1)?;
        if first.is_null() {
            return Ok(());
        }
        let second = self.surfaces.surface(surface2)?;
        if second.is_null() {
            return Ok(());
        }
        let locations = (
            &self.locations.location(location1)?,
            &self.locations.location(location2)?,
        );
        let continuity = continuity_from_raw(continuity)?;
        self.builder
            .continuity(edge, (&first, &second), locations, continuity)
    }

    /// `-5-`: the 3D polygon and location indices.
    fn read_polygon3d_record(&self, edge: &Edge, stream: &mut IoStream) -> Result<(), Failure> {
        let (polygon, location) = (get_integer(stream)?, get_integer(stream)?);
        let polygon = self.polygons3d.get(polygon)?;
        self.builder
            .update_edge_polygon3d(edge, &polygon, &self.locations.location(location)?)
    }

    /// `-6-` or `-7-` (two node arrays): the node array indices, then the triangulation
    /// and location indices.
    fn read_triangulation_record(
        &self,
        edge: &Edge,
        closed: bool,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let nodes = get_integer(stream)?;
        let nodes2 = if closed { get_integer(stream)? } else { 0 };
        let (triangulation, location) = (get_integer(stream)?, get_integer(stream)?);
        let polygon = self.nodes.get(nodes)?;
        let (triangulation, _) = self.triangulations.get(triangulation)?;
        let location = self.locations.location(location)?;
        if closed {
            let polygon2 = self.nodes.get(nodes2)?;
            self.builder.update_edge_polygons_on_triangulation(
                edge,
                (&polygon, &polygon2),
                &triangulation,
                &location,
            )
        } else {
            self.builder.update_edge_polygon_on_triangulation(
                edge,
                &polygon,
                &triangulation,
                &location,
            )
        }
    }
}
