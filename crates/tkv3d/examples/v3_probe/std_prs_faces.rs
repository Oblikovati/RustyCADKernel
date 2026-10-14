//! StdPrs face and surface presentations, the twin of tools/oracle/tkv3d_probe_std_prs_faces.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::sensitive_display::display_driver::{DisplayTrace, SensitiveDisplayDriver};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Builder, Compound, Face, Shape};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve, GeomLine,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{
    Axis1, Axis3, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3,
};
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tkservice::graphic3d::*;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::std_prs::*;
use rustycad_tkv3d::v3d::V3dViewer;
use std::f64::consts::PI;

fn pole_surface(bspline: bool) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut poles = Array2::with_bounds(1, 3, 1, 3).map_err(|e| e.into_failure("Array2"))?;
    for i in 1..=3 {
        for j in 1..=3 {
            let z = if i == 2 && j == 2 {
                2.
            } else {
                0.1 * f64::from(i) * f64::from(j)
            };
            poles
                .set_value(i, j, Point3::new(f64::from(i), f64::from(j), z))
                .map_err(|e| e.into_failure("Array2"))?;
        }
    }
    if !bspline {
        return Ok(Handle::new(GeomBezierSurface::new(&poles)?).into_cast());
    }
    let mut knot_values = [0., 1.];
    let mut mult_values = [3, 3];
    let knots = Array1::from_slice(&mut knot_values);
    let mults = Array1::from_slice(&mut mult_values);
    Ok(Handle::new(GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (2, 2),
        (false, false),
    )?)
    .into_cast())
}

fn face(kind: i32) -> Result<Face, Failure> {
    let (z, x) = (Direction3::new(0., 0., 1.)?, Direction3::new(1., 0., 0.)?);
    let origin = Point3::new(0.5, -1., 0.25);
    let ax = Axis3::new(origin, z, x)?;
    let maker = match kind {
        0 => {
            BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::from_position(ax), (0., 2., 0., 1.))?
        }
        1 => BRepBuilderAPIMakeFace::from_cylinder_bounds(
            Cylinder3::new(ax, 2.)?,
            (0., 2. * PI, 0., 3.),
        )?,
        2 => BRepBuilderAPIMakeFace::from_cylinder_bounds(
            Cylinder3::new(ax, 2.)?,
            (0., PI, -1., 1.),
        )?,
        3 => BRepBuilderAPIMakeFace::from_sphere(Sphere3::new(ax, 1.5)?)?,
        4 => BRepBuilderAPIMakeFace::from_cone_bounds(Cone3::new(ax, 0.4, 1.)?, (0., 4., 0., 2.))?,
        5 => BRepBuilderAPIMakeFace::from_torus(Torus3::new(ax, 3., 1.)?)?,
        6 => BRepBuilderAPIMakeFace::from_surface(&pole_surface(true)?, 1.0e-7)?,
        7 => BRepBuilderAPIMakeFace::from_surface(&pole_surface(false)?, 1.0e-7)?,
        _ => BRepBuilderAPIMakeFace::from_plane(Plane3::from_position(ax))?,
    };
    maker.face()
}

fn drawer(r: &mut Reader<'_>) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.u_iso_aspect().set_number(r.integer());
    drawer.v_iso_aspect().set_number(r.integer());
    drawer.set_discretisation(r.integer());
    drawer.set_maximal_chordial_deviation(r.real());
    drawer.enable_draw_hidden_line();
    drawer.set_maximal_parameter_value(r.real());
    drawer.set_type_of_deflection(if r.flag() {
        AspectTypeOfDeflection::Relative
    } else {
        AspectTypeOfDeflection::Absolute
    });
    drawer
}

fn bounds(b: &BoundingBox3) {
    flag(b.is_void());
    if !b.is_void() {
        for value in b.get().unwrap() {
            real(value);
        }
    }
}

fn curves(list: &StdPrsIsolinePolylines) {
    int(list.extent());
    for points in list.iter() {
        points.sequence(|sequence| {
            int(sequence.length());
            for p in sequence.iter() {
                pnt(*p);
            }
        });
    }
}

fn shape_tool(f: &Face, all: bool) -> Result<(), Failure> {
    let builder = Builder;
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &Shape::from(f.clone()))?;
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(5., 5., 5.), Point3::new(6., 5., 5.))?;
    builder.add(&mut compound, &Shape::from(edge.edge()?))?;
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(7., 7., 7.))?;
    builder.add(&mut compound, &Shape::from(vertex.vertex()?))?;
    let mut tool = StdPrsShapeTool::new(&Shape::from(compound), all)?;
    tool.init_face()?;
    while tool.more_face() {
        flag(tool.is_planar_current_face()?);
        flag(tool.has_surface()?);
        bounds(&tool.face_bound()?);
        flag(tool.current_triangulation()?.0.is_null());
        tool.next_face()?;
    }
    int(-1);
    tool.init_curve();
    while tool.more_curve() {
        int(tool.neighbours()?);
        int(tool.faces_of_edge()?.sequence(|s| s.length()));
        flag(tool.has_curve()?);
        bounds(&tool.curve_bound()?);
        flag(tool.polygon_3d()?.0.is_null());
        flag(tool.polygon_on_triangulation()?.0.is_null());
        tool.next_curve();
    }
    int(-1);
    tool.init_vertex();
    while tool.more_vertex() {
        let (x, y, z) = StdPrsToolVertex::coord(&tool.get_vertex()?)?;
        real(x);
        real(y);
        real(z);
        tool.next_vertex();
    }
    Ok(())
}

fn pole_curve(
    r: &mut Reader<'_>,
    dr: &Handle<Prs3dDrawer>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    kind: i32,
) -> Result<(), Failure> {
    let mut pole_values: Vec<Point3> = (1..=4)
        .map(|i| Point3::new(f64::from(i), f64::from(i % 2) * 1.5, 0.25 * f64::from(i)))
        .collect();
    let poles = Array1::from_slice(&mut pole_values);
    let curve: Handle<dyn GeomCurve> = match kind {
        0 => Handle::new(GeomBezierCurve::new(&poles)?).into_cast(),
        1 => {
            let mut knot_values = [0., 1.];
            let mut mult_values = [4, 4];
            Handle::new(GeomBSplineCurve::new(
                &poles,
                &Array1::from_slice(&mut knot_values),
                &Array1::from_slice(&mut mult_values),
                3,
                false,
            )?)
            .into_cast()
        }
        _ => Handle::new(GeomLine::new(Axis1::new(
            Point3::new(0., 0., 0.),
            Direction3::new(1., 1., 0.)?,
        )))
        .into_cast(),
    };
    let adaptor = GeomAdaptorCurve::with_range(&curve, 0., 1.)?;
    dr.set_line_arrow_draw(r.flag());
    attempt(|| StdPrsPoleCurve::add(prs, &adaptor, dr));
    for _ in 0..4 {
        let (x, y, z, d) = (r.real(), r.real(), r.real(), r.real());
        attempt(|| StdPrsPoleCurve::match_point(x, y, z, d, &adaptor, dr).map(flag));
        attempt(|| StdPrsPoleCurve::pick(x, y, z, d, &adaptor, dr).map(int));
    }
    Ok(())
}

/// `v3_stdprs_face Op Face Drawer Values...`.
pub fn std_prs_face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (op, kind) = (r.integer(), r.integer());
    let dr = drawer(&mut r);
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let structure =
        Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let prs: Handle<dyn Graphic3dStructureSource> = structure.clone().cast();
    let f = match face(kind) {
        Ok(f) => f,
        Err(failure) => {
            print!("{} ", failure.exception_type());
            return end();
        }
    };
    let adaptor = Handle::new(BRepAdaptorSurface::with_face(&f, true).unwrap());
    let dynamic: Handle<dyn Adaptor3dSurface> = adaptor.clone().into_cast();
    let point = |r: &mut Reader<'_>| {
        let (x, y, z) = (r.real(), r.real(), r.real());
        (Point3::new(x, y, z), r.real())
    };
    match op {
        0 => {
            let all = r.flag();
            attempt(|| shape_tool(&f, all));
        }
        1 => {
            let (u, v, nu, nv) = (r.flag(), r.flag(), r.integer(), r.integer());
            let mut list = StdPrsIsolinePolylines::default();
            attempt(|| {
                StdPrsWfRestrictedFace::add_isos(&prs, &adaptor, u, v, nu, nv, &dr, &mut list)
            });
            curves(&list);
        }
        2 => {
            for _ in 0..4 {
                let (p, d) = point(&mut r);
                attempt(|| StdPrsWfRestrictedFace::matches(&p, d, &adaptor, &dr).map(flag));
                attempt(|| StdPrsWfRestrictedFace::match_u_iso(&p, d, &adaptor, &dr).map(flag));
                attempt(|| StdPrsWfRestrictedFace::match_v_iso(&p, d, &adaptor, &dr).map(flag));
            }
        }
        3 => {
            let (u, v, nu, nv, defl) = (r.flag(), r.flag(), r.integer(), r.integer(), r.real());
            let mut list = StdPrsIsolinePolylines::default();
            attempt(|| {
                StdPrsWfDeflectionRestrictedFace::add_isos(
                    &prs, &adaptor, u, v, defl, nu, nv, &dr, &mut list,
                )
            });
            curves(&list);
        }
        4 => {
            for _ in 0..4 {
                let (p, d) = point(&mut r);
                attempt(|| {
                    StdPrsWfDeflectionRestrictedFace::matches(&p, d, &adaptor, &dr).map(flag)
                });
                attempt(|| {
                    StdPrsWfDeflectionRestrictedFace::match_u_iso(&p, d, &adaptor, &dr).map(flag)
                });
                attempt(|| {
                    StdPrsWfDeflectionRestrictedFace::match_v_iso(&p, d, &adaptor, &dr).map(flag)
                });
            }
        }
        5 => attempt(|| StdPrsWfSurface::add(&prs, &dynamic, &dr)),
        6 => attempt(|| StdPrsWfDeflectionSurface::add(&prs, &dynamic, &dr)),
        7 => attempt(|| StdPrsShadedSurface::add(&prs, &*adaptor, &dr)),
        8 => attempt(|| StdPrsWfPoleSurface::add(&prs, &*adaptor, &dr)),
        9 => attempt(|| pole_curve(&mut r, &dr, &prs, kind % 3)),
        10 | 11 => {
            let camera = Handle::new(Graphic3dCamera::default());
            camera.set_projection_type(if r.flag() {
                Graphic3dCameraProjection::Orthographic
            } else {
                Graphic3dCameraProjection::Perspective
            });
            let (eye, center) = (r.pnt(), r.pnt());
            attempt(|| camera.set_eye_and_center(&eye, &center));
            camera.set_up(&r.dir());
            camera.set_scale(r.real());
            let shape = Shape::from(f.clone());
            if op == 11 {
                BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false).unwrap();
            }
            if op == 10 {
                attempt(|| StdPrsHlrShape::default().compute_hlr(&prs, &shape, &dr, &camera));
            } else {
                attempt(|| StdPrsHlrPolyShape::default().compute_hlr(&prs, &shape, &dr, &camera));
            }
        }
        _ => {}
    }
    display_events(&trace);
    int(structure.groups().len() as i32);
    end();
}

fn display_events(trace: &DisplayTrace) {
    let trace = trace.read();
    int(trace.events.len() as i32);
    for value in &trace.events {
        real(*value);
    }
}
