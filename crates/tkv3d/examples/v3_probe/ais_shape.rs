//! AIS_Shape state, presentations, selection and HLR, the twin of
//! tools/oracle/tkv3d_probe_ais_shape.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, text};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis3, Cylinder3, Direction3, Plane3, Point2, Point3, Transform3, Vector3,
};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::prs3d::{Prs3dDrawer, Prs3dTypeOfHlr};
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkv3d::std_select::{StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use rustycad_tkv3d::v3d::V3dViewer;

fn shape_of(kind: i32) -> Shape {
    let ax = Axis3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let builder = Builder;
    match kind {
        0 => Shape::from(
            BRepBuilderAPIMakeVertex::new(Point3::new(1., 2., 3.))
                .unwrap()
                .vertex()
                .unwrap(),
        ),
        1 => Shape::from(
            BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 1., 0.))
                .unwrap()
                .edge()
                .unwrap(),
        ),
        2 => Shape::from(
            BRepBuilderAPIMakeFace::from_cylinder_bounds(
                Cylinder3::new(ax, 2.).unwrap(),
                (0., std::f64::consts::PI, 0., 3.),
            )
            .unwrap()
            .face()
            .unwrap(),
        ),
        3 | 4 => {
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            if kind == 3 {
                for part in [2, 1, 0] {
                    builder.add(&mut compound, &shape_of(part)).unwrap();
                }
            }
            Shape::from(compound)
        }
        6 => Shape::from(
            BRepBuilderAPIMakeFace::from_plane(Plane3::from_position(ax))
                .unwrap()
                .face()
                .unwrap(),
        ),
        _ => Shape::default(),
    }
}

fn rgb(color: Color) {
    real(color.red());
    real(color.green());
    real(color.blue());
}

fn state(o: &AisShape) {
    int(o.signature());
    int(o.kind_of_interactive() as i32);
    for mode in -1..=3 {
        flag(o.accept_display_mode(mode));
    }
    flag(o.accept_shape_decomposition());
    flag(o.has_color());
    flag(o.has_width());
    real(o.width());
    rgb(PrsMgrPresentableObject::color(o));
    int(o.material() as i32);
    real(o.transparency());
    flag(o.has_material());
    attempt(|| {
        let b = o.shape_bounding_box()?;
        flag(b.is_void());
        if !b.is_void() {
            for value in b.get()? {
                real(value);
            }
        }
        Ok(())
    });
    pnt2d(o.texture_repeat_uv());
    pnt2d(o.texture_origin_uv());
    pnt2d(o.texture_scale_uv());
    let (own, value, previous) = o.own_deviation_coefficient();
    flag(own);
    real(value);
    real(previous);
    let (own, value, previous) = o.own_deviation_angle();
    flag(own);
    real(value);
    real(previous);
    int(o.type_of_hlr() as i32);
    let d = o.attributes();
    flag(d.has_own_shading_aspect());
    flag(d.has_own_line_aspect());
    flag(d.has_own_wire_aspect());
    let line = d.line_aspect();
    if let Some(line) = line.get() {
        real(f64::from(line.aspect().width()));
        rgb(line.aspect().interior_color());
    }
}

fn select(o: &Handle<AisShape>, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    o.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for item in selection.entities() {
        let entity = item.base_sensitive();
        text(entity.dynamic_type().map_or("", |t| t.name()).as_bytes());
        int(entity.nb_sub_elements());
        let owner = entity.owner_id().cast::<dyn StdSelectBRepOwnerSource>();
        flag(!owner.is_null());
        if let Some(owner) = owner.get() {
            let shape = owner.shape_ref().clone_shape();
            int(if shape.is_null() {
                -1
            } else {
                shape.shape_type()? as i32
            });
            int(owner.priority());
        }
    }
    Ok(())
}

fn operation(
    o: &Handle<AisShape>,
    manager: &Handle<PrsMgrPresentationManager>,
    viewer: &Handle<V3dViewer>,
    op: i32,
    width: f64,
) -> Result<(), Failure> {
    match op {
        0 => o.set_color(Color::named(NamedColor::RED))?,
        1 => o.unset_color()?,
        2 => o.set_width(width)?,
        3 => o.unset_width()?,
        4 => o.set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))?,
        5 => o.unset_material()?,
        6 => o.set_transparency(0.4)?,
        7 => o.unset_transparency()?,
        8..=10 => {
            let mode = op - 8;
            if manager.has_presentation(&o.clone().cast(), mode) {
                manager.update(&o.clone().cast(), mode)?;
            } else {
                manager.presentation(&o.clone().cast(), mode, true, &Handle::default())?;
            }
        }
        11 => select(o, 0)?,
        12 => select(o, 2)?,
        13 => select(o, 4)?,
        14 => o.set_own_deviation_coefficient(0.01),
        15 => flag(o.reset_own_deviation_coefficient()),
        16 => o.set_own_deviation_angle(0.3),
        17 => o.set_angle_and_deviation(0.25),
        18 => o.set_type_of_hlr(Prs3dTypeOfHlr::Algo),
        19 => {
            o.set_texture_repeat_uv(Point2::new(2., 3.));
            o.set_texture_origin_uv(Point2::new(0.5, 0.25));
            o.set_texture_scale_uv(Point2::new(4., 5.));
        }
        20 => o.set_shape(&shape_of(1)),
        21 | 22 => {
            let camera = Handle::new(Graphic3dCamera::default());
            camera.set_eye_and_center(&Point3::new(10., 8., 6.), &Point3::new(0., 0., 0.))?;
            camera.set_up(&Direction3::new(0., 0., 1.)?);
            let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())?;
            let mut t = Transform3::default();
            if op == 22 {
                t.set_translation(Vector3::new(1., 2., 3.));
            }
            let datum = if op == 22 {
                Handle::new(Datum3::new(t))
            } else {
                Handle::default()
            };
            o.compute_hlr(&camera, &datum, &prs.clone().cast())?;
            int(prs.groups().len() as i32);
        }
        23 => flag(o.reset_own_deviation_angle()),
        _ => {
            for kind in [
                ShapeEnum::Compound,
                ShapeEnum::CompSolid,
                ShapeEnum::Solid,
                ShapeEnum::Shell,
                ShapeEnum::Face,
                ShapeEnum::Wire,
                ShapeEnum::Edge,
                ShapeEnum::Vertex,
                ShapeEnum::Shape,
            ] {
                int(ais_shape_selection_mode(kind));
            }
            for mode in -1..=9 {
                int(ais_shape_selection_type(mode) as i32);
            }
        }
    }
    Ok(())
}

/// `v3_ais_shape Shape Link Width Count Op...`.
pub fn ais_shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, link) = (r.integer(), r.integer());
    let width = r.real();
    let count = r.integer();
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let o = AisShape::new(&shape_of(kind));
    if link == 0 {
        o.attributes().setup_own_defaults().unwrap();
    }
    if link == 1 {
        let d = Handle::new(Prs3dDrawer::new());
        d.setup_own_defaults().unwrap();
        d.line_aspect().set_width(2.5).unwrap();
        o.attributes().link_to(&d);
    }
    state(&o);
    for _ in 0..count {
        let op = r.integer();
        attempt(|| operation(&o, &manager, &viewer, op, width));
        state(&o);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
