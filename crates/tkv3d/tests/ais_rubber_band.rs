//! Martensite live screen-point, appearance and cached rubber-band geometry contracts.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    ncollection::Vec2,
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn presentation() -> (
    Handle<PrsMgrPresentationManager>,
    Handle<dyn Graphic3dStructureSource>,
) {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    (manager, prs.cast())
}
#[test]
fn constructors_preserve_overlay_and_material_defaults() {
    let white = Color::named(NamedColor::WHITE);
    let red = Color::named(NamedColor::RED);
    let band = AisRubberBand::new().unwrap();
    assert_eq!(band.line_color(), white);
    assert_eq!(band.fill_color(), white);
    assert_eq!(band.line_width(), 1.);
    assert_eq!(band.line_type(), AspectTypeOfLine::Solid);
    assert_eq!(band.fill_transparency(), 1.);
    assert!(!band.is_filling());
    assert!(band.is_polygon_closed());
    assert_eq!(band.signature(), -1);
    assert_eq!(band.kind_of_interactive(), AisKindOfInteractive::None);
    assert_eq!(band.z_layer(), GRAPHIC3D_ZLAYER_ID_TOP_OSD);
    let persistence = band.transform_persistence();
    assert_eq!(persistence.mode(), Graphic3dTransModeFlags::TMF_2D);
    assert_eq!(
        persistence.corner_2d().unwrap(),
        AspectTypeOfTriedronPosition::LEFT_LOWER
    );
    let fill = band.attributes().shading_aspect().aspect();
    assert_eq!(fill.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
    assert_eq!(fill.alpha_mode(), Graphic3dAlphaMode::Blend);
    assert_eq!(
        fill.front_material().name(),
        Graphic3dNameOfMaterial::Plastified
    );
    let filled =
        AisRubberBand::with_filling(red, AspectTypeOfLine::Dash, white, 0.25, 2., false).unwrap();
    assert!(filled.is_filling());
    assert_eq!(filled.line_color(), red);
    assert_eq!(filled.line_type(), AspectTypeOfLine::Dash);
    assert_eq!(filled.line_width(), 2.);
    assert_eq!(filled.fill_transparency(), 0.25);
    assert!(!filled.is_polygon_closed());
    assert!(AisRubberBand::with_line(white, AspectTypeOfLine::Solid, 0., true).is_err());
    assert!(
        AisRubberBand::with_filling(white, AspectTypeOfLine::Solid, red, 2., 1., true).is_err()
    );
}
#[test]
fn point_sequence_retains_identity_bounds_order_and_empty_failure() {
    let band = AisRubberBand::new().unwrap();
    let borrowed_points = band.points();
    assert!(band.remove_last_point().is_err());
    band.set_rectangle(10, 20, -1, -2);
    let expected = [
        Vec2::new(10, 20),
        Vec2::new(10, -2),
        Vec2::new(-1, -2),
        Vec2::new(-1, 20),
    ];
    assert_eq!(
        borrowed_points.read().iter().copied().collect::<Vec<_>>(),
        expected
    );
    band.add_point(Vec2::new(4, 5));
    assert_eq!(borrowed_points.read().len(), 5);
    band.remove_last_point().unwrap();
    band.clear_points();
    assert!(borrowed_points.read().is_empty());
    assert!(std::ptr::eq(borrowed_points, band.points()));
}
#[test]
fn appearance_keeps_native_exception_order_and_ownership() {
    let band = AisRubberBand::new().unwrap();
    let red = Color::named(NamedColor::RED);
    band.set_line_color(red);
    band.set_line_type(AspectTypeOfLine::Dot);
    band.set_line_width(3.).unwrap();
    assert_eq!(band.line_color(), red);
    assert_eq!(band.line_type(), AspectTypeOfLine::Dot);
    assert_eq!(band.line_width(), 3.);
    assert!(!band.has_color());
    assert!(band.set_line_width(-1.).is_err());
    assert_eq!(band.line_width(), 3.);
    assert!(band.set_filling_color(red, -1.).is_err());
    assert!(band.is_filling());
    assert_eq!(band.fill_color(), Color::named(NamedColor::WHITE));
    band.set_filling_color(red, 0.5).unwrap();
    assert_eq!(band.fill_color(), red);
    assert_eq!(band.fill_transparency(), 0.5);
    band.set_filling(false);
    assert!(!band.is_filling());
    band.attributes()
        .shading_aspect()
        .aspect()
        .set_interior_style(AspectInteriorStyle::Hatch);
    assert!(band.is_filling());
    band.set_polygon_closed(false);
    assert!(!band.is_polygon_closed());
}
#[test]
fn rectangle_fill_and_border_reuse_native_handles_and_normals() {
    let band = AisRubberBand::new().unwrap();
    band.set_rectangle(1, 2, 10, 20);
    band.set_filling(true);
    let (manager, prs) = presentation();
    band.compute(&manager, &prs, 0).unwrap();
    let fields = &band.rubber_band_base().fields;
    let triangles = fields.triangles.read().clone();
    let borders = fields.borders.read().clone();
    assert_eq!(triangles.vertex_number(), 6);
    assert_eq!(borders.vertex_number(), 5);
    assert_eq!(borders.vertice_xyz(1).unwrap(), [1., 2., 0.]);
    assert_eq!(borders.vertice_xyz(5).unwrap(), [1., 2., 0.]);
    for index in 1..=6 {
        assert_eq!(triangles.vertex_normal_xyz(index).unwrap(), [0., 0., 1.]);
    }
    band.set_rectangle(3, 4, 30, 40);
    band.compute(&manager, &prs, 0).unwrap();
    assert_eq!(triangles, *fields.triangles.read());
    assert_eq!(borders, *fields.borders.read());
    assert_eq!(borders.vertice_xyz(5).unwrap(), [3., 4., 0.]);
    assert_eq!(prs.groups().len(), 4);
    band.set_polygon_closed(false);
    band.compute(&manager, &prs, 0).unwrap();
    assert_ne!(borders, *fields.borders.read());
    assert_eq!(fields.borders.read().vertex_number(), 4);
    band.clear_points();
    assert!(!band.fill_triangles().unwrap());
    assert_eq!(triangles, *fields.triangles.read());
}
#[test]
fn unsupported_modes_and_selection_preserve_existing_content() {
    let band = AisRubberBand::new().unwrap();
    let (manager, prs) = presentation();
    for mode in [-1, 0, 1, 2] {
        assert_eq!(band.accept_display_mode(mode), mode == 0);
    }
    band.compute(&manager, &prs, 1).unwrap();
    assert!(prs.groups().is_empty());
    let selected = Handle::new(SelectMgrSelection::new(0));
    let owner = SelectMgrEntityOwner::new(0);
    selected
        .add(
            &Handle::new(Select3DSensitivePoint::new(
                &owner.cast(),
                rustycad_tkmath::gp::Point3::default(),
            ))
            .cast(),
        )
        .unwrap();
    band.compute_selection(&selected, 0).unwrap();
    assert_eq!(selected.entities().len(), 1);
    assert!(band.compute(&manager, &prs, 0).is_err());
    assert_eq!(
        band.rubber_band_base()
            .fields
            .borders
            .read()
            .vertex_number(),
        0
    );
}
#[test]
fn destruction_releases_cached_arrays_without_mutating_retained_handles() {
    let band = AisRubberBand::new().unwrap();
    band.set_rectangle(8, 8, 36, 36);
    band.set_filling(true);
    let (manager, prs) = presentation();
    band.compute(&manager, &prs, 0).unwrap();
    let triangles = band.rubber_band_base().fields.triangles.read().clone();
    let borders = band.rubber_band_base().fields.borders.read().clone();
    let triangle_refs = triangles.ref_count();
    let border_refs = borders.ref_count();
    drop(band);
    assert_eq!(triangles.ref_count(), triangle_refs - 1);
    assert_eq!(borders.ref_count(), border_refs - 1);
    assert_eq!(triangles.vertex_number(), 6);
    assert_eq!(borders.vertex_number(), 5);
}
