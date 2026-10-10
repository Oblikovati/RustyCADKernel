//! `Prs3d_BndBox`: the twelve edges of an axis-aligned or oriented bounding box.
use super::Prs3dDrawer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::{BoundingBox3, OrientedBoundingBox};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfSegments, Graphic3dAspectLine3d, Graphic3dAspectsMembers as _,
    Graphic3dGroupMembers as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// The package class `Prs3d_BndBox`.
pub struct Prs3dBndBox;

/// The corners of an axis-aligned box in the native order (X fastest, then Y, then Z).
fn box_corners(bnd_box: &BoundingBox3) -> Result<[Point3; 8], Failure> {
    let (min, max) = (bnd_box.corner_min()?, bnd_box.corner_max()?);
    Ok([
        Point3::new(min.x(), min.y(), min.z()),
        Point3::new(max.x(), min.y(), min.z()),
        Point3::new(min.x(), max.y(), min.z()),
        Point3::new(max.x(), max.y(), min.z()),
        Point3::new(min.x(), min.y(), max.z()),
        Point3::new(max.x(), min.y(), max.z()),
        Point3::new(min.x(), max.y(), max.z()),
        Point3::new(max.x(), max.y(), max.z()),
    ])
}

impl Prs3dBndBox {
    /// `fillSegments`: appends the eight corners and the twelve edges between them.
    /// Example: `Prs3dBndBox::fill_corner_segments(&segments, &corners)?`.
    pub fn fill_corner_segments(
        segments: &Graphic3dArrayOfSegments,
        corners: &[Point3; 8],
    ) -> Result<(), Failure> {
        let from = segments.vertex_number();
        for corner in corners {
            segments.add_vertex_pnt(corner)?;
        }
        let edges = [
            (1, 2),
            (3, 4),
            (5, 6),
            (7, 8),
            (1, 3),
            (2, 4),
            (5, 7),
            (6, 8),
            (1, 5),
            (2, 6),
            (3, 7),
            (4, 8),
        ];
        for (a, b) in edges {
            segments.add_edges2(from + a, from + b)?;
        }
        Ok(())
    }
    /// `FillSegments(segments, Bnd_Box)`: appends the box edges unless the box is void.
    /// Example: `Prs3dBndBox::fill_box_segments_into(&segments, &bounds)?`.
    pub fn fill_box_segments_into(
        segments: &Graphic3dArrayOfSegments,
        bnd_box: &BoundingBox3,
    ) -> Result<(), Failure> {
        if bnd_box.is_void() {
            return Ok(());
        }
        Self::fill_corner_segments(segments, &box_corners(bnd_box)?)
    }
    /// `FillSegments(segments, Bnd_OBB)`: appends the box edges unless the box is void.
    /// Example: `Prs3dBndBox::fill_obb_segments_into(&segments, &obb)?`.
    pub fn fill_obb_segments_into(
        segments: &Graphic3dArrayOfSegments,
        obb: &OrientedBoundingBox,
    ) -> Result<(), Failure> {
        match obb.vertices() {
            Some(corners) if !obb.is_void() => Self::fill_corner_segments(segments, &corners),
            _ => Ok(()),
        }
    }
    /// `FillSegments(Bnd_Box)`: a new array of the box edges, null for a void box.
    /// Example: `Prs3dBndBox::fill_box_segments(&bounds)?`.
    pub fn fill_box_segments(
        bnd_box: &BoundingBox3,
    ) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        if bnd_box.is_void() {
            return Ok(Handle::default());
        }
        let segments = Graphic3dArrayOfSegments::new(8, 12 * 2, 0)?;
        Self::fill_box_segments_into(&segments, bnd_box)?;
        Ok(Handle::new(segments))
    }
    /// `FillSegments(Bnd_OBB)`: a new array of the box edges, null for a void box.
    /// Example: `Prs3dBndBox::fill_obb_segments(&obb)?`.
    pub fn fill_obb_segments(
        obb: &OrientedBoundingBox,
    ) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        if obb.is_void() {
            return Ok(Handle::default());
        }
        let segments = Graphic3dArrayOfSegments::new(8, 12 * 2, 0)?;
        Self::fill_obb_segments_into(&segments, obb)?;
        Ok(Handle::new(segments))
    }
    /// `Add(prs, Bnd_Box, drawer)`: the box edges in the current group, dot-dashed in the
    /// drawer's line color and width. Example: `Prs3dBndBox::add_box(&prs, &bounds, &drawer)?`.
    pub fn add_box(
        prs: &Handle<dyn Graphic3dStructureSource>,
        bnd_box: &BoundingBox3,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        if bnd_box.is_void() {
            return Ok(());
        }
        let segments = Self::fill_box_segments(bnd_box)?;
        add_segments(prs, drawer, &segments)
    }
    /// `Add(prs, Bnd_OBB, drawer)`; `Prs3dBndBox::add_obb(&prs, &obb, &drawer)?`.
    pub fn add_obb(
        prs: &Handle<dyn Graphic3dStructureSource>,
        obb: &OrientedBoundingBox,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        if obb.is_void() {
            return Ok(());
        }
        let segments = Self::fill_obb_segments(obb)?;
        add_segments(prs, drawer, &segments)
    }
}

/// The current group of `prs` gets a dot-dash aspect of the drawer's line aspect and the segments.
fn add_segments(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Handle<Prs3dDrawer>,
    segments: &Handle<Graphic3dArrayOfSegments>,
) -> Result<(), Failure> {
    let (Some(structure), Some(drawer)) = (prs.get(), drawer.get()) else {
        return Ok(());
    };
    let group = structure.current_group()?;
    let Some(group) = group.get() else {
        return Ok(());
    };
    let line = drawer.line_aspect().aspect();
    let (color, width) = line
        .get()
        .map_or_else(Default::default, |l| (l.interior_color(), l.width()));
    let aspect = Graphic3dAspectLine3d::new(color, AspectTypeOfLine::Dotdash, f64::from(width))?;
    group.set_group_primitives_aspect(&Handle::new(aspect).cast());
    group.add_primitive_array(&segments.cast(), true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_segments_have_eight_corners_and_twelve_edges() {
        let bounds = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 2.0, 3.0));
        let segments = Prs3dBndBox::fill_box_segments(&bounds).unwrap();
        assert_eq!((segments.vertex_number(), segments.edge_number()), (8, 24));
        assert!(
            Prs3dBndBox::fill_box_segments(&BoundingBox3::default())
                .unwrap()
                .is_null()
        );
        assert!(
            Prs3dBndBox::fill_obb_segments(&OrientedBoundingBox::default())
                .unwrap()
                .is_null()
        );
    }
}
