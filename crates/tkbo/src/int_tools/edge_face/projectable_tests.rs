use super::super::EdgeFace;
use super::append_projectable_ranges;
use crate::int_tools::{Context, Range};
use rustycad_tkbrep::{
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    topods::Face,
};
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

/// IntTools_EdgeFace::Perform tests IsProjectable at IntTools_Tools::IntermediatePoint
/// (0.43213918 of the range), not at the midpoint: the range [0.6, 1.5] of a line along x
/// over the unit square face keeps its sample 0.989 inside, while its midpoint 1.05 is out.
#[test]
fn projectable_ranges_are_sampled_at_the_native_intermediate_point() {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0.5, 0.), Point3::new(2., 0.5, 0.))
            .unwrap()
            .edge()
            .unwrap();
    let face: Face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 1., 0., 1.))
        .unwrap()
        .face()
        .unwrap();
    let mut intersector = EdgeFace::new();
    intersector.edge = edge.clone();
    intersector.face = face.clone();
    intersector.context = Handle::new(Context::default());
    intersector.curve = Handle::new(BRepAdaptorCurve::with_edge(&edge).unwrap());
    intersector.surface = Handle::new(BRepAdaptorSurface::with_face(&face, true).unwrap());
    intersector.criteria = 2e-7;
    let mut ranges = Sequence::default();
    ranges.append(Range::new(0.6, 1.5));
    ranges.append(Range::new(1.2, 1.8));

    append_projectable_ranges(&mut intersector, &ranges).unwrap();

    assert_eq!(intersector.common_parts.len(), 1);
    let part = intersector.common_parts.value(1).unwrap();
    assert_eq!(part.range1_bounds(), (0.6, 1.5));
}
