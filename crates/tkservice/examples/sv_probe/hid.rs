//! WNT_HIDSpaceMouse decoding, the twin of tools/oracle/tkservice_probe_hid.cxx.
use super::common::{Reader, end, flag, int, real};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::wnt::WntHidSpaceMouse;

fn print_vector((vector, idle): (Vec3<f64>, bool)) {
    real(vector.x);
    real(vector.y);
    real(vector.z);
    flag(idle);
}

/// `sv_hid ProductId Quadric N byte...`.
pub fn hid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let product_id = r.real() as u64;
    let quadric = r.flag();
    let count = r.integer();
    let bytes: Vec<u8> = (0..count).map(|_| r.integer() as u8).collect();
    let mouse = WntHidSpaceMouse::new(product_id, &bytes);
    flag(WntHidSpaceMouse::is_known_product(product_id));
    flag(mouse.is_translation());
    flag(mouse.is_rotation());
    flag(mouse.is_key_state());
    print_vector(mouse.translation(quadric));
    print_vector(mouse.rotation(quadric));
    int(i32::from(mouse.raw_value_range()));
    if mouse.is_key_state() && count >= 5 {
        real(f64::from(mouse.key_state()));
    }
    (0..34u16).for_each(|bit| int(mouse.hid_to_space_key(bit) as i32));
    end();
}
