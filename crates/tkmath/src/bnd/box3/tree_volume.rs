//! `Bnd_Box` as the bounding volume of `NCollection_UBTree<int, Bnd_Box>`.
use crate::bnd::BoundingBox3;
use rustycad_tkernel::ncollection::BoundingVolume;

/// The members that `NCollection_UBTree` calls on its bound type: `Add`, `IsOut(Bnd_Box)`
/// and `SquareExtent`. It lives here, beside `Bnd_Box`, so that every toolkit
/// instantiates the tree on the same type (TKTopAlgo's BRepClass3d_SolidExplorer, for
/// one; docs/topalgo-design.md section 5).
///
/// ```
/// use rustycad_tkernel::ncollection::BoundingVolume;
/// use rustycad_tkmath::bnd::BoundingBox3;
/// use rustycad_tkmath::gp::Point3;
/// let mut unit = BoundingBox3::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 1.));
/// let far = BoundingBox3::new(Point3::new(5., 5., 5.), Point3::new(6., 6., 6.));
/// assert!(BoundingVolume::is_out(&unit, &far));
/// BoundingVolume::add(&mut unit, &far);
/// assert_eq!(BoundingVolume::square_extent(&unit), 108.0);
/// ```
impl BoundingVolume for BoundingBox3 {
    fn add(&mut self, other: &Self) {
        self.add_box(other);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.is_out_box(other)
    }
    fn square_extent(&self) -> f64 {
        BoundingBox3::square_extent(self)
    }
}
