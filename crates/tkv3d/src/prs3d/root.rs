//! `Prs3d_Root` (the deprecated group helpers of the presentation algorithms) and the
//! `Prs3d_Point` template (a point presentation over a point tool).
use super::Prs3dDrawer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPoints, Graphic3dGroup, Graphic3dGroupMembers as _,
    Graphic3dStructureMembers as _, Graphic3dStructureSource,
};
use std::marker::PhantomData;

/// The deprecated base of the presentation algorithms.
pub struct Prs3dRoot;

impl Prs3dRoot {
    /// Deprecated `CurrentGroup`: the presentation's current group.
    /// Example: `Prs3dRoot::current_group(&prs)?`.
    pub fn current_group(
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        prs.get()
            .map_or_else(|| Ok(Handle::default()), |s| s.current_group())
    }
    /// Deprecated `NewGroup`: a new group of the presentation; `Prs3dRoot::new_group(&prs)?`.
    pub fn new_group(
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        prs.get()
            .map_or_else(|| Ok(Handle::default()), |s| s.new_group())
    }
}

/// The `PointTool` parameter of `Prs3d_Point`: reads the coordinates of a point type.
pub trait Prs3dPointTool<P> {
    /// `PointTool::Coord(point, x, y, z)`; `Tool::coord(&p)`.
    fn coord(point: &P) -> (f64, f64, f64);
}

/// The `Prs3d_Point<AnyPoint, PointTool>` template: draws and matches one point.
///
/// Example: `Prs3dPoint::<MyPoint, MyTool>::add(&prs, &point, &drawer)?`.
pub struct Prs3dPoint<P, T: Prs3dPointTool<P>> {
    marker: PhantomData<(P, T)>,
}

impl<P, T: Prs3dPointTool<P>> Prs3dPoint<P, T> {
    /// The private `DrawPoint`: one point primitive in the group.
    fn draw_point(point: &P, group: &dyn Graphic3dGroup) -> Result<(), Failure> {
        let (x, y, z) = T::coord(point);
        let points = Graphic3dArrayOfPoints::new(1, 0)?;
        points.add_vertex_f64(x, y, z)?;
        group.add_primitive_array(&Handle::new(points).cast(), true);
        Ok(())
    }
    /// `Add`: the point in the current group with the drawer's point aspect.
    /// Example: `Prs3dPoint::<P, T>::add(&prs, &point, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        point: &P,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let (Some(structure), Some(drawer)) = (prs.get(), drawer.get()) else {
            return Ok(());
        };
        let group = structure.current_group()?;
        let Some(group) = group.get() else {
            return Ok(());
        };
        if let Some(aspect) = drawer.point_aspect().get() {
            group.set_primitives_aspect(&aspect.aspect().cast());
        }
        Self::draw_point(point, group)
    }
    /// `Match`: whether the point is within `distance` of `(x, y, z)`.
    /// Example: `Prs3dPoint::<P, T>::matches(&point, 0.0, 0.0, 0.0, 1.0)`.
    pub fn matches(point: &P, x: f64, y: f64, z: f64, distance: f64) -> bool {
        let (px, py, pz) = T::coord(point);
        ((x - px) * (x - px) + (y - py) * (y - py) + (z - pz) * (z - pz)).sqrt() <= distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::Point3;

    struct GpPointTool;
    impl Prs3dPointTool<Point3> for GpPointTool {
        fn coord(point: &Point3) -> (f64, f64, f64) {
            (point.x(), point.y(), point.z())
        }
    }

    #[test]
    fn match_uses_euclidean_distance() {
        let point = Point3::new(1.0, 1.0, 1.0);
        assert!(Prs3dPoint::<Point3, GpPointTool>::matches(
            &point, 1.0, 1.0, 2.0, 1.0
        ));
        assert!(!Prs3dPoint::<Point3, GpPointTool>::matches(
            &point, 1.0, 1.0, 2.5, 1.0
        ));
    }
}
