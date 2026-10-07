//! Graphic3d_ClipPlane, the twin of tools/oracle/tkservice_probe_clip.cxx.
use super::common::{Reader, dir, end, flag, int, pnt, real};
use super::material::material_name;
use super::tried;
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkservice::aspect::AspectHatchStyle;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsRenderMembers, Graphic3dBndBox3d, Graphic3dClipPlane, Graphic3dMaterialAspect,
    Graphic3dTexture2D, Graphic3dTextureMap,
};

fn vec4(v: &Vec4<f64>) {
    [v.x, v.y, v.z, v.w].into_iter().for_each(real);
}
fn read_vec4(r: &mut Reader<'_>) -> Vec4<f64> {
    let [x, y, z, w] = [0; 4].map(|_| r.real());
    Vec4::new(x, y, z, w)
}

/// `Kind A B C D` as the native `sv_clip_make`.
fn make(kind: i32, e: Vec4<f64>) -> Result<Graphic3dClipPlane, Failure> {
    let from_plane = || Plane3::from_equation(e.x, e.y, e.z, e.w);
    match kind {
        1 => Graphic3dClipPlane::from_equation(e),
        2 => Graphic3dClipPlane::from_plane(&from_plane()?),
        _ => {
            let plane = Graphic3dClipPlane::default();
            match kind {
                0 => plane.set_equation(e)?,
                3 => plane.set_plane(&from_plane()?)?,
                _ => {}
            }
            Ok(plane)
        }
    }
}

/// The handle of a built plane, or `None` after printing the raised exception type.
fn made(plane: Result<Graphic3dClipPlane, Failure>) -> Option<Handle<Graphic3dClipPlane>> {
    let mut built = None;
    super::print::show(plane, |plane| built = Some(Handle::new(plane)));
    built
}

fn state(p: &Graphic3dClipPlane) {
    vec4(&p.get_equation());
    vec4(&p.reversed_equation());
    let plane = p.to_plane();
    pnt(plane.location());
    dir(plane.axis().direction());
    dir(plane.x_axis().direction());
    dir(plane.y_axis().direction());
    int(p.m_count_equation() as i32);
    flag(p.is_on());
    flag(p.is_chain());
    int(p.nb_chain_next_planes());
}

/// Reads the chain planes and links them after `head`; returns the last plane.
fn read_chain(r: &mut Reader<'_>, head: &Handle<Graphic3dClipPlane>) -> Handle<Graphic3dClipPlane> {
    let mut last = head.clone();
    for _ in 0..r.integer() {
        let equation = read_vec4(r);
        tried(
            Graphic3dClipPlane::from_equation(equation).and_then(|plane| {
                let plane = Handle::new(plane);
                Graphic3dClipPlane::set_chain_next_plane(&last, plane.clone())?;
                last = plane;
                Ok(())
            }),
        );
    }
    last
}

fn probe_points(r: &mut Reader<'_>, head: &Graphic3dClipPlane) {
    for _ in 0..r.integer() {
        let point = read_vec4(r);
        int(head.probe_point(&point) as i32);
        int(head.probe_point_halfspace(&point) as i32);
        flag(head.is_point_out_halfspace(&point));
    }
}

fn probe_boxes(r: &mut Reader<'_>, head: &Graphic3dClipPlane) {
    for _ in 0..r.integer() {
        let [x0, y0, z0, x1, y1, z1] = [0; 6].map(|_| r.real());
        let bounds = Graphic3dBndBox3d::from_corners(Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z1));
        int(head.probe_box(&bounds) as i32);
        flag(head.probe_box_touch(&bounds));
        int(head.probe_box_halfspace(&bounds) as i32);
        flag(head.is_box_full_out_halfspace(&bounds));
        flag(head.is_box_full_in_halfspace(&bounds));
        int(head.probe_box_max_point_halfspace(&bounds) as i32);
    }
}

/// `sv_clip`: a plane, a chain after it, its point and box tests, then the chain cut.
pub fn clip_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let Some(head) = made(make(kind, read_vec4(&mut r))) else {
        return end();
    };
    state(&head);
    let last = read_chain(&mut r, &head);
    int(head.nb_chain_next_planes());
    flag(head.is_chain());
    int(last.nb_chain_next_planes());
    flag(last.chain_previous_plane().is_null());
    tried(last.set_on(true));
    probe_points(&mut r, &head);
    probe_boxes(&mut r, &head);
    cut_chain(&head);
    end();
}

fn cut_chain(head: &Handle<Graphic3dClipPlane>) {
    let second = head.chain_next_plane();
    tried(Graphic3dClipPlane::set_chain_next_plane(
        head,
        Handle::default(),
    ));
    int(head.nb_chain_next_planes());
    flag(head.is_chain());
    int(head.m_count_equation() as i32);
    if let Some(second) = second.get() {
        flag(second.chain_previous_plane().is_null());
        tried(second.set_on(false));
        flag(second.is_on());
    }
}

fn capping_state(p: &Graphic3dClipPlane) {
    let color = p.capping_color();
    [color.red(), color.green(), color.blue()]
        .into_iter()
        .for_each(real);
    int(p.capping_material().name() as i32);
    flag(p.is_hatch_on());
    let has_hatch = p.capping_custom_hatch().is_some();
    flag(has_hatch);
    if has_hatch {
        super::print::show(p.capping_hatch(), |style| int(style as i32));
    }
    flag(p.capping_texture().is_some());
    flag(p.capping_aspect().to_map_texture());
    flag(p.to_use_object_material());
    flag(p.to_use_object_texture());
    flag(p.to_use_object_shader());
    flag(p.to_use_object_properties());
    flag(p.is_capping());
    int(p.m_count_aspect() as i32);
    int(p.m_count_equation() as i32);
}

/// The hatch style of a corpus value (0..=12).
fn hatch(value: i32) -> AspectHatchStyle {
    use AspectHatchStyle as H;
    [
        H::Solid,
        H::GridDiagonal,
        H::GridDiagonalWide,
        H::Grid,
        H::GridWide,
        H::Diagonal45,
        H::Diagonal135,
        H::Horizontal,
        H::Vertical,
        H::Diagonal45Wide,
        H::Diagonal135Wide,
        H::HorizontalWide,
        H::VerticalWide,
    ][value as usize]
}

fn capping_texture(with: bool) -> Handle<dyn Graphic3dTextureMap> {
    if !with {
        return Handle::default();
    }
    let texture = Graphic3dTexture2D::from_file(&AsciiString::from("capping.png"));
    Handle::new(texture).cast::<dyn Graphic3dTextureMap>()
}

/// `R G B`: SetCappingColor of an RGB color (an out-of-range one raises).
fn set_color(r: &mut Reader<'_>, p: &Graphic3dClipPlane) {
    let [red, green, blue] = [0; 3].map(|_| r.real());
    tried(Color::new(red, green, blue, ColorSpace::Rgb).and_then(|c| p.set_capping_color(c)));
}

/// `Flag On`: sets the object material (0), texture (1) or shader (2) flag.
fn set_flag(r: &mut Reader<'_>, p: &Graphic3dClipPlane) {
    let (which, on) = (r.integer(), r.flag());
    match which {
        0 => p.set_use_object_material(on),
        1 => p.set_use_object_texture(on),
        _ => p.set_use_object_shader(on),
    }
}

/// Applies one capping operation of the corpus; returns the plane to continue with.
fn capping_op(r: &mut Reader<'_>, p: Handle<Graphic3dClipPlane>) -> Handle<Graphic3dClipPlane> {
    match r.integer() {
        1 => set_color(r, &p),
        2 => p.set_capping_material(Graphic3dMaterialAspect::new(material_name(r.integer()))),
        3 => p.set_capping_hatch(hatch(r.integer())),
        4 => p.set_capping_hatch_on(),
        5 => p.set_capping_hatch_off(),
        6 => set_flag(r, &p),
        7 => p.set_capping(r.flag()),
        8 => return p.clone_plane(),
        9 => tried(p.set_capping_texture(capping_texture(r.flag()))),
        _ => tried(p.set_equation(read_vec4(r))),
    }
    p
}

/// `sv_clip_capping`: capping operations on a default plane, the state after each.
pub fn capping_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut plane = Handle::new(Graphic3dClipPlane::default());
    for _ in 0..r.integer() {
        plane = capping_op(&mut r, plane);
        capping_state(&plane);
    }
    end();
}
