//! The periodic part of `IntUnifyFaces` (ShapeUpgrade_UnifySameDomain.cxx): the seams of
//! the faces, the B-spline surface made periodic across a smooth edge with two p-curves
//! (with `ConcatBSplines`), and the reference surface moved to a new origin of a periodic
//! direction so that the merged face needs no seam.
use super::face_bounds::{CoordBounds, compute_min_edge_size, find_coord_bounds};
use super::face_groups::{FaceGroup, GlobalMaps};
use super::geometry::{is_uiso, null_pcurve, null_surface, update_boundaries};
use super::relocate::{Relocation, RelocationMaps, relocate_pcurves_to_new_uorigin};
use super::{EdgePCurveMap, UnifyState};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_convert::ApproxSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib;

/// The 2D data of the boundary the wire building uses.
pub(super) struct MergeFrame {
    pub coord_tol: f64,
    pub ve_map: IndexedDataMapOfShapeListOfShape,
    pub edges_map: MapOfShape,
    pub u_seam: bool,
    pub v_seam: bool,
}

/// The vertices of the boundary edges with their edges.
pub(super) fn vertex_edge_map(
    edges: &[Shape],
) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    for edge in edges {
        topexp::map_shapes_and_unique_ancestors(
            edge,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut map,
            false,
        )?;
    }
    Ok(map)
}

/// The U and V seams of the outer wires of the faces, and the last edge closed on a face
/// without being really closed (an edge with two p-curves).
fn find_seams(faces: &[Shape]) -> Result<(bool, bool, Edge), Failure> {
    let (mut u_seam, mut v_seam, mut two_pcurves) = (false, false, Edge::default());
    for shape in faces {
        let face = topods::face(shape)?;
        let outer = brep_tools::outer_wire(&face)?;
        for child in ShapeIterator::new(&outer, true, true) {
            let edge = topods::edge(&child)?;
            if !BRepTool::is_closed_on_face(&edge, &face)? {
                continue;
            }
            if !brep_tools::is_really_closed(&edge, &face)? {
                two_pcurves = edge;
            } else if is_uiso(&edge, &face)? {
                u_seam = true;
            } else {
                v_seam = true;
            }
        }
    }
    Ok((u_seam, v_seam, two_pcurves))
}

/// The reference face rebuilt on `surface` (forward, identity location), the p-curves of
/// the edges moved to it from the old one (the temporary ones removed from the old one).
fn rebuild_ref_face(
    group: &mut FaceGroup,
    surface: &Handle<dyn GeomSurface>,
    new_pcurve: impl Fn(&Edge, &Face) -> Result<Handle<dyn Geom2dCurve>, Failure>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let old = std::mem::take(&mut group.ref_face);
    builder.make_face_on_located_surface(&mut group.ref_face, surface, &Location::default(), 0.)?;
    for shape in &group.edges {
        let edge = topods::edge(shape)?;
        let pcurve = new_pcurve(&edge, &old)?;
        if group.temporary.contains(shape) {
            builder.update_edge_pcurve_on_face(&edge, &Handle::default(), &old, 0.)?;
        }
        builder.update_edge_pcurve_on_face(&edge, &pcurve, &group.ref_face, 0.)?;
    }
    Ok(())
}

impl UnifyState {
    /// The seams, the periodic conversion and the relocation of the merged area before
    /// its wires are built; `map_ef` maps the edges of the faces to the faces.
    pub(super) fn prepare_periodic(
        &self,
        group: &mut FaceGroup,
        map_ef: &IndexedDataMapOfShapeListOfShape,
        maps: GlobalMaps<'_>,
    ) -> Result<MergeFrame, Failure> {
        let mut edges_map = MapOfShape::default();
        let min_size =
            compute_min_edge_size(&group.edges, &group.forward_ref_face(), &mut edges_map)?;
        let coord_tol = (min_size / 10.).max(precision::confusion());
        let ve_map = vertex_edge_map(&group.edges)?;
        let (u_seam, v_seam, two_pcurves) = find_seams(&group.faces)?;
        let mut frame = MergeFrame {
            coord_tol,
            ve_map,
            edges_map,
            u_seam,
            v_seam,
        };
        if self.concat_bsplines && !two_pcurves.is_null() && !(u_seam || v_seam) {
            self.make_periodic_across(group, &two_pcurves, maps)?;
        }
        self.relocate_origin(group, map_ef, &mut frame)?;
        set_closed_periods(group)?;
        Ok(frame)
    }

    /// The B-spline basis made periodic across a G1 edge with two p-curves on the
    /// reference face, in the direction the two p-curves differ.
    fn make_periodic_across(
        &self,
        group: &mut FaceGroup,
        edge: &Edge,
        maps: GlobalMaps<'_>,
    ) -> Result<(), Failure> {
        let faces = maps.edge_faces.find_from_key(edge.as_ref())?;
        let face1 = topods::face(faces.first().ok_or_else(null_surface)?)?;
        let face2 = topods::face(faces.last().ok_or_else(null_surface)?)?;
        let continuity = brep_lib::continuity_of_faces(edge, &face1, &face2, self.ang_tol)?;
        if continuity < Continuity::G1 {
            return Ok(());
        }
        let ref_face = group.forward_ref_face();
        let first = BRepTool::curve_on_face(edge, &ref_face)?;
        let mut reversed = edge.clone();
        reversed.reverse();
        let second = BRepTool::curve_on_face(&reversed, &ref_face)?;
        let p1 = first
            .pcurve
            .get()
            .ok_or_else(null_pcurve)?
            .value(second.first)?;
        let p2 = second
            .pcurve
            .get()
            .ok_or_else(null_pcurve)?
            .value(second.first)?;
        let u_closed = (p1.x() - p2.x()).abs() > (p1.y() - p2.y()).abs();
        let (make_u, make_v) = (
            u_closed && group.u_period == 0.,
            !u_closed && group.v_period == 0.,
        );
        if make_u || make_v {
            self.make_periodic(group, make_u, make_v)?;
        }
        Ok(())
    }

    fn make_periodic(
        &self,
        group: &mut FaceGroup,
        make_u: bool,
        make_v: bool,
    ) -> Result<(), Failure> {
        let mut spline = group.base_surface.cast::<GeomBSplineSurface>();
        if spline.is_null() {
            let approx = ApproxSurface::from_surface(
                &group.base_surface,
                1.0e-4,
                (Continuity::C1, Continuity::C1),
                (14, 14),
                16,
                1,
            )?;
            spline = approx.surface();
        }
        let surface = spline.get().ok_or_else(null_surface)?;
        if make_u {
            surface.set_u_periodic()?;
            group.u_period = surface.u_period()?;
        }
        if make_v {
            surface.set_v_periodic()?;
            group.v_period = surface.v_period()?;
        }
        let spline: Handle<dyn GeomSurface> = spline.cast();
        if spline == group.base_surface {
            return Ok(());
        }
        rebuild_ref_face(group, &spline, |edge, old| {
            Ok(BRepTool::curve_on_face(edge, old)?.pcurve)
        })
    }

    /// "Perform relocating to new U-origin": for each periodic direction without seam,
    /// a seam when the faces span the period, else with two intervals the p-curves
    /// relocated and the reference surface trimmed to the new origin when it fits.
    fn relocate_origin(
        &self,
        group: &mut FaceGroup,
        map_ef: &IndexedDataMapOfShapeListOfShape,
        frame: &mut MergeFrame,
    ) -> Result<(), Failure> {
        let periods = [group.u_period, group.v_period];
        let mut seams = [frame.u_seam, frame.v_seam];
        let surface = group.base_surface.get().ok_or_else(null_surface)?;
        let (umin, _, vmin, _) = surface.bounds();
        let surf_min = [umin, vmin];
        // F_RefFace is not refreshed inside the loop natively.
        let ref_face = group.forward_ref_face();
        for ii in 0..2 {
            if periods[ii] == 0. || seams[ii] {
                continue;
            }
            let index = ii as i32 + 1;
            let Some(bounds) = find_coord_bounds(
                &group.faces,
                &ref_face,
                map_ef,
                &frame.edges_map,
                (index, periods[ii]),
            )?
            else {
                break;
            };
            if bounds.max - bounds.min > periods[ii] - 1.0e-5 {
                seams[ii] = true;
                continue;
            }
            if bounds.nb_intervals != 2 {
                continue;
            }
            let direction = PeriodicDirection {
                index,
                period: periods[ii],
                surf_min: surf_min[ii],
            };
            if !self.relocate_direction(group, &ref_face, &bounds, direction, frame)? {
                break;
            }
        }
        (frame.u_seam, frame.v_seam) = (seams[0], seams[1]);
        Ok(())
    }

    /// One relocation; false for the native `break` out of the direction loop.
    fn relocate_direction(
        &self,
        group: &mut FaceGroup,
        ref_face: &Face,
        bounds: &CoordBounds,
        d: PeriodicDirection,
        frame: &mut MergeFrame,
    ) -> Result<bool, Failure> {
        let mut used = MapOfShape::default();
        let mut new_pcurves = EdgePCurveMap::default();
        let first_face = group
            .faces
            .get(bounds.face_max.wrapping_sub(1))
            .cloned()
            .ok_or_else(|| face_index_error(bounds.face_max))?;
        let relocation = Relocation {
            ref_face,
            coord_tol: frame.coord_tol,
            index: d.index,
            period: d.period,
        };
        let maps = RelocationMaps {
            ve_map: &mut frame.ve_map,
            edge_new_pcurve: &mut new_pcurves,
            used_edges: &mut used,
        };
        if !relocate_pcurves_to_new_uorigin(&group.edges, &first_face, relocation, maps)? {
            frame.ve_map = vertex_edge_map(&group.edges)?;
            return Ok(false);
        }
        if !unused_pcurves(&group.edges, ref_face, &used, &mut new_pcurves)? {
            return Ok(false);
        }
        frame.ve_map = vertex_edge_map(&group.edges)?;
        let (new_min, new_max) = new_range(&group.edges, &new_pcurves, d.index)?;
        if fits_without_seam((new_min, new_max), d.period, frame.coord_tol) {
            self.trim_to_new_origin(group, (new_min, new_max), d, &new_pcurves)?;
        }
        Ok(true)
    }

    /// The reference face rebuilt on the basis trimmed to the new origin (or on the
    /// basis itself when the origin is its bound) with the relocated p-curves.
    fn trim_to_new_origin(
        &self,
        group: &mut FaceGroup,
        (new_min, new_max): (f64, f64),
        d: PeriodicDirection,
        new_pcurves: &EdgePCurveMap,
    ) -> Result<(), Failure> {
        let rest = d.period - (new_max - new_min);
        let mut origin = new_min - rest / 2.;
        if origin < d.surf_min {
            origin = d.surf_min;
        }
        let surface = if origin == d.surf_min {
            group.base_surface.clone()
        } else {
            let trimmed = GeomRectangularTrimmedSurface::with_one_direction(
                &group.base_surface,
                origin,
                origin + d.period,
                d.index == 1,
                true,
            )?;
            Handle::new(trimmed).cast()
        };
        rebuild_ref_face(group, &surface, |edge, _| {
            Ok(new_pcurves.find(edge.as_ref())?.clone())
        })
    }
}

/// A periodic direction of the basis surface.
#[derive(Clone, Copy)]
struct PeriodicDirection {
    index: i32,
    period: f64,
    surf_min: f64,
}

/// "PCurves from unused edges (may be degenerated edges)": false when one is missing
/// (the map is then cleared) or the map does not cover every edge.
fn unused_pcurves(
    edges: &[Shape],
    ref_face: &Face,
    used: &MapOfShape,
    map: &mut EdgePCurveMap,
) -> Result<bool, Failure> {
    for shape in edges {
        if used.contains(shape) {
            continue;
        }
        let found = BRepTool::curve_on_face(&topods::edge(shape)?, ref_face)?;
        if found.pcurve.is_null() {
            map.clear(false);
            break;
        }
        let trimmed = Geom2dTrimmedCurve::new(&found.pcurve, found.first, found.last, true, true)?;
        map.bind(shape.clone(), Handle::new(trimmed).cast());
    }
    Ok(map.extent() == edges.len())
}

/// The coordinate range of the relocated p-curves.
fn new_range(edges: &[Shape], map: &EdgePCurveMap, index: i32) -> Result<(f64, f64), Failure> {
    let (mut min, mut max) = (f64::MAX, f64::MIN);
    for shape in edges {
        let pcurve = map.find(shape)?;
        let curve = pcurve.get().ok_or_else(null_pcurve)?;
        let range = (curve.first_parameter(), curve.last_parameter());
        update_boundaries(pcurve, range, index, (&mut min, &mut max))?;
    }
    Ok((min, max))
}

/// Whether the relocated range fits in a period and leaves `[0, period]`.
fn fits_without_seam((min, max): (f64, f64), period: f64, coord_tol: f64) -> bool {
    let conf = precision::confusion();
    max - min < period - coord_tol
        && (-conf >= min || min >= period + conf || -conf >= max || max >= period + conf)
}

/// "Set the periods for closed non-periodic surface".
fn set_closed_periods(group: &mut FaceGroup) -> Result<(), Failure> {
    if group.u_period != 0. && group.v_period != 0. {
        return Ok(());
    }
    let (surface, _) = BRepTool::surface_with_location(&group.ref_face)?;
    let surface = super::geometry::clear_rts(&surface);
    let surface = surface.get().ok_or_else(null_surface)?;
    let (ufirst, ulast, vfirst, vlast) = surface.bounds();
    if group.u_period == 0. && surface.is_u_closed()? {
        group.u_period = ulast - ufirst;
    }
    if group.v_period == 0. && surface.is_v_closed()? {
        group.v_period = vlast - vfirst;
    }
    Ok(())
}

fn face_index_error(index: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "ShapeUpgrade_UnifySameDomain: face index {index}; expected a face of the merged sequence"
        ),
        "",
    )
}
