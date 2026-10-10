//! The package class `Prs3d`: segment matching, deflection, polyline primitives and the free
//! edges of a triangulation.
use super::Prs3dLineAspect;
use rustycad_tkernel::ncollection::{Array1, HSequence, List, Sequence, Vec3};
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::exceptions::no_such_object;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::poly::{Connect, Triangulation};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPrimitives, Graphic3dArrayOfSegments, Graphic3dGroupMembers as _,
    Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// The package class `Prs3d`.
pub struct Prs3d;

impl Prs3d {
    /// `MatchSegment`: whether `(x, y, z)` projects inside the segment `p1 p2` within the
    /// Manhattan distance `distance`; returns the match and the distance (the native out
    /// argument, untouched (`None`) when the projection falls outside or the segment is null).
    /// Example: `Prs3d::match_segment(0.5, 0.1, 0.0, 0.2, &a, &b)`.
    pub fn match_segment(
        x: f64,
        y: f64,
        z: f64,
        distance: f64,
        p1: &Point3,
        p2: &Point3,
    ) -> (bool, Option<f64>) {
        let (x1, y1, z1) = (p1.x(), p1.y(), p1.z());
        let (x2, y2, z2) = (p2.x(), p2.y(), p2.z());
        let (dx, dy, dz) = (x2 - x1, y2 - y1, z2 - z1);
        let dist = dx * dx + dy * dy + dz * dz;
        if dist == 0.0 {
            return (false, None);
        }
        let lambda = ((x - x1) * dx + (y - y1) * dy + (z - z1) * dz) / dist;
        if !(0.0..=1.0).contains(&lambda) {
            return (false, None);
        }
        let found = (x - x1 - lambda * dx).abs()
            + (y - y1 - lambda * dy).abs()
            + (z - z1 - lambda * dz).abs();
        (found < distance, Some(found))
    }
    /// `GetDeflection` of box corners: the largest extent times the coefficient times 4, at
    /// least `Precision::Confusion`. Example: `Prs3d::get_deflection(&min, &max, 0.001)`.
    pub fn get_deflection(
        bnd_min: &Vec3<f64>,
        bnd_max: &Vec3<f64>,
        deviation_coefficient: f64,
    ) -> f64 {
        let diag = *bnd_max - *bnd_min;
        let deflection = diag.max_comp() * deviation_coefficient * 4.0;
        // std::max(a, b) keeps `a` unless `a < b` (a NaN deflection stays NaN).
        if deflection < confusion() {
            confusion()
        } else {
            deflection
        }
    }
    /// `GetDeflection` of a box: the maximal chordial deviation for a void box or an open box
    /// without finite part, else the deflection of the finite part.
    /// Example: `Prs3d::get_deflection_of_box(&bounds, 0.001, 0.1)`.
    pub fn get_deflection_of_box(
        bnd_box: &BoundingBox3,
        deviation_coefficient: f64,
        maximal_chordial_deviation: f64,
    ) -> f64 {
        if bnd_box.is_void() {
            return maximal_chordial_deviation;
        }
        let mut finite = bnd_box.clone();
        if bnd_box.is_open() {
            if !bnd_box.has_finite_part() {
                return maximal_chordial_deviation;
            }
            finite = bnd_box.finite_part();
        }
        let Ok([x0, y0, z0, x1, y1, z1]) = finite.get() else {
            return maximal_chordial_deviation;
        };
        Self::get_deflection(
            &Vec3::new(x0, y0, z0),
            &Vec3::new(x1, y1, z1),
            deviation_coefficient,
        )
    }
    /// `PrimitivesFromPolylines`: one segments array of all polylines, null for no polyline;
    /// an empty polyline raises `Standard_NoSuchObject` from `First()`.
    /// Example: `Prs3d::primitives_from_polylines(&polylines)?`.
    pub fn primitives_from_polylines(
        points: &List<Handle<HSequence<Point3>>>,
    ) -> Result<Handle<Graphic3dArrayOfPrimitives>, Failure> {
        if points.is_empty() {
            return Ok(Handle::default());
        }
        let nb_vertices: i32 = points.iter().map(|p| p.sequence(Sequence::length)).sum();
        let edge_nb = (nb_vertices - points.length()) * 2;
        let segments = Graphic3dArrayOfSegments::new(nb_vertices, edge_nb, 0)?;
        for polyline in points.iter() {
            polyline.sequence(|sequence| add_polyline(&segments, sequence))?;
        }
        Ok(Handle::new(segments).cast())
    }
    /// `AddPrimitivesGroup`: a new group of the polylines with a line aspect; clears the list.
    /// Example: `Prs3d::add_primitives_group(&prs, &aspect, &mut polylines)?`.
    pub fn add_primitives_group(
        prs: &Handle<dyn Graphic3dStructureSource>,
        aspect: &Handle<Prs3dLineAspect>,
        polylines: &mut List<Handle<HSequence<Point3>>>,
    ) -> Result<(), Failure> {
        let primitives = Self::primitives_from_polylines(polylines)?;
        polylines.clear();
        let (Some(structure), Some(line)) = (prs.get(), aspect.get()) else {
            return Ok(());
        };
        if primitives.is_null() {
            return Ok(());
        }
        let group = structure.new_group()?;
        if let Some(group) = group.get() {
            group.set_primitives_aspect(&line.aspect().cast());
            group.add_primitive_array(&primitives, true);
        }
        Ok(())
    }
    /// `AddFreeEdges`: appends both ends of every free triangle edge, transformed.
    /// Example: `Prs3d::add_free_edges(&mut segments, &triangulation, &trsf)?`.
    pub fn add_free_edges(
        segments: &mut Sequence<Point3>,
        poly_tri: &Handle<Triangulation>,
        location: &Transform3,
    ) -> Result<(), Failure> {
        let Some(triangulation) = poly_tri.get().filter(|t| t.has_geometry()) else {
            return Ok(());
        };
        let connect = Connect::new(poly_tri)?;
        let free = free_edge_nodes(&connect, triangulation)?;
        for pair in free.as_slice().chunks(2) {
            segments.append(triangulation.node(pair[0])?.transformed(*location));
            segments.append(triangulation.node(pair[1])?.transformed(*location));
        }
        Ok(())
    }
}

/// Adds one polyline: its vertices and the edges between consecutive ones.
fn add_polyline(
    segments: &Graphic3dArrayOfSegments,
    sequence: &Sequence<Point3>,
) -> Result<(), Failure> {
    let first = sequence.first().ok_or_else(|| {
        no_such_object("NCollection_Sequence::First: empty polyline; expected at least one point")
    })?;
    let mut edge = segments.vertex_number() + 1;
    segments.add_vertex_pnt(first)?;
    for index in 2..=sequence.length() {
        segments.add_vertex_pnt(sequence.value(index as usize)?)?;
        segments.add_edge(edge)?;
        edge += 1;
        segments.add_edge(edge)?;
    }
    Ok(())
}

/// The node pairs of the free edges, in triangle order (the native `aFree` array).
fn free_edge_nodes(
    connect: &Connect,
    triangulation: &Triangulation,
) -> Result<Array1<'static, i32>, Failure> {
    let nb_triangles = triangulation.nb_triangles();
    let mut nb_free = 0;
    for index in 1..=nb_triangles {
        let (t0, t1, t2) = connect.triangles(index)?;
        nb_free += [t0, t1, t2].iter().filter(|t| **t == 0).count() as i32;
    }
    let mut free =
        Array1::with_bounds(1, 2 * nb_free).map_err(|e| e.into_failure("Prs3d::AddFreeEdges"))?;
    let mut free_index = 1;
    for index in 1..=nb_triangles {
        let (t0, t1, t2) = connect.triangles(index)?;
        let (n0, n1, n2) = triangulation.triangle(index)?.get();
        let (adjacent, nodes) = ([t0, t1, t2], [n0, n1, n2]);
        for j in 0..3 {
            if adjacent[j] == 0 {
                free.set_value(free_index, nodes[j])
                    .map_err(|e| e.into_failure("Prs3d::AddFreeEdges"))?;
                free.set_value(free_index + 1, nodes[(j + 1) % 3])
                    .map_err(|e| e.into_failure("Prs3d::AddFreeEdges"))?;
                free_index += 2;
            }
        }
    }
    Ok(free)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_segment_projects_and_measures() {
        let (a, b) = (Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
        assert_eq!(
            Prs3d::match_segment(0.5, 0.1, 0.0, 0.2, &a, &b),
            (true, Some(0.1))
        );
        assert_eq!(
            Prs3d::match_segment(0.5, 0.3, 0.0, 0.2, &a, &b),
            (false, Some(0.3))
        );
        assert_eq!(
            Prs3d::match_segment(2.0, 0.0, 0.0, 0.2, &a, &b),
            (false, None)
        );
        assert_eq!(
            Prs3d::match_segment(0.0, 0.0, 0.0, 0.2, &a, &a),
            (false, None)
        );
    }

    #[test]
    fn deflection_of_corners_and_boxes() {
        let d = Prs3d::get_deflection(&Vec3::new(0.0, 0.0, 0.0), &Vec3::new(1.0, 2.0, 3.0), 0.001);
        assert_eq!(d, 3.0 * 0.001 * 4.0);
        let tiny =
            Prs3d::get_deflection(&Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 0.0, 0.0), 0.001);
        assert_eq!(tiny, confusion());
        assert_eq!(
            Prs3d::get_deflection_of_box(&BoundingBox3::default(), 0.001, 0.5),
            0.5
        );
        let bounds = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 1.0, 1.0));
        assert_eq!(
            Prs3d::get_deflection_of_box(&bounds, 0.001, 0.5),
            10.0 * 0.001 * 4.0
        );
    }

    #[test]
    fn polylines_become_segments() {
        let mut polylines = List::default();
        let line = HSequence::default();
        [0.0, 1.0, 2.0]
            .into_iter()
            .for_each(|x| line.append(Point3::new(x, 0.0, 0.0)));
        polylines.append(Handle::new(line));
        let array = Prs3d::primitives_from_polylines(&polylines).unwrap();
        assert_eq!(array.vertex_number(), 3);
        assert_eq!(array.edge_number(), 4);
        assert!(
            Prs3d::primitives_from_polylines(&List::default())
                .unwrap()
                .is_null()
        );
        let mut with_empty = List::default();
        with_empty.append(Handle::new(HSequence::default()));
        assert!(Prs3d::primitives_from_polylines(&with_empty).is_err());
    }
}
