// Rust translation of the static helpers ComputeTrsf, Longueur and GoodOrientation
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Draft.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The frame of the drafted wire, the length of the draft up to a stop box and the
//! orientation of the draft law.
use super::null;
use crate::brep_fill::location_law::BRepFillLocationLaw;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkmath::toploc::Location;

/// `ComputeTrsf(W, D, Box, Tf)`: the transformation to the frame of normal `d` at the
/// approximate barycentre of the wire vertices, and the box of the transformed wire.
pub(super) fn compute_trsf(w: &Wire, d: Direction3) -> Result<(BoundingBox3, Transform3), Failure> {
    let mut exp = WireExplorer::with_wire(w)?;
    let mut bary = Xyz::new(0., 0., 0.);
    let mut nb: i32 = 0;
    while exp.more() {
        bary += BRepTool::pnt(exp.current_vertex())?.xyz();
        nb += 1;
        exp.next()?;
    }
    bary /= f64::from(nb);
    let mut tf = Transform3::default();
    tf.set_transformation_to(Axis3::from_normal(Point3::from(bary), d)?);
    let mut the_w: Wire = w.clone();
    the_w.set_location(&Location::new(tf), false)?;
    let mut bbox = BoundingBox3::default();
    bbox.set_void();
    let mut exp = WireExplorer::with_wire(&the_w)?;
    while exp.more() {
        let ac = BRepAdaptorCurve::with_edge(exp.current())?;
        Add3dCurve::add(&ac as &dyn Adaptor3dCurve, 0.1, &mut bbox)?;
        exp.next()?;
    }
    Ok((bbox, tf))
}

/// `Longueur(WBox, SBox, D, P)`: the length from the wire box to the far side of the
/// stop box along the frame Z (the direction reversed when the stop is below the
/// wire) and the middle of that side.
pub(super) fn longueur(
    w_box: &BoundingBox3,
    s_box: &BoundingBox3,
    d: &mut Direction3,
) -> Result<(f64, Point3), Failure> {
    let [_, _, wz_min, _, _, wz_max] = w_box.get()?;
    let [xmin, ymin, zmin, xmax, ymax, zmax] = s_box.get()?;
    let mut p = Point3::new((xmin + xmax) / 2., (ymin + ymax) / 2., zmax);
    if zmax < wz_min {
        // Skin in the wrong direction: inverted.
        d.reverse();
        p.set_z(zmin);
        return Ok((wz_max - zmin, p));
    }
    Ok((zmax - wz_min, p))
}

/// `GoodOrientation(B, Law, D)`: whether the path turns counterclockwise about `d`
/// (the sum of the angles between consecutive sample points seen from their
/// barycentre, about `d`, is not negative).
pub(super) fn good_orientation(
    b: &BoundingBox3,
    law: &Handle<dyn BRepFillLocationLaw>,
    d: Direction3,
) -> Result<bool, Failure> {
    let law = law.get().ok_or_else(|| null("draft law"))?;
    let [axmin, aymin, azmin, axmax, aymax, azmax] = b.get()?;
    let v = Vector3::between(
        Point3::new(axmin, aymin, azmin),
        Point3::new(axmax, aymax, azmax),
    );
    let (_, l) = law.curvilinear_bounds(law.nb_law()?)?;
    let r: f64 = v.magnitude() / l;
    let mut nb: i32 = (4. + 10. * r) as i32;
    let r: f64 = l / f64::from(nb);
    nb += 1;
    let mut pnts: Vec<Point3> = Vec::with_capacity(nb.max(0) as usize);
    let mut bary = Xyz::new(0., 0., 0.);
    for ii in 1..=nb {
        let (ind, t) = law.parameter(f64::from(ii - 1) * r)?;
        let curve = law
            .law(ind)?
            .get()
            .ok_or_else(|| null("draft law"))?
            .get_curve();
        let p: Point3 = curve.get().ok_or_else(|| null("path curve"))?.d0(t)?;
        bary += p.xyz();
        pnts.push(p);
    }
    bary /= f64::from(nb);
    let centre = Point3::from(bary);
    let normal = Vector3::from(d.xyz());
    let mut angle: f64 = 0.;
    let first: Point3 = *pnts.first().ok_or_else(|| null("sample point"))?;
    let mut reference = Vector3::between(centre, first);
    for p in pnts.iter().skip(1) {
        let r = Vector3::between(centre, *p);
        angle += reference.angle_with_ref(r, normal)?;
        reference = r;
    }
    Ok(angle >= 0.)
}
