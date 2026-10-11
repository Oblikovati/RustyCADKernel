// Local Martensite camera primitives use actual V3d cameras and retained CPU scenes.
#include <AIS_AnimationCamera.hxx>
#include <Aspect_OpenVRSession.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include "tkservice_probe_null_driver.cxx"
// Reuse the already pinned complete UI/GL seed and snapshot schema without duplicating it.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"
#define main martensite_unused_transfer_oracle
#include "martensite_view_controller_transfer.cxx"
#undef main
#pragma GCC diagnostic pop
class PrimitiveRecordingView : public SvView {
public:
  bool invalidated = false;
  PrimitiveRecordingView(const occ::handle<Graphic3d_StructureManager>& m) : SvView(m, {}) {}
  void Invalidate() override { invalidated = true; }
  bool IsInvalidated() override { return invalidated; }
  void Redraw() override { invalidated = false; }
};
class PrimitiveRecordingDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& m) override {
    return new PrimitiveRecordingView(m);
  }
};
class PrimitiveHeadSession : public Aspect_OpenVRSession {
public:
  PrimitiveHeadSession() { myHeadPose.SetTranslation(gp_Vec(1.25,-2.5,0.75)); }
};
static double scroll_delta(int index) {
  const double values[] = {-100,0,0.25,40,500}; return values[(index / 5) % 5];
}
static Aspect_ScrollDelta primitive_scroll(int index) {
  const NCollection_Vec2<int> pixel = (index / 25) % 2 == 0 ? NCollection_Vec2<int>(-1,-1) : NCollection_Vec2<int>(137,91);
  return Aspect_ScrollDelta(pixel, scroll_delta(index), Aspect_VKeyFlags(256));
}
class PrimitiveProbe : public AIS_ViewController {
public:
  void seed(int index) {
    seed_buffer(myUI,index,0); seed_buffer(myGL,index,1);
    myGL.Panning.ToPan = (index / 25) % 2 == 0; myGL.Panning.Delta.SetValues(17,-9);
    myGL.ZRotate.ToRotate = (index / 50) % 2 == 0; myGL.ZRotate.Angle = ((index / 5) % 5 - 2) * 0.07;
    if ((index / 50) % 2 != 0) setPanningAnchorPoint(gp_Pnt(7,-3,20));
    const bool allow = (index / 100) % 2 == 0;
    SetAllowPanning(allow); SetAllowRotation(allow); SetAllowZooming(allow); SetAllowZFocus(allow);
    SetMinZoomDistance((index / 200) % 2 == 0 ? 1.0 : 250.0);
  }
  void emit_requests() const { emit_buffer(myUI); emit_buffer(myGL); }
};
struct PrimitiveScene {
  occ::handle<V3d_Viewer> viewer;
  occ::handle<V3d_View> view;
};
static PrimitiveScene primitive_scene(int index) {
  PrimitiveScene s; s.viewer = new V3d_Viewer(new PrimitiveRecordingDriver()); s.view = s.viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(640,480);
  s.view->SetWindow(window); s.view->SetImmediateUpdate(false);
  auto c = s.view->Camera(); c->SetEyeAndCenter(gp_Pnt(0,0,200),gp_Pnt()); c->SetScale(200); c->SetAspect(4.0/3.0);
  c->SetProjectionType(Graphic3d_Camera::Projection(index % 5));
  c->SetZFocus((index / 400) % 3 == 1 ? Graphic3d_Camera::FocusType_Absolute : Graphic3d_Camera::FocusType_Relative, (index / 400) % 3 == 0 ? .25 : ((index / 400) % 3 == 1 ? 1 : 1.95));
  return s;
}
static void setup_primitive_xr(const PrimitiveScene& s, int index) {
  if ((index / 1200) % 2 == 0) return;
  auto view = s.view->View();
  view->SetXRSession(new PrimitiveHeadSession());
  view->SetBaseXRCamera(new Graphic3d_Camera(s.view->Camera()));
  view->SetPosedXRCamera(new Graphic3d_Camera(s.view->Camera()));
  view->SetupXRPosedCamera();
}
static void emit_camera(const occ::handle<Graphic3d_Camera>& c) {
  for (const gp_Pnt p : {c->Center(), c->Eye()}) std::cout << ' ' << p.X() << ' ' << p.Y() << ' ' << p.Z();
  for (const gp_Dir d : {c->Up(), c->Direction()}) std::cout << ' ' << d.X() << ' ' << d.Y() << ' ' << d.Z();
  std::cout << ' ' << c->Scale() << ' ' << c->Distance() << ' ' << c->ZFocus() << ' ' << int(c->ZFocusType()) << ' ' << c->Aspect() << ' ' << int(c->ProjectionType()) << ' ' << c->ZNear() << ' ' << c->ZFar() << ' ' << c->FOVy();
}
static void snapshot(int index, int frame, const PrimitiveProbe& c, const PrimitiveScene& s) {
  std::cout << index << ' ' << frame << ' ' << s.view->View()->IsInvalidated()
    << ' ' << c.ViewAnimation()->IsStopped() << ' ' << c.ViewAnimation()->View().IsNull()
    << ' ' << c.hasPanningAnchorPoint();
  const auto& anchor = c.panningAnchorPoint(); std::cout << ' ' << anchor.X() << ' ' << anchor.Y() << ' ' << anchor.Z();
  emit_camera(s.view->Camera());
  std::cout << ' ' << !s.view->View()->BaseXRCamera().IsNull();
  if (s.view->View()->BaseXRCamera().IsNull()) { for (int i=0;i<21;++i) std::cout << " 0"; }
  else emit_camera(s.view->View()->BaseXRCamera());
  c.emit_requests(); std::cout << '\n';
}
static void apply_primitive(int index, PrimitiveProbe& c, const PrimitiveScene& s) {
  const auto scroll = primitive_scroll(index); const gp_Pnt hit(7,-3,20);
  switch ((index / 2400) % 4) {
    case 0: c.handlePanning(s.view); break;
    case 1: c.handleZRotate(s.view); break;
    case 2: c.handleZoom(s.view,scroll,(index / 50) % 2 != 0 ? &hit : nullptr); break;
    case 3: c.handleZFocusScroll(s.view,scroll); break;
  }
}
static void run_primitive(int index) {
  auto scene = primitive_scene(index); setup_primitive_xr(scene,index);
  PrimitiveProbe controller; controller.seed(index);
  controller.ViewAnimation()->SetView(scene.view); controller.ViewAnimation()->Start(false);
  scene.view->View()->Redraw(); snapshot(index,0,controller,scene);
  for (int frame=1;frame<=2;++frame) { scene.view->View()->Redraw(); apply_primitive(index,controller,scene); snapshot(index,frame,controller,scene); }
}
static bool meaningful_case(int index) {
  const int operation = index / 2400;
  const bool pixel = (index / 25) % 2 != 0, anchor = (index / 50) % 2 != 0;
  const bool minimum = (index / 200) % 2 != 0, focus = (index / 400) % 3 != 0;
  if (operation == 0) return (index / 5) % 5 == 0 && !minimum && !focus;
  if (operation == 1) return !pixel && !minimum && !focus;
  if (operation == 2) return !focus;
  return !pixel && !anchor && !minimum;
}
class PrimitiveStopFailure : public AIS_Animation {
public:
  PrimitiveStopFailure() : AIS_Animation("primitive stop failure") {}
  void Stop() override { throw Standard_Failure("primitive child stop rejected; expected successful stop"); }
};
static void run_stop_failure(int operation) {
  const int index = operation * 2400 + (operation == 3 ? 2 : 1);
  auto scene = primitive_scene(index); PrimitiveProbe controller; controller.seed(index);
  controller.ViewAnimation()->SetView(scene.view);
  controller.ViewAnimation()->Add(new PrimitiveStopFailure());
  controller.ViewAnimation()->Start(false);
  scene.view->View()->Redraw(); snapshot(index,0,controller,scene);
  bool failed = false;
  try { apply_primitive(index,controller,scene); } catch (const Standard_Failure&) { failed = true; }
  if (failed != (operation < 3)) throw Standard_Failure("unexpected primitive animation failure order");
  snapshot(index,1,controller,scene);
  scene.view->View()->Redraw(); apply_primitive(index,controller,scene); snapshot(index,2,controller,scene);
}
static void run_zoom_limit(int scaleIndex, int offset) {
  const int seed = 4825 + offset, index = 10000 * (scaleIndex + 1) + seed;
  auto scene = primitive_scene(seed); scene.view->Camera()->SetScale(scaleIndex == 0 ? 1e-9 : 1e14);
  PrimitiveProbe controller; controller.seed(seed);
  controller.ViewAnimation()->SetView(scene.view); controller.ViewAnimation()->Start(false);
  scene.view->View()->Redraw(); snapshot(index,0,controller,scene);
  for (int frame=1;frame<=2;++frame) { scene.view->View()->Redraw(); apply_primitive(seed,controller,scene); snapshot(index,frame,controller,scene); }
}
int main(int argc, char** argv) {
  std::cout << std::setprecision(17);
  if (argc > 1 && std::string(argv[1]) == "limits") {
    for (int scaleIndex=0;scaleIndex<2;++scaleIndex) for (int offset=0;offset<25;++offset) run_zoom_limit(scaleIndex,offset);
    return 0;
  }
  if (argc > 1) { for (int operation=0;operation<4;++operation) run_stop_failure(operation); return 0; }
  for (int index=0;index<9600;++index) if (meaningful_case(index)) run_primitive(index);
}
