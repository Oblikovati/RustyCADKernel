// Native Martensite oracle uses actual AIS selection, presentations and camera fitting.
#include <AIS_InteractiveContext.hxx>
#include <AIS_PointCloud.hxx>
#include <AIS_Selection.hxx>
#include <AIS_Trihedron.hxx>
#include <AIS_ViewController.hxx>
#include <Geom_Axis2Placement.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_SensitiveEntity.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iomanip>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
struct FitScene {
  occ::handle<V3d_Viewer> viewer;
  occ::handle<V3d_View> view;
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_PointCloud> selected;
  occ::handle<AIS_PointCloud> outer;
  occ::handle<AIS_Trihedron> trihedron;
};
static occ::handle<AIS_PointCloud> cloud(bool outer) {
  occ::handle<Graphic3d_ArrayOfPoints> points = new Graphic3d_ArrayOfPoints(2);
  points->AddVertex(outer ? gp_Pnt(-20,-30,-40) : gp_Pnt(-2,-3,-4));
  points->AddVertex(outer ? gp_Pnt(40,50,60) : gp_Pnt(1,5,7));
  occ::handle<AIS_PointCloud> object = new AIS_PointCloud(); object->SetPoints(points);
  return object;
}
static void fit_view(FitScene& scene, int index) {
  scene.viewer = new V3d_Viewer(new SvDriver()); scene.view = scene.viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(640,480);
  scene.view->SetWindow(window); scene.view->SetImmediateUpdate(false);
  auto camera = scene.view->Camera();
  camera->SetEyeAndCenter(gp_Pnt(0,0,200), gp_Pnt()); camera->SetScale(200); camera->SetAspect(4.0/3.0);
  camera->SetProjectionType(index / 256 == 0 ? Graphic3d_Camera::Projection_Orthographic : Graphic3d_Camera::Projection_Perspective);
  scene.context = new AIS_InteractiveContext(scene.viewer);
}
static void degenerate_cloud(FitScene& scene, int index) {
  if (index % 8 != 7) return;
  occ::handle<Graphic3d_ArrayOfPoints> points = new Graphic3d_ArrayOfPoints(1);
  points->AddVertex(gp_Pnt(1,2,3)); scene.selected->SetPoints(points);
}
static void fit_objects(FitScene& s, int index) {
  s.selected = cloud(false); s.outer = cloud(true); degenerate_cloud(s, index);
  s.trihedron = new AIS_Trihedron(new Geom_Axis2Placement(gp_Ax2(gp_Pnt(5,6,7), gp_Dir(0,0,1))));
  s.trihedron->SetSize(20);
  const int flags = (index / 16) % 16;
  s.selected->SetInfiniteState((flags & 4) != 0); s.trihedron->SetInfiniteState((flags & 8) != 0);
  s.context->Display(s.selected, false); s.context->Display(s.outer, false);
  s.context->Display(s.trihedron, 0, 2, false);
  s.context->SetViewAffinity(s.selected, s.view, (flags & 1) == 0);
  s.context->SetViewAffinity(s.trihedron, s.view, (flags & 2) == 0);
}
static void fit_transform(FitScene& s, int index) {
  if ((index / 8) % 2 == 0) return;
  gp_Trsf transform; transform.SetTranslation(gp_Vec(10,-4,3));
  s.context->SetLocation(s.selected, TopLoc_Location(transform));
  s.context->SetLocation(s.trihedron, TopLoc_Location(transform));
}
static occ::handle<SelectMgr_EntityOwner> axis_owner(const FitScene& s, int rank) {
  return s.trihedron->Selection(2)->Entities().Value(rank)->BaseSensitive()->OwnerId();
}
static void select_fit_owners(FitScene& s, int index) {
  const int selection = index % 8;
  if (selection == 1 || selection == 2 || selection == 5 || selection == 7)
    s.context->Selection()->AddSelect(s.selected->GlobalSelOwner());
  if (selection == 2) s.context->Selection()->AddSelect(s.outer->GlobalSelOwner());
  if (selection == 3 || selection == 4 || selection == 5 || selection == 6)
    s.context->Selection()->AddSelect(axis_owner(s, 1));
  if (selection == 4) s.context->Selection()->AddSelect(axis_owner(s, 2));
  if (selection == 6) s.context->Selection()->AddSelect(axis_owner(s, 1));
}
static void emit_bounds(const Bnd_Box& bounds) {
  std::cout << ' ' << bounds.IsVoid();
  if (bounds.IsVoid()) { std::cout << " 0 0 0 0 0 0"; return; }
  double x0,y0,z0,x1,y1,z1; bounds.Get(x0,y0,z0,x1,y1,z1);
  std::cout << ' ' << x0 << ' ' << y0 << ' ' << z0 << ' ' << x1 << ' ' << y1 << ' ' << z1;
}
static void emit_point(const gp_Pnt& point) {
  std::cout << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z();
}
static void emit_direction(const gp_Dir& direction) {
  std::cout << ' ' << direction.X() << ' ' << direction.Y() << ' ' << direction.Z();
}
static void emit_camera(const occ::handle<Graphic3d_Camera>& camera) {
  emit_point(camera->Center()); emit_point(camera->Eye());
  emit_direction(camera->Up()); emit_direction(camera->Direction());
  std::cout << ' ' << camera->Scale() << ' ' << camera->Distance() << ' ' << camera->Aspect()
            << ' ' << int(camera->ProjectionType()) << ' ' << camera->ZNear() << ' ' << camera->ZFar() << ' ' << camera->FOVy();
}
static void snapshot(int index, int step, const FitScene& s) {
  std::cout << index << ' ' << step << ' ' << s.context->NbSelected();
  emit_bounds(s.context->BoundingBoxOfSelection(s.view));
  emit_bounds(s.context->BoundingBoxOfSelection(occ::handle<V3d_View>()));
  emit_bounds(s.view->View()->MinMaxValues()); emit_camera(s.view->Camera());
  std::cout << '\n';
}
static void run_scene(int index) {
  FitScene scene; fit_view(scene, index); fit_objects(scene, index); fit_transform(scene, index); select_fit_owners(scene, index);
  AIS_ViewController controller; snapshot(index, 0, scene);
  for (int step = 1; step <= 3; ++step) { controller.FitAllAuto(scene.context, scene.view); snapshot(index, step, scene); }
  scene.context->FitSelected(scene.view); snapshot(index, 4, scene);
  scene.context->FitSelected(scene.view, 0.05, false); snapshot(index, 5, scene);
}
int main() {
  std::cout << std::setprecision(17);
  for (int index = 0; index < 512; ++index) run_scene(index);
}
