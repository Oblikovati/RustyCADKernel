//! The presentation builders and the mesh, the twin of tools/oracle/tkmeshvs_probe_build.cxx.
use super::display_driver::SensitiveDisplayDriver;
use super::print::{Reader, aspect, attempt, color, end, flag, int, pnt, real, structure, trace};
use super::source::{MV_ELEMENTS, mv_hmap, mv_map, mv_source};
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::selection_policy::*;
use martensite_tkmeshvs::two_colors::bind_two_colors;
use martensite_tkmeshvs::{
    MeshVSDataSource, MeshVSDrawerAttribute as Key, MeshVSDrawerSource,
    MeshVSElementalColorPrsBuilder, MeshVSMesh, MeshVSMeshMembers, MeshVSMeshPrsBuilder,
    MeshVSMeshSource, MeshVSNodalColorPrsBuilder, MeshVSPackedMap, MeshVSPrsBuilder,
    MeshVSPrsBuilderMembers, MeshVSTextPrsBuilder, MeshVSVectorPrsBuilder,
    vector_prs_builder::calculate_arrow,
};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::quantity::{Color, ColorSpace, NamedColor};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectMembers, PrsMgrPresentationManager,
};
use rustycad_tkv3d::select_mgr::SelectMgrSelectableObject;
use rustycad_tkv3d::v3d::V3dViewer;

pub fn named(name: NamedColor) -> Color {
    Color::named(name)
}

/// The drawer changes of a request, bit by bit.
pub fn mv_drawer(d: &Handle<dyn MeshVSDrawerSource>, bits: i32) {
    let d = &**d.get().expect("drawer");
    let set_b = |k: Key, v: bool| d.set_boolean(k as i32, v);
    if bits & 1 != 0 {
        set_b(Key::Reflection, false);
    }
    if bits & 2 != 0 {
        set_b(Key::SmoothShading, true);
    }
    if bits & 4 != 0 {
        set_b(Key::ShowEdges, false);
    }
    if bits & 8 != 0 {
        set_b(Key::IsAllowOverlapped, true);
    }
    if bits & 16 != 0 {
        set_b(Key::ColorReflection, true);
    }
    if bits & 32 != 0 {
        set_b(Key::SupressBackFaces, true);
    }
    if bits & 64 != 0 {
        set_b(Key::DisplayNodes, false);
    }
    if bits & 128 != 0 {
        d.set_double(Key::ShrinkCoeff as i32, 0.5);
    }
    if bits & 256 != 0 {
        d.set_integer(Key::MaxFaceNodes as i32, 6);
    }
    if bits & 512 != 0 {
        let mut m = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
        m.set_transparency(0.5).expect("valid transparency");
        d.set_material(Key::FrontMaterial as i32, &m);
    }
    if bits & 1024 != 0 {
        d.set_integer(Key::EdgeType as i32, 1);
        d.set_integer(Key::BeamType as i32, 2);
    }
    if bits & 2048 != 0 {
        d.set_ascii_string(Key::TextFont as i32, &AsciiString::new("Arial"));
        d.set_integer(Key::TextFontAspect as i32, 0);
        d.set_integer(Key::TextStyle as i32, 1);
    }
    if bits & 4096 != 0 {
        d.set_double(Key::EdgeWidth as i32, 2.5);
        d.set_double(Key::BeamWidth as i32, 4.);
    }
    if bits & 8192 != 0 {
        d.set_integer(Key::MaxFaceNodes as i32, 4);
    }
    if bits & 16384 != 0 {
        set_b(Key::SelectableAuto, true);
    }
}

pub fn mv_ids(mask: i32, first: i32, count: i32) -> Handle<HPackedMapOfInteger> {
    if mask < 0 {
        return Handle::default();
    }
    let map = Handle::new(HPackedMapOfInteger::default());
    for i in 0..count {
        if mask & (1 << i) != 0 {
            map.change_map().add(first + i);
        }
    }
    map
}
fn packed(mask: i32, first: i32, count: i32) -> MeshVSPackedMap {
    let mut map = MeshVSPackedMap::default();
    map.assign(&mv_ids(mask, first, count).get().expect("ids").map());
    map
}
fn node_color(i: i32) -> Color {
    let f = f64::from(i);
    Color::new(f / 14., 1. - f / 14., 0.5, ColorSpace::Rgb).expect("valid color")
}

pub fn mv_builder(
    mesh: &Handle<dyn MeshVSMeshSource>,
    kind: i32,
    variant: i32,
) -> Handle<dyn MeshVSPrsBuilder> {
    match kind {
        0 => Handle::new(MeshVSMeshPrsBuilder::with_defaults(mesh)).cast(),
        1 => nodal(mesh, variant).cast(),
        2 => elemental(mesh, variant).cast(),
        3 => {
            let b = MeshVSTextPrsBuilder::with_defaults(mesh, 12., &named(NamedColor::RED));
            for i in 1..=14 {
                b.set_text(false, i, &AsciiString::new(&format!("N{i}")));
            }
            for i in 101..=108 {
                if variant != 1 || i != 103 {
                    b.set_text(true, i, &AsciiString::new(&format!("E{i}")));
                }
            }
            Handle::new(b).cast()
        }
        4 => {
            let b = MeshVSVectorPrsBuilder::with_defaults(mesh, 2., &named(NamedColor::GREEN));
            b.set_simple_prs_mode(variant & 1 != 0);
            if variant & 2 != 0 {
                b.set_simple_prs_params(3., 0.7, 0.9);
            }
            for i in 1..=14 {
                let f = f64::from(i);
                let v = if i == 4 {
                    Vector3::new(0., 0., 0.)
                } else {
                    Vector3::new(0.1 * f, -0.05 * f, 0.2)
                };
                b.set_vector(false, i, &v);
            }
            for i in 101..=108 {
                b.set_vector(true, i, &Vector3::new(0., 0., 0.1 * f64::from(i - 100)));
            }
            Handle::new(b).cast()
        }
        _ => Handle::new(MeshVSMeshPrsBuilder::new(
            mesh,
            MESHVS_DMF_OCC_MASK | MESHVS_DMF_DEFORMED_MASK,
            &Handle::default(),
            -1,
            MESHVS_BP_MESH,
        ))
        .cast(),
    }
}

fn nodal(mesh: &Handle<dyn MeshVSMeshSource>, variant: i32) -> Handle<MeshVSNodalColorPrsBuilder> {
    let b = MeshVSNodalColorPrsBuilder::with_defaults(mesh);
    if variant == 1 || variant == 3 {
        b.use_texture(true);
        let mut map = Sequence::default();
        map.append(named(NamedColor::RED));
        if variant == 1 {
            map.append(named(NamedColor::GREEN));
            map.append(named(NamedColor::BLUE1));
        }
        b.set_color_map(&map);
        b.set_invalid_color(&named(NamedColor::BLACK));
        for i in 1..=14 {
            b.set_texture_coord(i, f64::from(i) / 14. - 0.1);
        }
    } else {
        for i in 1..=14 {
            if variant != 2 || i != 5 {
                b.set_color(i, &node_color(i));
            }
        }
    }
    Handle::new(b)
}

fn elemental(
    mesh: &Handle<dyn MeshVSMeshSource>,
    variant: i32,
) -> Handle<MeshVSElementalColorPrsBuilder> {
    let b = MeshVSElementalColorPrsBuilder::with_defaults(mesh);
    if variant != 1 {
        let c = [
            NamedColor::RED,
            NamedColor::RED,
            NamedColor::BLUE1,
            NamedColor::RED,
            NamedColor::GREEN,
            NamedColor::BLUE1,
            NamedColor::RED,
            NamedColor::YELLOW,
        ];
        for (i, name) in c.into_iter().enumerate() {
            if variant == 0 || MV_ELEMENTS[i].kind != MESHVS_ET_FACE {
                b.set_color1(101 + i as i32, &named(name));
            }
        }
    }
    if variant != 0 {
        b.set_color2_colors(101, &named(NamedColor::RED), &named(NamedColor::BLUE1));
        b.set_color2_colors(102, &named(NamedColor::RED), &named(NamedColor::BLUE1));
        b.set_color2_colors(103, &named(NamedColor::GREEN), &named(NamedColor::GREEN));
        b.set_color2(
            108,
            &bind_two_colors(named(NamedColor::YELLOW), named(NamedColor::CYAN1)),
        );
    }
    Handle::new(b)
}

pub fn new_mesh(overlapped: bool, variant: i32) -> Handle<MeshVSMesh> {
    let mesh = MeshVSMesh::new(overlapped).expect("mesh");
    mesh.set_data_source(&mv_source(variant).cast::<dyn MeshVSDataSource>());
    mesh
}

/// `mv_build Kind Variant Mode IsElement IdMask DrawerBits HiddenNodes HiddenElems ExclMask Excluding`.
pub fn build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, variant, mode, is_element) = (r.integer(), r.integer(), r.integer(), r.flag());
    let (id_mask, bits, hidden_nodes, hidden_elems, excl, excluding) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let (driver, display) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let mesh = new_mesh(false, 3);
    mv_drawer(&mesh.get_drawer(), bits);
    mesh.set_hidden_nodes(&mv_ids(hidden_nodes, 1, 14));
    mesh.set_hidden_elems(&mv_ids(hidden_elems, 101, 8));
    let b = mv_builder(&mesh.clone().cast(), kind, variant);
    if excluding >= 0 {
        b.set_excluding(excluding != 0);
    }
    let prs: Handle<dyn Graphic3dStructureSource> =
        Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
            .unwrap()
            .cast();
    let (first, count) = if is_element { (101, 8) } else { (1, 14) };
    let ids = packed(id_mask, first, count);
    let mut excluded = packed(excl, first, count);
    int(b.get_id());
    int(b.get_priority());
    int(b.get_flags());
    flag(b.is_excluding_on());
    flag(b.test_flags(mode));
    attempt(|| b.build(&prs, &ids, &mut excluded, is_element, mode));
    trace(&display);
    mv_map(&excluded);
    structure(&prs);
    build_extras(&mesh, &b, kind);
    end();
}

fn build_extras(mesh: &Handle<MeshVSMesh>, b: &Handle<dyn MeshVSPrsBuilder>, kind: i32) {
    if kind == 2 {
        let e = b.cast::<MeshVSElementalColorPrsBuilder>();
        let e = e.get().expect("elemental");
        int(e.get_colors1().extent() as i32);
        int(e.get_colors2().extent() as i32);
        let mut c = Color::default();
        flag(e.get_color1(103, &mut c));
        color(&c);
        let (mut c1, mut c2) = (Color::default(), Color::default());
        flag(e.get_color2_colors(101, &mut c1, &mut c2));
        color(&c1);
        color(&c2);
    }
    if kind == 1 {
        let n = b.cast::<MeshVSNodalColorPrsBuilder>();
        let n = n.get().expect("nodal");
        flag(n.is_use_texture());
        int(n.get_colors().extent() as i32);
        int(n.get_color_map().length());
        color(&n.get_invalid_color());
        real(n.get_texture_coord(3));
        real(n.get_texture_coord(99));
        let shading = mesh.attributes().shading_aspect();
        flag(shading.is_null());
        if let Some(shading) = shading.get() {
            aspect(&shading.aspect().cast());
        }
    }
    if kind == 4 {
        let v = b.cast::<MeshVSVectorPrsBuilder>();
        let v = v.get().expect("vector");
        let (mut mn, mut mx) = (-1., -1.);
        v.get_min_max_vector_value(false, &mut mn, &mut mx);
        real(mn);
        real(mx);
        let mut a = [Point3::default(); 8];
        real(calculate_arrow(&mut a, 2., 0.1));
        for p in &a {
            pnt(p);
        }
    }
}

/// `mv_compute Mode BuilderSet DrawerBits Overlapped`.
pub fn compute_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, set, bits, overlapped) = (r.integer(), r.integer(), r.integer(), r.flag());
    let (driver, display) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let mesh = new_mesh(overlapped, 3);
    mv_drawer(&mesh.get_drawer(), bits);
    for k in 0..5 {
        if set & (1 << k) != 0 {
            mesh.add_builder(&mv_builder(&mesh.clone().cast(), k, 0), k == 0);
        }
    }
    for m in [-1, 0, 1, 2, 3, 4, 8, 16, 32, 64, 128, 1024] {
        flag(mesh.accept_display_mode(m));
    }
    for i in 1..=mesh.get_builders_count() {
        let b = mesh.get_builder(i);
        int(b.get_id());
        int(b.get_priority());
    }
    let prs: Handle<dyn Graphic3dStructureSource> =
        Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
            .unwrap()
            .cast();
    attempt(|| mesh.compute(&Handle::<PrsMgrPresentationManager>::default(), &prs, mode));
    trace(&display);
    structure(&prs);
    end();
}

fn builders_state(mesh: &Handle<MeshVSMesh>) {
    int(mesh.get_builders_count());
    for i in 1..=mesh.get_builders_count() {
        let b = mesh.get_builder(i);
        int(b.get_id());
        int(b.get_priority());
    }
    let hilighter = mesh.get_hilighter();
    int(hilighter.get().map_or(-1, |b| b.get_id()));
    int(mesh.get_free_id());
    let text = mesh.find_builder(MeshVSTextPrsBuilder::static_type());
    int(text.get().map_or(-1, |b| b.get_id()));
    let any = mesh.find_builder(<dyn MeshVSPrsBuilder as RuntimeTyped>::static_type());
    int(any.get().map_or(-1, |b| b.get_id()));
    flag(mesh.get_builder(0).is_null());
    flag(mesh.get_builder_by_id(7).is_null());
    for i in [1, 2, 7, 14] {
        flag(mesh.is_hidden_node(i));
        flag(mesh.is_selectable_node(i));
    }
    for i in [101, 104, 106] {
        flag(mesh.is_hidden_elem(i));
        flag(mesh.is_selectable_elem(i));
    }
    mv_hmap(&mesh.get_selectable_nodes());
    int(mesh.get_mesh_sel_method() as i32);
}

/// `mv_mesh_api Count Op...`.
pub fn mesh_api_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let count = r.integer();
    let mesh = new_mesh(false, 0);
    let parent: Handle<dyn MeshVSMeshSource> = mesh.clone().cast();
    int(mesh.display_mode());
    int(mesh.hilight_mode());
    flag(mesh.is_auto_hilight());
    color(&mesh.attributes().color());
    flag(mesh.has_color());
    flag(mesh.has_material());
    for _ in 0..count {
        match r.integer() {
            0 => mesh.add_builder(
                &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&parent)).cast(),
                false,
            ),
            1 => mesh.add_builder(
                &Handle::new(MeshVSNodalColorPrsBuilder::with_defaults(&parent)).cast(),
                false,
            ),
            2 => mesh.add_builder(
                &Handle::new(MeshVSTextPrsBuilder::with_defaults(
                    &parent,
                    10.,
                    &named(NamedColor::RED),
                ))
                .cast(),
                true,
            ),
            3 => mesh.add_builder(
                &Handle::new(MeshVSMeshPrsBuilder::new(
                    &parent,
                    MESHVS_DMF_OCC_MASK,
                    &Handle::default(),
                    7,
                    MESHVS_BP_ELEM_COLOR,
                ))
                .cast(),
                false,
            ),
            4 => mesh.remove_builder(1),
            5 => mesh.remove_builder_by_id(7),
            6 => flag(mesh.set_hilighter_at(2)),
            7 => flag(mesh.set_hilighter_by_id(0)),
            8 => mesh.remove_builder(5),
            9 => mesh.set_hilighter(&Handle::default()),
            10 => mesh.set_hidden_nodes(&mv_ids(0x7, 1, 14)),
            11 => mesh.set_hidden_elems(&mv_ids(0x9, 101, 8)),
            12 => mesh.update_selectable_nodes(),
            13 => mesh.set_selectable_nodes(&Handle::default()),
            14 => mesh
                .get_drawer()
                .get()
                .expect("drawer")
                .set_boolean(Key::SelectableAuto as i32, true),
            15 => mesh.set_hidden_nodes(&Handle::default()),
            16 => mesh.set_mesh_sel_method(MeshVSMeshSelectionMethod::Box),
            17 => mesh.add_builder(&Handle::default(), false),
            18 => mesh.set_hidden_nodes(&mv_ids(0, 1, 14)),
            19 => mesh
                .get_drawer()
                .get()
                .expect("drawer")
                .set_integer(Key::MaxFaceNodes as i32, 2),
            _ => {}
        }
        builders_state(&mesh);
    }
    end();
}
