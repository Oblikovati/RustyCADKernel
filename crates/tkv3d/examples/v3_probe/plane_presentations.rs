//! Bit-exact native plane buffer and aspect callback comparisons.
use super::{
    common::{Reader, attempt, end, flag, int, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSphericalSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{dsg_prs::*, prs3d::*, std_prs::StdPrsPlane, v3d::V3dViewer};
fn probe_surface(kind: i32) -> GeomAdaptorSurface {
    let frame = probe_frame(kind);
    let geometry = if kind == 2 {
        Handle::new(GeomSphericalSurface::new(frame, 2.).unwrap()).cast()
    } else {
        Handle::new(GeomPlane::new(frame)).cast()
    };
    GeomAdaptorSurface::with_surface(&geometry).unwrap()
}
fn probe_frame(kind: i32) -> Axis3 {
    if kind == 0 {
        return Axis3::default();
    }
    Axis3::new(
        Point3::new(2., 3., 4.),
        Direction3::new(0., 0.6, 0.8).unwrap(),
        Direction3::X,
    )
    .unwrap()
}
fn probe_drawer(reader: &mut Reader<'_>, flags: i32) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    let aspect = Handle::new(Prs3dPlaneAspect::new().unwrap());
    aspect.set_plane_length(reader.real(), reader.real());
    aspect.set_iso_distance(reader.real());
    aspect.set_display_edges(flags & 1 != 0);
    aspect.set_display_iso(flags & 2 != 0);
    aspect.set_display_center_arrow(flags & 4 != 0);
    aspect.set_display_edges_arrows(flags & 8 != 0);
    aspect.set_arrows_size(2.);
    aspect.set_arrows_length(0.7);
    aspect.set_arrows_angle(0.2);
    drawer.set_plane_aspect(aspect);
    drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new().unwrap()));
    drawer
}
pub fn plane_add_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (kind, variant, flags) = (reader.integer(), reader.integer(), reader.integer());
    let drawer = probe_drawer(&mut reader, flags);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    if flags & 16 != 0 {
        trace.write().mutate_plane_aspect = drawer.plane_aspect();
    }
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    attempt(|| submit_plane(&prs.cast(), &drawer, kind, variant));
    let trace = trace.read();
    int(trace.events.len() as i32);
    for value in &trace.events {
        real(*value);
    }
    int(prs.groups().len() as i32);
    end();
}
fn submit_plane(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Handle<Prs3dDrawer>,
    kind: i32,
    variant: i32,
) -> Result<(), Failure> {
    let [a, b, c] = [
        Point3::new(1., 2., 3.),
        Point3::new(-4., 5., 6.),
        Point3::new(7., -8., 9.),
    ];
    match variant {
        1 => DsgPrsXYZPlanePresentation::add(prs, drawer, &a, &b, &c),
        2 => DsgPrsShadedPlanePresentation::add(prs, drawer, &a, &b, &c),
        _ => StdPrsPlane::add(prs, &probe_surface(kind), drawer),
    }
}
pub fn plane_match_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let surface = probe_surface(reader.integer());
    let point = reader.pnt();
    let distance = reader.real();
    attempt(|| {
        flag(StdPrsPlane::matches(
            point.x(),
            point.y(),
            point.z(),
            distance,
            &surface,
            &Handle::default(),
        )?);
        Ok(())
    });
    end();
}
