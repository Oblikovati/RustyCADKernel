//! AIS_MediaPlayer without media input, the twin of
//! tools/oracle/tkv3d_probe_ais_media_player.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::Graphic3dTransformPersMembers;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::v3d::V3dViewer;

fn state(p: &AisMediaPlayer) {
    int(p.z_layer());
    flag(p.is_infinite());
    let pers = p.transform_persistence();
    flag(!pers.is_null());
    if let Some(pers) = pers.get() {
        int(pers.mode().0);
        int(pers.corner_2d().map_or(-1, |c| c.0));
    }
    for mode in -1..=1 {
        flag(p.accept_display_mode(mode));
    }
    flag(p.player_context().is_null());
    real(p.duration());
    let fields = p.fields.read().clone();
    int(fields.frame_size.x());
    int(fields.frame_size.y());
    int(fields.frame_bottom_left.x());
    int(fields.frame_bottom_left.y());
    int(p.presentations().len() as i32);
}

/// `v3_ais_media_player X Y W H Count Op...`.
pub fn media_player_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x, y, w, h, count) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let p = AisMediaPlayer::new().unwrap();
    let dynamic: Handle<dyn AisInteractiveObject> = p.clone().into_cast();
    state(&p);
    for _ in 0..count {
        match r.integer() {
            0 => attempt(|| p.update_size(Vec2::new(x, y), Vec2::new(w, h)).map(flag)),
            1 => attempt(|| p.present_frame(Vec2::new(x, y), Vec2::new(w, h)).map(flag)),
            2 => ctx.display(&dynamic, false).unwrap(),
            3 => p.set_close_player(),
            4 => {
                let s = Handle::new(SelectMgrSelection::new(0));
                p.compute_selection(&s, 0).unwrap();
                int(s.entities().len() as i32);
                for e in s.entities() {
                    let sensitive = e.base_sensitive();
                    int(sensitive.nb_sub_elements());
                    int(
                        rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers::owner_id(
                            &*sensitive,
                        )
                        .priority(),
                    );
                }
            }
            5 => p.play_pause(),
            _ => p.open_input(&AsciiString::default(), false),
        }
        int(ctx.display_status(&dynamic) as i32);
        state(&p);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
