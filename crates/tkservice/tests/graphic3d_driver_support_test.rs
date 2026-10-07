//! The support classes of the Graphic3d driver model: graduated trihedron, driver factory
//! registry, frame statistics frames, subviews, XR cameras and the view settings.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::osd::{CpuTimes, Timer, WallClock};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkservice::aspect::{
    AspectDisplayConnection, AspectNeutralWindow, AspectTypeOfTriedronPosition, AspectWindow,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCView, Graphic3dCViewComputed, Graphic3dCViewMembers, Graphic3dCViewStructures,
    Graphic3dCViewSubviews, Graphic3dCViewXr, Graphic3dCamera, Graphic3dDiagnosticInfo,
    Graphic3dFrameStats, Graphic3dFrameStatsBase, Graphic3dFrameStatsDict,
    Graphic3dFrameStatsMembers, Graphic3dGraduatedTrihedron, Graphic3dGraphicDriver,
    Graphic3dGraphicDriverFactory, Graphic3dGraphicDriverFactoryBase, Graphic3dStructureManager,
    Graphic3dTypeOfShadingModel, graphic3d_graphic_driver_factories,
    graphic3d_graphic_driver_factory_default, graphic3d_graphic_driver_factory_register,
    graphic3d_graphic_driver_factory_unregister,
};
use rustycad_tkservice::null_driver::{NullGraphicDriver, NullView};
use std::sync::Arc;

#[test]
fn graduated_trihedron_defaults_and_axis_checks() {
    let mut trihedron = Graphic3dGraduatedTrihedron::default();
    assert_eq!(trihedron.names_size(), 12);
    assert_eq!(
        trihedron.axes[1].name.as_utf16(),
        "Y".encode_utf16().collect::<Vec<_>>().as_slice()
    );
    trihedron.change_axis_aspect(2).expect("z").tickmarks_number = 3;
    assert_eq!(trihedron.axis_aspect(2).expect("z").tickmarks_number, 3);
    assert_eq!(
        trihedron.axis_aspect(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

struct FakeDriverFactory {
    transient: StandardTransient,
    base: Graphic3dGraphicDriverFactoryBase,
}
rustycad_tkernel::impl_transient!(FakeDriverFactory, "FakeDriverFactory", StandardTransient, transient,
    interfaces: [dyn Graphic3dGraphicDriverFactory]);
impl Graphic3dGraphicDriverFactory for FakeDriverFactory {
    fn graphic_driver_factory_base(&self) -> &Graphic3dGraphicDriverFactoryBase {
        &self.base
    }
    fn create_driver(
        &self,
        _display: &Handle<AspectDisplayConnection>,
    ) -> Result<Handle<dyn Graphic3dGraphicDriver>, Failure> {
        Ok(NullGraphicDriver::new_handle())
    }
}

fn text_of(text: &AsciiString) -> String {
    String::from_utf8_lossy(text.as_bytes()).into_owned()
}

fn factory(name: &str) -> Handle<dyn Graphic3dGraphicDriverFactory> {
    let base = Graphic3dGraphicDriverFactoryBase::new(&AsciiString::from(name));
    Handle::new(FakeDriverFactory {
        transient: StandardTransient::default(),
        base,
    })
    .cast()
}

#[test]
fn driver_factory_registry() {
    let (first, second) = (factory("fake-a"), factory("FAKE-B"));
    graphic3d_graphic_driver_factory_register(&first, false);
    graphic3d_graphic_driver_factory_register(&factory("FAKE-A"), false);
    graphic3d_graphic_driver_factory_register(&second, true);
    assert!(graphic3d_graphic_driver_factory_default() == second);
    let names: Vec<_> = graphic3d_graphic_driver_factories()
        .iter()
        .map(|f| text_of(&f.name()))
        .collect();
    assert!(names.contains(&"fake-a".to_string()) && !names.contains(&"FAKE-A".to_string()));
    graphic3d_graphic_driver_factory_unregister(&AsciiString::from("Fake-B"));
    graphic3d_graphic_driver_factory_unregister(&AsciiString::from("fake-a"));
    assert!(
        graphic3d_graphic_driver_factories()
            .iter()
            .all(|f| !text_of(&f.name()).starts_with("fake"))
    );
}

/// A wall clock that the test moves.
struct FakeWallClock(SharedState<f64>);
impl WallClock for FakeWallClock {
    fn now(&self) -> f64 {
        *self.0.read()
    }
}
/// A CPU clock at a fixed half second.
struct FakeCpuTimes;
impl CpuTimes for FakeCpuTimes {
    fn process_cpu(&self) -> (f64, f64) {
        (0.5, 0.0)
    }
    fn thread_cpu(&self) -> (f64, f64) {
        (0.5, 0.0)
    }
}

#[derive(Default)]
struct FakeFrameStats {
    transient: StandardTransient,
    base: Graphic3dFrameStatsBase,
}
rustycad_tkernel::impl_transient!(FakeFrameStats, "FakeFrameStats", dyn Graphic3dFrameStats, transient,
    interfaces: [dyn Graphic3dFrameStats]);
impl Graphic3dFrameStats for FakeFrameStats {
    fn frame_stats_base(&self) -> &Graphic3dFrameStatsBase {
        &self.base
    }
    fn update_statistics(&self, _view: &Handle<dyn Graphic3dCView>, _is_immediate_only: bool) {}
}

#[test]
fn frame_start_and_end_measure_the_rates() {
    let clock = Arc::new(FakeWallClock(SharedState::new(0.0)));
    let stats = Handle::new(FakeFrameStats::default());
    stats.base.fields.write().fps_timer =
        Timer::with_cpu(true, Arc::new(FakeCpuTimes), clock.clone());
    stats.frame_start(&Handle::default(), false);
    *clock.0.write() = 0.5;
    stats.frame_end(&Handle::default(), false);
    assert_eq!(stats.frame_duration(), 0.5);
    assert_eq!(stats.frame_rate(), -1.0);
    stats.frame_start(&Handle::default(), false);
    *clock.0.write() = 2.0;
    stats.frame_end(&Handle::default(), false);
    assert_eq!(stats.frame_rate(), 1.0);
    assert_eq!(
        stats.timer_value(rustycad_tkservice::graphic3d::Graphic3dFrameStatsTimer::ElapsedFrame),
        1.0
    );
}

fn view_with_window(
    manager: &Handle<Graphic3dStructureManager>,
    driver: &Handle<dyn Graphic3dGraphicDriver>,
    size: [i32; 2],
) -> Handle<dyn Graphic3dCView> {
    let view = driver.create_view(manager).expect("view");
    let window = AspectNeutralWindow::default();
    window.set_size(size[0], size[1]);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.cast::<NullView>().set_null_window(&window);
    view
}

#[test]
fn subviews_are_placed_in_the_parent_window() {
    let driver = NullGraphicDriver::new_handle();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver).expect("manager"));
    let parent = view_with_window(&manager, &driver, [800, 600]);
    let child = view_with_window(&manager, &driver, [1, 1]);
    parent.add_subview(&child);
    child.cview_base().fields.write().parent_view = parent.downgrade();
    child.set_subview_corner(AspectTypeOfTriedronPosition::RIGHT_LOWER);
    child.set_subview_size(Vec2::new(0.25, 100.0));
    child.set_subview_margins(Vec2::new(2, 2));
    child.resized();
    assert_eq!(child.subview_top_left(), Vec2::new(602, 502));
    assert!(
        child.is_subview() && !child.is_sub_view_relative_size() && child.identification() == 1
    );
    child.remove();
    assert!(
        child.is_removed() && parent.subviews().is_empty() && manager.defined_views().len() == 1
    );
}

#[test]
fn view_settings_xr_and_diagnostics() {
    let driver = NullGraphicDriver::new_handle();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver).expect("manager"));
    let view = view_with_window(&manager, &driver, [640, 480]);
    assert!(
        view.set_shading_model(Graphic3dTypeOfShadingModel::Default)
            .is_err()
    );
    assert!(view.set_unit_factor(0.0).is_err());
    view.set_computed_mode(true);
    assert!(view.computed_mode() && view.is_computed(0).is_null());
    let base = Handle::new(Graphic3dCamera::default());
    view.set_base_xr_camera(&base);
    let world = view.pose_xr_to_world(&Transform3::default()).expect("pose");
    let eye = base.eye();
    assert!(
        Point3::new(0.0, 0.0, 0.0)
            .transformed(world)
            .is_equal(&eye, 1e-12)
    );
    assert!(!view.init_xr() && !view.is_active_xr());
    let mut dict = Graphic3dFrameStatsDict::default();
    view.diagnostic_information(&mut dict, Graphic3dDiagnosticInfo::DEVICE);
    assert_eq!(dict.extent(), 5);
    assert_eq!(view.consider_zoom_persistence_objects().expect("coef"), 1.0);
}
