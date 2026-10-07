//! The Graphic3d driver model on the null driver, the twin of
//! tools/oracle/tkservice_probe_driver.cxx (`sv_scene`).
use super::camera::read_camera;
use super::common::{Reader, end, flag, int, real, text};
use super::tried;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis2, Transform3, Vector3, dz};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectTypeOfTriedronPosition, AspectWindow};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_DISPLAY_PRIORITY_NB, Graphic3dArrayOfPoints, Graphic3dBndBox3d,
    Graphic3dCStructureMembers, Graphic3dCView, Graphic3dCViewComputed, Graphic3dCViewMembers,
    Graphic3dCViewStructures, Graphic3dCamera, Graphic3dCullingTool, Graphic3dDisplayPriority as P,
    Graphic3dGraphicDriver, Graphic3dGraphicDriverMembers, Graphic3dGroupMembers, Graphic3dLayer,
    Graphic3dRenderingParamsFrustumCulling as Culling, Graphic3dStructure,
    Graphic3dStructureBounds, Graphic3dStructureDisplay, Graphic3dStructureManager,
    Graphic3dStructureMembers, Graphic3dStructureNetwork, Graphic3dStructureSource, Graphic3dText,
    Graphic3dTransModeFlags, Graphic3dTransformPers, Graphic3dTransformPersSource,
    Graphic3dTypeOfConnection, Graphic3dTypeOfStructure,
};
use rustycad_tkservice::null_driver::{NullGraphicDriver, NullView};

/// Prints a `Bnd_Box`: void and whole flags, then the limits of a non-void box.
fn bnd(bounds: &BoundingBox3) {
    flag(bounds.is_void());
    flag(bounds.is_whole());
    if let Ok(limits) = bounds.get() {
        limits.into_iter().for_each(real);
    }
}

/// Prints a BVH box: valid flag and corners.
fn bvh_box(bounds: &Graphic3dBndBox3d) {
    flag(bounds.is_valid());
    if bounds.is_valid() {
        let (min, max) = (bounds.corner_min(), bounds.corner_max());
        [min.x, min.y, min.z, max.x, max.y, max.z]
            .into_iter()
            .for_each(real);
    }
}

struct Scene {
    manager: Handle<Graphic3dStructureManager>,
    view: Handle<dyn Graphic3dCView>,
    structs: Vec<Handle<dyn Graphic3dStructureSource>>,
    size: [i32; 2],
}

fn structure_at(scene: &Scene, r: &mut Reader<'_>) -> Handle<dyn Graphic3dStructureSource> {
    let index = r.integer();
    usize::try_from(index)
        .ok()
        .and_then(|i| scene.structs.get(i))
        .cloned()
        .unwrap_or_default()
}

fn persistence(r: &mut Reader<'_>) -> Result<Handle<dyn Graphic3dTransformPersSource>, Failure> {
    let (mode, point) = (r.integer(), r.pnt());
    let flags = Graphic3dTransModeFlags(mode);
    let pers = match mode {
        0 => return Ok(Handle::default()),
        64 | 32 => Graphic3dTransformPers::with_corner(
            flags,
            AspectTypeOfTriedronPosition::LEFT_LOWER,
            Vec2::new(point.x() as i32, point.y() as i32),
        )?,
        _ => Graphic3dTransformPers::with_point(flags, &point)?,
    };
    Ok(Handle::new(pers).cast())
}

fn cull(scene: &Scene, state: i32) {
    let mut tool = Graphic3dCullingTool::default();
    tool.set_view_volume(
        &scene.view.camera(),
        &rustycad_tkernel::ncollection::Mat4::identity(),
    );
    tool.set_viewport_size(scene.size[0], scene.size[1], 1.0);
    tool.cache_clip_pts_projections();
    let state = [Culling::Off, Culling::On, Culling::NoUpdate][state.clamp(0, 2) as usize];
    for layer in scene.view.layers().iter().filter_map(Handle::get) {
        tried(layer.update_culling(scene.view.identification(), &tool, state));
    }
}

fn create(scene: &mut Scene, r: &mut Reader<'_>) {
    let structure: Handle<dyn Graphic3dStructureSource> =
        Graphic3dStructure::new(&scene.manager, &Handle::default())
            .map(|s| s.cast())
            .unwrap_or_default();
    let (min, max) = (r.pnt(), r.pnt());
    if let Ok(group) = structure.new_group() {
        group.set_min_max_values([min.x(), min.y(), min.z(), max.x(), max.y(), max.z()]);
    }
    scene.structs.push(structure);
}

fn add_text(s: &dyn Graphic3dStructureSource, r: &mut Reader<'_>) {
    let text = Graphic3dText::new(10.0);
    let point = r.pnt();
    text.set_position(&point);
    if r.flag()
        && let Ok(axes) = Axis2::new(point, dz())
    {
        text.set_orientation(&axes);
    }
    if let Ok(group) = s.current_group() {
        group.add_text(&Handle::new(text), true);
    }
}

fn add_points(s: &dyn Graphic3dStructureSource, r: &mut Reader<'_>) {
    let count = r.integer();
    let points: Vec<_> = (0..count).map(|_| r.pnt()).collect();
    let Ok(array) = Graphic3dArrayOfPoints::new(count, 0) else {
        return;
    };
    for point in &points {
        let _ = array.add_vertex(point.x() as f32, point.y() as f32, point.z() as f32);
    }
    if let Ok(group) = s.new_group() {
        group.add_primitive_array(&Handle::new(array).cast(), true);
    }
}

fn structure_op(
    scene: &Scene,
    op: i32,
    handle: &Handle<dyn Graphic3dStructureSource>,
    r: &mut Reader<'_>,
) {
    let Some(s) = handle.get() else { return };
    match op {
        2 => s.display(),
        3 => s.erase(),
        4 => tried(s.set_display_priority(priority(r.integer()))),
        5 => s.set_z_layer(r.integer()),
        6 => s.set_infinite_state(r.flag()),
        7 => {
            let other = structure_at(scene, r);
            let connection = if r.integer() == 0 {
                Graphic3dTypeOfConnection::Ancestor
            } else {
                Graphic3dTypeOfConnection::Descendant
            };
            s.connect(&other, connection, r.flag());
        }
        8 => s.disconnect(&structure_at(scene, r)),
        9 => s.set_visible(r.flag()),
        10 => s.highlight(&Handle::default(), true),
        11 => s.unhighlight(),
        12 => {
            if let Ok(pers) = persistence(r) {
                s.set_transform_persistence(&pers);
            }
        }
        13 => {
            let mut trsf = Transform3::default();
            let v = r.xyz();
            trsf.set_translation(Vector3::new(v.x(), v.y(), v.z()));
            s.set_transformation(&Handle::new(Datum3::new(trsf)));
        }
        14 => s.remove(),
        15 => s.set_mutable(r.flag()),
        17 => add_text(s, r),
        18 => add_points(s, r),
        20 => s.set_is_for_highlight(r.flag()),
        21 => s.clear(r.flag()),
        22 => {
            let index = r.integer();
            let groups = s.groups();
            if index >= 1
                && index as usize <= groups.len()
                && let Some(group) = groups[index as usize - 1].get()
            {
                group.remove();
            }
        }
        23 => s.set_visual(visual(r.integer())),
        _ => {}
    }
}

fn priority(value: i32) -> P {
    const ALL: [P; 11] = [
        P::Bottom,
        P::AlmostBottom,
        P::Below2,
        P::Below1,
        P::Below,
        P::Normal,
        P::Above,
        P::Above1,
        P::Above2,
        P::Highlight,
        P::Topmost,
    ];
    usize::try_from(value)
        .ok()
        .and_then(|i| ALL.get(i).copied())
        .unwrap_or(P::Invalid)
}

fn visual(value: i32) -> Graphic3dTypeOfStructure {
    [
        Graphic3dTypeOfStructure::Wireframe,
        Graphic3dTypeOfStructure::Shading,
        Graphic3dTypeOfStructure::Computed,
        Graphic3dTypeOfStructure::All,
    ][value.clamp(0, 3) as usize]
}

fn scene_op(scene: &mut Scene, r: &mut Reader<'_>) {
    match r.integer() {
        1 => create(scene, r),
        19 => cull(scene, r.integer()),
        24 => scene.view.set_computed_mode(r.flag()),
        25 => {
            if r.flag() {
                scene.view.activate()
            } else {
                scene.view.deactivate()
            }
        }
        op => {
            let handle = structure_at(scene, r);
            structure_op(scene, op, &handle, r);
        }
    }
}

fn print_structure(scene: &Scene, handle: &Handle<dyn Graphic3dStructureSource>) {
    let Some(s) = handle.get() else { return };
    flag(s.is_deleted());
    if s.is_deleted() {
        return;
    }
    let cstructure = s.cstructure();
    let Some(c) = cstructure.get() else { return };
    int(s.identification());
    [s.is_displayed(), s.is_visible(), s.is_highlighted()]
        .into_iter()
        .for_each(flag);
    int(s.display_priority() as i32);
    int(s.get_z_layer());
    [s.is_empty(), s.is_infinite(), s.is_transformed()]
        .into_iter()
        .for_each(flag);
    int(s.number_of_groups());
    [
        c.is_culled(),
        c.cstructure_base().fields.read().is_2d_text,
        c.is_always_rendered(),
        scene.view.is_displayed(handle),
    ]
    .into_iter()
    .for_each(flag);
    bvh_box(&c.bounding_box());
    bnd(&s.min_max_values(false));
    bnd(&s.min_max_values(true));
    for group in s.groups().iter().filter_map(Handle::get) {
        group.min_max_values().into_iter().for_each(real);
        flag(group.is_empty());
    }
    let mut set = Vec::new();
    s.descendants(&mut set);
    int(set.len() as i32);
}

fn print_layer(scene: &Scene, layer: &Graphic3dLayer) {
    int(layer.layer_id());
    int(layer.nb_structures());
    int(layer.nb_structures_not_culled());
    int(layer.nb_of_transform_persistence_objects());
    flag(layer.is_immediate());
    for p in 0..GRAPHIC3D_DISPLAY_PRIORITY_NB {
        let map = layer.structures(priority(p));
        int(map.extent() as i32);
        for weak in map.iter() {
            int(weak.upgrade().get().map_or(-1, |c| c.identification()));
        }
    }
    let camera: Handle<Graphic3dCamera> = scene.view.camera();
    for aux in [false, true] {
        super::print::show(
            layer.bounding_box(scene.view.identification(), &camera, scene.size, aux),
            |b| bnd(&b),
        );
    }
    super::print::show(
        layer.consider_zoom_persistence_objects(scene.view.identification(), &camera, scene.size),
        real,
    );
}

/// Replaces every printed address ("0x...") by "?", then masks the uninitialised render
/// parameter and camera tile values, as `sv_mask_dump`.
fn mask_dump(text: &[u8]) -> Vec<u8> {
    let head = b"\": \"0x";
    let mut out = text.to_vec();
    let mut from = 0;
    while let Some(found) = out[from..].windows(head.len()).position(|w| w == head) {
        let start = from + found + 4;
        let end = out[start..]
            .iter()
            .position(|&b| b == b'"')
            .map_or(out.len(), |e| start + e);
        out.splice(start..end, *b"?");
        from += found + 1;
    }
    let masked =
        super::params::masked(&super::params::masked(&out, "ChartSize"), "Params.Params3d");
    super::params::masked(
        &super::params::masked(&masked, "Tile"),
        "WorldViewProjState",
    )
}

/// Prints a masked dump.
fn dump_masked(write: impl FnOnce(&mut DumpStream) -> std::io::Result<()>) {
    let mut stream = DumpStream::default();
    match write(&mut stream) {
        Ok(()) => text(&mask_dump(stream.bytes())),
        Err(error) => print!("dump-error:{error} "),
    }
}

/// The dumps of the live structures, the driver, the view and (with one displayed and one
/// highlighted structure at most) the manager.
fn print_dumps(scene: &Scene, driver: &Handle<dyn Graphic3dGraphicDriver>) {
    for structure in scene
        .structs
        .iter()
        .filter_map(Handle::get)
        .filter(|s| !s.is_deleted())
    {
        dump_masked(|s| structure.dump_json(s, -1));
    }
    if let Some(driver) = driver.get() {
        dump_masked(|s| driver.dump_json(s, -1));
    }
    if let Some(view) = scene.view.get() {
        dump_masked(|s| view.cview_dump_json(s, -1));
    }
    let manager = &scene.manager;
    if manager.displayed_structures().len() <= 1 && manager.highlighted_structures().len() <= 1 {
        dump_masked(|s| manager.dump_json(s, -1));
    }
}

/// `sv_scene`: a scene script on the null driver, then its structures, view and layers.
pub fn scene_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let camera = Handle::new(read_camera(&mut r));
    let size = [r.integer(), r.integer()];
    let driver = NullGraphicDriver::new_handle();
    let Ok(manager) = Graphic3dStructureManager::new(&driver) else {
        return;
    };
    let manager = Handle::new(manager);
    let Ok(view) = driver.create_view(&manager) else {
        return;
    };
    let window = AspectNeutralWindow::default();
    window.set_size(size[0], size[1]);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.cast::<NullView>().set_null_window(&window);
    view.set_camera(&camera);
    view.activate();
    let mut scene = Scene {
        manager,
        view,
        structs: Vec::new(),
        size,
    };
    for _ in 0..r.integer() {
        scene_op(&mut scene, &mut r);
    }
    int(scene.structs.len() as i32);
    scene
        .structs
        .iter()
        .for_each(|s| print_structure(&scene, s));
    super::print::show(scene.view.min_max_values(false), |b| bnd(&b));
    super::print::show(scene.view.min_max_values(true), |b| bnd(&b));
    super::print::show(scene.view.consider_zoom_persistence_objects(), real);
    int(scene.view.number_of_displayed_structures());
    for layer in scene.view.layers().iter().filter_map(Handle::get) {
        print_layer(&scene, layer);
    }
    print_dumps(&scene, &driver);
    end();
}
