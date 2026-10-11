// OCCT is a local oracle only; Martensite replays the resulting asset in Rust.
#include <AIS_InteractiveContext.hxx>
#include <AIS_Point.hxx>
#include <AIS_ViewController.hxx>
#include <Geom_CartesianPoint.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iomanip>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"

class OraclePickingController : public AIS_ViewController {
public:
  void seed_previous() { myPrevMoveTo.SetValues(14, 18); }
};

static occ::handle<V3d_View> picking_view(const occ::handle<V3d_Viewer>& viewer, int projection) {
  auto view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow();
  window->SetSize(80, 60);
  view->SetWindow(window);
  auto camera = view->Camera();
  camera->SetEyeAndCenter(gp_Pnt(0., 0., 20.), gp_Pnt());
  camera->SetScale(20.);
  camera->SetAspect(4. / 3.);
  camera->SetProjectionType(projection == 0 ? Graphic3d_Camera::Projection_Orthographic
                                          : Graphic3d_Camera::Projection_Perspective);
  return view;
}
static void print_pick(int projection, int operation, int cursor, int stick, bool hit,
                       const gp_Pnt& point, const AIS_ViewController& controller) {
  std::cout << projection << ' ' << operation << ' ' << cursor << ' ' << stick << ' ' << hit
            << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z()
            << ' ' << controller.HasPreviousMoveTo() << '\n';
}
static void gravity_cases(int projection, const occ::handle<AIS_InteractiveContext>& context,
                          const occ::handle<V3d_View>& view) {
  OraclePickingController controller;
  for (int mode = 0; mode < 5; ++mode) {
    for (int scenario = 0; scenario < 4; ++scenario) {
      controller.seed_previous();
      controller.SetRotationMode(AIS_RotationMode(mode));
      controller.SetStickToRayOnRotation(scenario % 2 != 0);
      controller.ChangeInputBuffer(AIS_ViewInputBufferType_GL).OrbitRotation.PointStart.SetValues(
        scenario / 2 == 0 ? 40.9 : 80.9, 30.9);
      gp_Pnt point = controller.GravityPoint(context, view);
      print_pick(projection, 2, mode, scenario, true, point, controller);
    }
  }
}
static void cursor_cases(int projection, const occ::handle<AIS_InteractiveContext>& context,
                         const occ::handle<V3d_View>& view) {
  OraclePickingController controller;
  for (int operation = 0; operation < 2; ++operation) {
    for (int cursor = -2; cursor <= 2; ++cursor) {
      for (int stick = 0; stick < 2; ++stick) {
        gp_Pnt point(97., 98., 99.);
        controller.seed_previous();
        bool hit = operation == 0
          ? controller.PickPoint(point, context, view, NCollection_Vec2<int>(40 + cursor * 5, 30), stick != 0)
          : controller.PickAxis(point, context, view, gp_Ax1(gp_Pnt(cursor * 0.2, 0.3, 20.), gp_Dir(0., 0., -1.)));
        print_pick(projection, operation, cursor, stick, hit, point, controller);
      }
    }
  }
}
static void picking_cases(int projection) {
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  auto view = picking_view(viewer, projection);
  occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  context->Display(new AIS_Point(new Geom_CartesianPoint(0.2, 0.3, 2.)), false);
  cursor_cases(projection, context, view);
  gravity_cases(projection, context, view);
}
int main() {
  std::cout << std::setprecision(17);
  picking_cases(0);
  picking_cases(1);
}
