//! AIS_InteractiveContext attribute members, the twin of
//! tools/oracle/tkv3d_probe_ais_context_attributes.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3};
use rustycad_tkservice::aspect::{AspectTypeOfFacingModel, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::*;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkv3d::prs3d::{Prs3dDrawer, Prs3dLineAspect};
use rustycad_tkv3d::select_mgr::{SelectMgrFilter, SelectMgrFilterType};
use rustycad_tkv3d::v3d::V3dViewer;

const PRIORITIES: [Graphic3dDisplayPriority; 12] = [
    Graphic3dDisplayPriority::Invalid,
    Graphic3dDisplayPriority::Bottom,
    Graphic3dDisplayPriority::AlmostBottom,
    Graphic3dDisplayPriority::Below2,
    Graphic3dDisplayPriority::Below1,
    Graphic3dDisplayPriority::Below,
    Graphic3dDisplayPriority::Normal,
    Graphic3dDisplayPriority::Above,
    Graphic3dDisplayPriority::Above1,
    Graphic3dDisplayPriority::Above2,
    Graphic3dDisplayPriority::Highlight,
    Graphic3dDisplayPriority::Topmost,
];
const KINDS: [AisKindOfInteractive; 7] = [
    AisKindOfInteractive::None,
    AisKindOfInteractive::Datum,
    AisKindOfInteractive::Shape,
    AisKindOfInteractive::Object,
    AisKindOfInteractive::Relation,
    AisKindOfInteractive::Dimension,
    AisKindOfInteractive::LightSource,
];
const FACING: [AspectTypeOfFacingModel; 3] = [
    AspectTypeOfFacingModel::BothSide,
    AspectTypeOfFacingModel::BackSide,
    AspectTypeOfFacingModel::FrontSide,
];

fn state(ctx: &AisInteractiveContext, objects: &[Handle<dyn AisInteractiveObject>]) {
    for o in objects {
        int(ctx.display_status(o) as i32);
        flag(o.has_display_mode());
        int(o.display_mode());
        flag(ctx.has_color(o).unwrap());
        let c = ctx.color(o).unwrap();
        real(c.red());
        real(c.green());
        real(c.blue());
        real(ctx.width(o).unwrap());
        real(o.transparency());
        flag(ctx.has_polygon_offsets(o));
        let (m, f, u) = ctx.polygon_offsets(o).unwrap_or((0, 0., 0.));
        int(m);
        real(f64::from(f));
        real(f64::from(u));
        int(ctx.get_z_layer(o));
        int(ctx.display_priority(o) as i32);
        let mut modes = List::default();
        ctx.activated_modes(o, &mut modes);
        int(modes.extent());
        for mode in modes.iter() {
            int(*mode);
        }
        flag(ctx.is_selected_object(o));
        flag(o.has_material());
        int(o.current_facing_model() as i32);
        let mut owners = AisEntityOwners::default();
        ctx.entity_owners(&mut owners, o, -1);
        int(owners.extent() as i32);
        int(ctx.status(o).length());
    }
    int(ctx.filters().len() as i32);
    int(ctx.filter_type() as i32);
    int(ctx.pixel_tolerance());
    real(ctx.trihedron_size());
    let (equal, x, y) = ctx.plane_size();
    flag(equal);
    real(x);
    real(y);
    flag(ctx.draw_hidden_line());
    int(ctx.display_mode());
    flag(ctx.has_selected_shape());
    int(ctx.nb_selected());
}

fn operation(
    ctx: &AisInteractiveContext,
    o: &Handle<dyn AisInteractiveObject>,
    op: i32,
    arg: i32,
) -> Result<(), Failure> {
    match op {
        0 => ctx.set_color(o, Color::named(NamedColor::RED), true)?,
        1 => ctx.unset_color(o, true)?,
        2 => ctx.set_width(o, 3., true)?,
        3 => ctx.unset_width(o, true)?,
        4 => ctx.set_material(
            o,
            &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold),
            true,
        )?,
        5 => ctx.unset_material(o, true)?,
        6 => ctx.set_transparency(o, f64::from(arg) / 10., true)?,
        7 => ctx.unset_transparency(o, true)?,
        8 => ctx.set_polygon_offsets(o, 1, 1.5, 2., true)?,
        9 => ctx.set_display_mode(o, arg, true)?,
        10 => ctx.unset_display_mode(o, true)?,
        11 => ctx.set_default_display_mode(arg, true)?,
        12 => ctx.set_display_priority(o, PRIORITIES[(arg + 1).clamp(0, 11) as usize])?,
        13 => ctx.set_z_layer(o, arg),
        14 => ctx.activate(o, arg, false)?,
        15 => ctx.deactivate(o)?,
        16 => ctx.deactivate_mode(o, arg)?,
        17 => ctx.activate_all(arg, false)?,
        18 => ctx.deactivate_all()?,
        19 => ctx.deactivate_all_mode(arg)?,
        20 => ctx.set_selection_mode_active(
            o,
            arg,
            true,
            AisSelectionModesConcurrency::Single,
            false,
        )?,
        21 => ctx.set_selection_mode_active(
            o,
            arg,
            true,
            AisSelectionModesConcurrency::Multiple,
            false,
        )?,
        22 => {
            let filter: Handle<dyn SelectMgrFilter> =
                Handle::new(AisTypeFilter::new(KINDS[arg.clamp(0, 6) as usize])).into_cast();
            ctx.add_filter(&filter);
        }
        23 => ctx.remove_filters(),
        24 => ctx.set_filter_type(if arg == 0 {
            SelectMgrFilterType::And
        } else {
            SelectMgrFilterType::Or
        }),
        25 => ctx.set_pixel_tolerance(arg),
        26 => ctx.set_trihedron_size(f64::from(arg), true)?,
        27 => ctx.set_plane_size_xy(f64::from(arg), f64::from(arg) + 1., true)?,
        28 => ctx.set_plane_size(f64::from(arg), true)?,
        29 => ctx.enable_draw_hidden_line(),
        30 => ctx.disable_draw_hidden_line(),
        31 => ctx.sub_intensity_on(o, true)?,
        32 => ctx.sub_intensity_off(o, true)?,
        33 => ctx.set_selected_object(o, true)?,
        34 => ctx.init_attributes()?,
        35 => {
            let d = Handle::new(Prs3dDrawer::new());
            d.setup_own_defaults()?;
            d.set_color(Color::named(NamedColor::BLUE1));
            ctx.set_local_attributes(o, &d, true)?;
        }
        36 => ctx.unset_local_attributes(o, true)?,
        37 => ctx.set_angle_and_deviation(o, 0.3, true)?,
        38 => ctx.set_selection_sensitivity(o, 0, arg)?,
        39 => ctx.set_current_facing_model(o, FACING[arg.clamp(0, 2) as usize]),
        40 => ctx.erase(o, true)?,
        41 => ctx.display(o, true)?,
        42 => ctx.rebuild_selection_structs()?,
        43 => ctx.set_sub_intensity_color(Color::named(NamedColor::GREEN)),
        44 => {
            let pers = Graphic3dTransformPers::with_point(
                Graphic3dTransModeFlags::ZOOM_PERS,
                &Point3::new(1., 2., 3.),
            )?;
            ctx.set_transform_persistence(o, &Handle::new(pers).into_cast())?;
        }
        45 => {
            let line =
                Prs3dLineAspect::new(Color::named(NamedColor::ORANGE), AspectTypeOfLine::Dot, 2.)?;
            ctx.set_hidden_line_aspect(&Handle::new(line));
        }
        46 => ctx.clear_selected(true)?,
        47 => ctx.redisplay_kind(KINDS[arg.clamp(0, 6) as usize], 0, true)?,
        _ => {}
    }
    Ok(())
}

/// `v3_ais_ctx_attr Count Op Object Arg...`.
pub fn context_attributes_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let count = r.integer();
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let ax = Axis3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        Cylinder3::new(ax, 2.).unwrap(),
        (0., std::f64::consts::PI, 0., 3.),
    )
    .unwrap()
    .face()
    .unwrap();
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 1., 0.))
            .unwrap()
            .edge()
            .unwrap();
    let objects: Vec<Handle<dyn AisInteractiveObject>> = vec![
        AisShape::new(&Shape::from(face)).into_cast(),
        AisShape::new(&Shape::from(edge)).into_cast(),
        AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast())
            .unwrap()
            .into_cast(),
    ];
    for o in &objects {
        ctx.display(o, false).unwrap();
    }
    state(&ctx, &objects);
    for _ in 0..count {
        let (op, obj, arg) = (r.integer(), r.integer(), r.integer());
        attempt(|| operation(&ctx, &objects[(obj % 3) as usize], op, arg));
        state(&ctx, &objects);
    }
    int(trace.read().events.len() as i32);
    end();
}
