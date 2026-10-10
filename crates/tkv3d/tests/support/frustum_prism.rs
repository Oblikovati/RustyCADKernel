//! A named rectangular prism fixture with native cache ordering.
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkv3d::select_mgr::SelectMgrFrustumCore;

pub fn rectangular_prism() -> SelectMgrFrustumCore<4> {
    let mut core = SelectMgrFrustumCore::default();
    core.planes_mut().copy_from_slice(&[
        Xyz::new(0., 1., 0.),
        Xyz::new(0., -1., 0.),
        Xyz::new(1., 0., 0.),
        Xyz::new(-1., 0., 0.),
        Xyz::new(0., 0., 1.),
        Xyz::new(0., 0., -1.),
    ]);
    let mut index = 0;
    for (x, y) in [(-1., 1.), (-1., -1.), (1., 1.), (1., -1.)] {
        for z in [-2., 2.] {
            core.vertices_mut()[index] = Point3::new(x, y, z);
            index += 1;
        }
    }
    core.max_verts_projections_mut()
        .copy_from_slice(&[1., 1., 1., 1., 2., 2.]);
    core.min_verts_projections_mut()
        .copy_from_slice(&[-1., -1., -1., -1., -2., -2.]);
    core.max_ortho_verts_projections_mut()
        .copy_from_slice(&[1., 1., 2.]);
    core.min_ortho_verts_projections_mut()
        .copy_from_slice(&[-1., -1., -2.]);
    core.edge_dirs_mut().copy_from_slice(&[
        Xyz::new(2., 0., 0.),
        Xyz::new(0., -2., 0.),
        Xyz::new(0., 0., 4.),
        Xyz::new(0., 0., 4.),
        Xyz::new(0., 0., 4.),
        Xyz::new(0., 0., 4.),
    ]);
    core
}
