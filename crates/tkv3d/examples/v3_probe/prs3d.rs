//! Prs3d numeric parts, the twin of tools/oracle/tkv3d_probe_prs3d.cxx.
use super::array::array_state;
use super::common::{Reader, end, flag, int, pnt, real};
use super::print::show;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL, Graphic3dArrayOfPrimitives, Graphic3dArrayOfTriangles,
};
use rustycad_tkv3d::prs3d::{
    Prs3d, Prs3dArrow, Prs3dBndBox, Prs3dToolCylinder, Prs3dToolDisk, Prs3dToolQuadric,
    Prs3dToolSector, Prs3dToolSphere, Prs3dToolTorus,
};

/// `v3_trsf`: a rotation about an axis through the origin, then a translation part.
fn read_trsf(r: &mut Reader<'_>) -> Transform3 {
    let (t, axis, angle) = (r.xyz(), r.dir(), r.real());
    let mut trsf = Transform3::default();
    trsf.set_rotation(Axis1::new(Point3::new(0.0, 0.0, 0.0), axis), angle)
        .expect("probe rotation");
    trsf.set_translation_part(Vector3::new(t.x(), t.y(), t.z()));
    trsf
}

/// `v3_tool`: the tool of a kind with its five parameters.
fn tool_of(kind: i32, p: [f64; 5], slices: i32, stacks: i32) -> Box<dyn Prs3dToolQuadric> {
    match kind {
        0 => Box::new(Prs3dToolCylinder::new(p[0], p[1], p[2], slices, stacks)),
        1 => {
            let mut disk = Prs3dToolDisk::new(p[0], p[1], slices, stacks);
            disk.set_angle_range(p[2], p[3]);
            Box::new(disk)
        }
        2 => Box::new(Prs3dToolSphere::new(p[0], slices, stacks)),
        3 => Box::new(Prs3dToolTorus::new(
            p[0], p[1], p[2], p[3], p[4], slices, stacks,
        )),
        _ => Box::new(Prs3dToolSector::new(p[0], slices, stacks)),
    }
}

/// Prints the array of a handle (the base class view of `sv_array_state`).
fn state_of<T: rustycad_tkernel::standard::RuntimeTyped>(array: &Handle<T>) {
    let base: Handle<Graphic3dArrayOfPrimitives> = array.clone().cast();
    if let Some(base) = base.get() {
        array_state(base);
    }
}

fn poly_triangulation(tool: &dyn Prs3dToolQuadric, trsf: &Transform3) -> Result<(), Failure> {
    let t = tool.create_poly_triangulation(trsf)?;
    int(t.nb_nodes());
    int(t.nb_triangles());
    for i in 1..=t.nb_nodes() {
        pnt(t.node(i)?);
    }
    for i in 1..=t.nb_triangles() {
        let (n1, n2, n3) = t.triangle(i)?.get();
        [n1, n2, n3].into_iter().for_each(int);
    }
    Ok(())
}

/// `v3_quadric Kind P0..P4 Slices Stacks Trsf`.
pub fn quadric_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let p: [f64; 5] = std::array::from_fn(|_| r.real());
    let (slices, stacks) = (r.integer(), r.integer());
    let trsf = read_trsf(&mut r);
    let tool = tool_of(kind, p, slices, stacks);
    int(tool.triangles_nb());
    int(tool.vertices_nb(true));
    int(tool.vertices_nb(false));
    show(tool.create_triangulation(&trsf), |a| state_of(&a));
    let filled = Graphic3dArrayOfTriangles::new(
        tool.vertices_nb(false),
        0,
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
    )
    .map(Handle::new)
    .and_then(|mut b| tool.fill_array(&mut b, &trsf).map(|()| b));
    show(filled, |b| state_of(&b));
    show(poly_triangulation(tool.as_ref(), &trsf), |()| ());
    end();
}

/// `v3_arrow Loc(3) Dir(3) Angle Length NbSegments Tube AxisLength Cone ConeLength NbFacettes`.
pub fn arrow_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (location, dir) = (r.pnt(), r.dir());
    let (angle, length, nb) = (r.real(), r.real(), r.integer());
    let (tube, axis_length, cone, cone_length) = (r.real(), r.real(), r.real(), r.real());
    let facettes = r.integer();
    show(
        Prs3dArrow::draw_segments(&location, &dir, angle, length, nb),
        |s| state_of(&s),
    );
    let shaded = Prs3dArrow::draw_shaded(
        &Axis1::new(location, dir),
        tube,
        axis_length,
        cone,
        cone_length,
        facettes,
    );
    show(shaded, |s| {
        flag(!s.is_null());
        state_of(&s);
    });
    end();
}

/// `v3_prs3d Min(3) Max(3) Coef MaxChord Open X Y Z Distance P1(3) P2(3)`.
pub fn prs3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mn, mx) = (r.pnt(), r.pnt());
    let (coef, chord, open) = (r.real(), r.real(), r.flag());
    let (x, y, z, distance) = (r.real(), r.real(), r.real(), r.real());
    let (p1, p2) = (r.pnt(), r.pnt());
    let min = Vec3::new(mn.x(), mn.y(), mn.z());
    let max = Vec3::new(mx.x(), mx.y(), mx.z());
    real(Prs3d::get_deflection(&min, &max, coef));
    let mut bounds = BoundingBox3::default();
    if mn.x() <= mx.x() {
        bounds.update_bounds(*mn.coordinates(), *mx.coordinates());
        if open {
            bounds.open_xmin();
        }
    }
    real(Prs3d::get_deflection_of_box(&bounds, coef, chord));
    let (found, dist) = Prs3d::match_segment(x, y, z, distance, &p1, &p2);
    flag(found);
    real(dist.unwrap_or(-1.0));
    let segments = Prs3dBndBox::fill_box_segments(&bounds).unwrap_or_default();
    flag(!segments.is_null());
    state_of(&segments);
    end();
}
