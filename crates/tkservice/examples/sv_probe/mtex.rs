//! Graphic3d_MediaTexture and Graphic3d_MediaTextureSet without FFmpeg, the twin of
//! tools/oracle/tkservice_probe_mtex.cxx.
use super::common::{Reader, end, flag, int, real, text};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dMediaTexture, Graphic3dMediaTextureSet, Graphic3dShaderProgram, Graphic3dTextureRoot,
    Graphic3dTextureRootMembers,
};
use rustycad_tkservice::image::ImageSupportedFormats;

fn program(program: &Handle<Graphic3dShaderProgram>) {
    let Some(program) = program.get() else { return };
    text(program.header().as_bytes());
    int(program.nb_lights_max());
    int(program.nb_clip_planes_max());
    for shader in program.shader_objects().iter().filter_map(Handle::get) {
        int(shader.shader_type() as i32);
        text(shader.source().as_bytes());
    }
}

fn textures(set: &Graphic3dMediaTextureSet) {
    int(set.size());
    for texture in set.textures() {
        let texture = texture.cast::<Graphic3dMediaTexture>();
        let Some(texture) = texture.get() else {
            continue;
        };
        let params = texture.get_params();
        if let Some(params) = params.get() {
            int(params.texture_unit() as i32);
            flag(params.is_modulate());
            flag(params.is_repeat());
            int(params.filter() as i32);
        }
        flag(texture.frame().is_null());
        flag(
            texture
                .get_image(&Handle::new(ImageSupportedFormats::default()))
                .is_null(),
        );
    }
}

fn frame_queue(set: &Graphic3dMediaTextureSet, nb_locks: i32) {
    let queue = set.frame_queue();
    for index in 0..nb_locks {
        let frame = queue.lock_frame();
        flag(frame.is_null());
        let Some(value) = frame.get() else { continue };
        flag(value.is_locked());
        flag(queue.lock_frame().is_null());
        if index % 2 == 0 {
            queue.release_frame(&frame);
            flag(value.is_locked());
        }
    }
}

/// `sv_mtex`: a media texture set, its programs and frame queue, then the swap without player.
pub fn mtex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let nb_locks = r.integer();
    let Ok(set) = Graphic3dMediaTextureSet::new() else {
        return;
    };
    textures(&set);
    program(&set.shader_yuv());
    program(&set.shader_yuvj());
    flag(set.shader_program().is_null());
    flag(set.is_planar_yuv());
    flag(set.is_full_range_yuv());
    frame_queue(&set, nb_locks);
    flag(set.swap_frames());
    flag(set.player_context().is_null());
    set.open_input(&AsciiString::default(), false);
    text(set.input().as_bytes());
    flag(set.player_context().is_null());
    let size = set.frame_size();
    int(size.x);
    int(size.y);
    real(set.progress());
    real(set.duration());
    end();
}
