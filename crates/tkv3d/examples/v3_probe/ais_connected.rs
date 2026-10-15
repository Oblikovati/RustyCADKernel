//! AIS_ConnectedInteractive and AIS_MultipleConnectedInteractive in a context, the twin of
//! tools/oracle/tkv3d_probe_ais_connected.cxx.
use super::common::{Reader, attempt, end, flag, int, text, xyz};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkv3d::std_select::{StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use rustycad_tkv3d::v3d::V3dViewer;

fn selection(o: &Handle<dyn SelectMgrSelectableObject>, mode: i32) -> Result<(), Failure> {
    let sel = Handle::new(SelectMgrSelection::new(mode));
    o.compute_selection(&sel, mode)?;
    int(sel.entities().len() as i32);
    // Sorted rows: the decomposed owner order is memory dependent natively.
    let mut rows = Vec::new();
    for item in sel.entities() {
        let e = item.base_sensitive();
        let owner = e.owner_id();
        let brep = owner.cast::<dyn StdSelectBRepOwnerSource>();
        let kind = match brep.get() {
            Some(b) => b.shape_ref().clone_shape().shape_type()? as i32,
            None => -1,
        };
        rows.push(format!(
            "{} {} {} {} {} {}",
            e.dynamic_type().map_or("", |t| t.name()),
            i32::from(owner.selectable().object_address() == o.object_address()),
            i32::from(owner.has_location()),
            i32::from(owner.comes_from_decomposition()),
            owner.priority(),
            kind
        ));
    }
    rows.sort();
    for row in rows {
        text(row.as_bytes());
    }
    Ok(())
}

fn object(o: &dyn AisInteractiveObject) {
    int(o.kind_of_interactive() as i32);
    int(o.signature());
    flag(o.accept_shape_decomposition());
    for mode in 0..4 {
        flag(o.accept_display_mode(mode));
    }
    int(o.type_of_presentation3d() as i32);
    flag(o.has_own_presentations());
    int(o.children().len() as i32);
    flag(o.has_transformation());
    if o.has_transformation() {
        xyz(o.transformation().translation_part());
    }
    int(o.presentations().len() as i32);
}

/// `v3_ais_connected Case Arg`.
pub fn connected_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, arg) = (r.integer(), r.integer());
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let ax = Axis3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let face: Handle<dyn AisInteractiveObject> = AisShape::new(&Shape::from(
        BRepBuilderAPIMakeFace::from_cylinder_bounds(
            Cylinder3::new(ax, 2.).unwrap(),
            (0., std::f64::consts::PI, 0., 3.),
        )
        .unwrap()
        .face()
        .unwrap(),
    ))
    .into_cast();
    let edge: Handle<dyn AisInteractiveObject> = AisShape::new(&Shape::from(
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(2., 1., 0.))
            .unwrap()
            .edge()
            .unwrap(),
    ))
    .into_cast();
    let point: Handle<dyn AisInteractiveObject> =
        AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast())
            .unwrap()
            .into_cast();
    let mut t = Transform3::default();
    t.set_translation(Vector3::new(f64::from(arg), 2. * f64::from(arg), 0.5));
    match kind {
        0 => {
            let c = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            object(&*c);
            let target = if arg % 2 == 0 { &face } else { &edge };
            attempt(|| c.connect_trsf(target, &t));
            object(&*c);
            flag(c.has_connection());
            let selectable: Handle<dyn SelectMgrSelectableObject> = c.clone().into_cast();
            for mode in [0, 1, 2, 4] {
                attempt(|| selection(&selectable, mode));
            }
            let dynamic: Handle<dyn AisInteractiveObject> = c.clone().into_cast();
            ctx.display_with_modes(&dynamic, 0, 0, false, PrsMgrDisplayStatus::None)
                .unwrap();
            int(ctx.display_status(&dynamic) as i32);
            object(&*c);
            attempt(|| {
                let camera = Handle::new(Graphic3dCamera::default());
                camera.set_eye_and_center(&Point3::new(10., 8., 6.), &Point3::new(0., 0., 0.))?;
                let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())?;
                c.compute_hlr(&camera, &Handle::default(), &prs.clone().cast())?;
                int(prs.groups().len() as i32);
                Ok(())
            });
            attempt(|| ctx.disconnect(&dynamic, &Handle::default()));
            object(&*c);
        }
        1 => {
            ctx.display(&face, false).unwrap();
            let c = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            attempt(|| c.connect(&face));
            flag(c.has_connection());
            let c2 = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            let m: Handle<dyn AisInteractiveObject> =
                AisMultipleConnectedInteractive::new().into_cast();
            attempt(|| c2.connect(&m));
            flag(c2.has_connection());
            let c3 = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            let c4 = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            c3.connect_trsf(&edge, &t).unwrap();
            attempt(|| c4.connect(&c3.clone().into_cast()));
            flag(c4.connected_to().object_address() == edge.object_address());
            object(&*c4);
        }
        _ => {
            let m = AisMultipleConnectedInteractive::new();
            object(&*m);
            flag(m.has_connection());
            let a = m.connect_trsf(&face, &t).unwrap();
            let b = m.connect(&point).unwrap();
            object(&*m);
            object(a.get().unwrap());
            object(b.get().unwrap());
            flag(m.has_connection());
            flag(m.get_assembly_owner().object_address() == m.global_sel_owner().object_address());
            let m2 = AisMultipleConnectedInteractive::new();
            let c = m2.connect(&m.clone().into_cast()).unwrap();
            object(&*m2);
            object(c.get().unwrap());
            flag(
                c.cast::<AisMultipleConnectedInteractive>()
                    .get_assembly_owner()
                    .object_address()
                    == m2.get_assembly_owner().object_address(),
            );
            let selectable: Handle<dyn SelectMgrSelectableObject> = m.clone().into_cast();
            for mode in [0, arg % 3 + 1] {
                attempt(|| selection(&selectable, mode));
            }
            let dynamic: Handle<dyn AisInteractiveObject> = m.clone().into_cast();
            ctx.display_with_modes(&dynamic, 0, arg % 2, false, PrsMgrDisplayStatus::None)
                .unwrap();
            int(ctx.display_status(&dynamic) as i32);
            object(&*m);
            flag(a.has_interactive_context());
            let mut modes = List::default();
            ctx.activated_modes(&dynamic, &mut modes);
            int(modes.extent());
            attempt(|| ctx.disconnect(&dynamic, &a));
            object(&*m);
            attempt(|| m.disconnect_all());
            object(&*m);
        }
    }
    int(trace.read().events.len() as i32);
    end();
}
