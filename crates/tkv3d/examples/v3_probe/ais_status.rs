//! Independent native status setters, list order and retained drawer identity.
use super::common::{Reader, end, flag, int};
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::{ais::AisGlobalStatus, prs3d::Prs3dDrawer};
fn status_state(status: &AisGlobalStatus, styles: &[Handle<Prs3dDrawer>]) {
    int(status.display_mode());
    flag(status.is_hilighted());
    flag(status.is_sub_intensity_on());
    int(styles
        .iter()
        .position(|s| s == &*status.hilight_style())
        .unwrap() as i32);
    let modes = status.selection_modes();
    int(modes.len() as i32);
    for mode in modes.iter() {
        int(*mode);
    }
}
pub fn ais_status_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let status = AisGlobalStatus::new();
    let styles = [
        Handle::default(),
        Handle::new(Prs3dDrawer::new()),
        Handle::new(Prs3dDrawer::new()),
    ];
    status_state(&status, &styles);
    for _ in 0..r.integer() {
        let (op, value) = (r.integer(), r.integer());
        match op {
            0 => status.set_display_mode(value),
            1 => status.set_hilight_status(value != 0),
            2 => status.set_sub_intensity(value != 0),
            3 => status.set_hilight_style(&styles[value as usize]),
            4 => flag(status.add_selection_mode(value)),
            5 => flag(status.remove_selection_mode(value)),
            6 => status.clear_selection_modes(),
            7 => flag(status.is_s_mode_in(value)),
            _ => unreachable!(),
        }
        status_state(&status, &styles);
    }
    end();
}
