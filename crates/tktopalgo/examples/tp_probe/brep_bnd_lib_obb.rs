//! The BRepBndLib::AddOBB probe operation over shapes read by `tp_shape`, the Rust twin
//! of tools/oracle/tktopalgo_probe_brepbndlib_obb.cxx.
use super::common::{Reader, attempt, end, flag, real, xyz};
use super::shape::tp_shape;
use rustycad_tkmath::bnd::OrientedBoundingBox;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// `tp_obb`: void flag, centre, axes, half sizes and axis-aligned flag.
pub fn print_obb(b: &OrientedBoundingBox) {
    flag(b.is_void());
    xyz(b.center());
    xyz(b.x_direction());
    xyz(b.y_direction());
    xyz(b.z_direction());
    real(b.x_h_size());
    real(b.y_h_size());
    real(b.z_h_size());
    flag(b.is_aa_box());
}

/// `tp_initial_obb`: a void box or an initial box set member by member.
fn initial_obb(r: &mut Reader<'_>) -> OrientedBoundingBox {
    let mut b = OrientedBoundingBox::default();
    if !r.flag() {
        return b;
    }
    let center = r.pnt();
    let (x, y, z) = (r.dir(), r.dir(), r.dir());
    let (hx, hy, hz) = (r.real(), r.real(), r.real());
    b.set_center(center);
    b.set_x_component(x, hx);
    b.set_y_component(y, hy);
    b.set_z_component(z, hz);
    b.set_aa_box(r.flag());
    b
}

/// `shape pre triangulation optimal tolerance`: AddOBB into the box, then the box.
pub fn obb_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mut b = initial_obb(&mut r);
        let (t, o, g) = (r.flag(), r.flag(), r.flag());
        attempt(|| BRepBndLib::add_obb(&s.shape, &mut b, t, o, g));
        print_obb(&b);
        Ok(())
    });
    end();
}
