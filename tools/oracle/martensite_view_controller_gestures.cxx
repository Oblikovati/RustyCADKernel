// Local Martensite oracle: a real V3d_View forwards invalidation to a recording CPU driver.
#include <AIS_ViewController.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
#include "tkservice_probe_null_driver.cxx"
#include <iomanip>
#include <iostream>
#include <limits>
class GestureRecordingView : public SvView {
public:
  bool invalidated = false;
  GestureRecordingView(const occ::handle<Graphic3d_StructureManager>& m) : SvView(m, {}) {}
  void Invalidate() override { invalidated = true; }
  bool IsInvalidated() override { return invalidated; }
  void Redraw() override { invalidated = false; }
};
class GestureRecordingDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& m) override {
    return new GestureRecordingView(m);
  }
};
static double scale(int index) {
  const double values[] = {1, 0, -1, 2, std::numeric_limits<double>::infinity(), std::numeric_limits<double>::quiet_NaN()};
  return values[(index / 60) % 6];
}
static NCollection_Vec2<double> origin(int touch) {
  return NCollection_Vec2<double>(-3.25 + touch * 20, 7.5 + touch * 3);
}
static NCollection_Vec2<double> displacement(int index, int touch, int frame) {
  const double moves[] = {0, 4, 6, 6.0001, -8, 0.25, 3.9999, 4.0001};
  const double dx = moves[(index / 5 + frame) % 8];
  const double dy = moves[(index / 40 + frame) % 8];
  return NCollection_Vec2<double>(dx * (touch == 1 && index % 2 == 0 ? -1 : 1), dy);
}
class GestureProbe : public AIS_ViewController {
public:
  void configure(int index) {
    myNavigationMode = AIS_NavigationMode((index / 5) % 3);
    myTouchToleranceScale = float(scale(index));
    myMouseAccel = index % 2 == 0 ? 2.0f : -0.5f;
    myOrbitAccel = index % 3 == 0 ? 0.0f : 1.5f;
    myToAllowTouchZRotation = (index / 15) % 2 != 0;
    myToAllowRotation = myToAllowPanning = myToAllowZooming = false;
    myStartRotCoord.SetValues(-11.25, 12.5); myStartPanCoord.SetValues(-13.25, 14.5);
    myUpdateStartPointRot = (index / 30) % 2 == 0; myUpdateStartPointPan = (index / 20) % 2 == 0;
    myGL.OrbitRotation.PointStart.SetValues(31.25, -32.5);
    myGL.ViewRotation.PointStart.SetValues(33.25, -34.5);
    myGL.ZRotate.Point.SetValues(35, -36); myGL.ZRotate.Angle = -0.75;
    seed_contacts(index);
    if (index >= 360) configure_boundary(index);
  }
  void advance(int index, int frame) {
    if (index >= 360) { advance_boundary(index); return; }
    for (int touch = 0; touch < myTouchPoints.Extent(); ++touch)
      myTouchPoints.ChangeFromIndex(touch + 1).To = origin(touch) + displacement(index, touch, frame);
  }
  void synthesize(const occ::handle<V3d_View>& view) { flushGestures({}, view); }
  void snapshot(int index, int frame, bool invalidated) {
    std::cout << index << ' ' << frame << ' ' << invalidated << ' ' << myNbTouchesLast
              << ' ' << myUpdateStartPointRot << ' ' << myUpdateStartPointPan;
    emit_contacts(); emit_motion();
    std::cout << '\n';
  }
private:
  void seed_contacts(int index) {
    for (int touch = 0; touch < index % 5; ++touch) {
      Aspect_Touch t(origin(touch), ((index / 10) >> touch) & 1);
      myTouchPoints.Add(size_t(touch + 1), t);
    }
  }
  void configure_boundary(int index) {
    myTouchPoints.Clear(); myTouchToleranceScale = 1.0f; myToAllowTouchZRotation = true;
    myUpdateStartPointRot = false; myUpdateStartPointPan = false;
    myTouchPoints.Add(1, Aspect_Touch(NCollection_Vec2<double>(0, 0), index >= 368));
    if (index < 368) myTouchPoints.Add(2, Aspect_Touch(NCollection_Vec2<double>(20, 0), false));
  }
  void advance_boundary(int index) {
    if (index < 368) {
      const double x[] = {-20, 0, 1e-9, 0, 26, 26.0001, 20, 20};
      const double y[] = {0, 0, 0, 20, 0, 0, 0.7, 0.8};
      myTouchPoints.ChangeFromIndex(2).To.SetValues(x[index - 360], y[index - 360]);
      return;
    }
    const double small = std::numeric_limits<double>::min();
    const double x[] = {small * 0.5, small, small * 2, 0};
    myTouchPoints.ChangeFromIndex(1).To.SetValues(x[index - 368], 0);
  }
  void emit_contacts() {
    std::cout << ' ' << myTouchPoints.Extent();
    for (int i = 1; i <= 4; ++i) {
      if (i > myTouchPoints.Extent()) { std::cout << " 0 0 0 0 0"; continue; }
      const auto& t = myTouchPoints.FindFromIndex(i);
      std::cout << ' ' << t.From.x() << ' ' << t.From.y() << ' ' << t.To.x() << ' ' << t.To.y() << ' ' << t.IsPreciseDevice;
    }
  }
  void emit_motion() {
    std::cout << ' ' << myGL.IsNewGesture << ' ' << myGL.Dragging.ToMove << ' ' << myGL.Dragging.PointTo.x() << ' ' << myGL.Dragging.PointTo.y();
    emit_rotations();
    std::cout << ' ' << myGL.ZRotate.ToRotate << ' ' << myGL.ZRotate.Point.x() << ' ' << myGL.ZRotate.Point.y() << ' ' << myGL.ZRotate.Angle
              << ' ' << myGL.Panning.ToStart << ' ' << myGL.Panning.PointStart.x() << ' ' << myGL.Panning.PointStart.y()
              << ' ' << myGL.Panning.ToPan << ' ' << myGL.Panning.Delta.x() << ' ' << myGL.Panning.Delta.y();
    emit_zoom();
  }
  void emit_rotations() {
    std::cout << ' ' << myGL.OrbitRotation.ToStart << ' ' << myGL.OrbitRotation.PointStart.x() << ' ' << myGL.OrbitRotation.PointStart.y()
              << ' ' << myGL.OrbitRotation.ToRotate << ' ' << myGL.OrbitRotation.PointTo.x() << ' ' << myGL.OrbitRotation.PointTo.y()
              << ' ' << myGL.ViewRotation.ToStart << ' ' << myGL.ViewRotation.PointStart.x() << ' ' << myGL.ViewRotation.PointStart.y()
              << ' ' << myGL.ViewRotation.ToRotate << ' ' << myGL.ViewRotation.PointTo.x() << ' ' << myGL.ViewRotation.PointTo.y();
  }
  void emit_zoom() {
    std::cout << ' ' << myGL.ZoomActions.Size();
    for (int i = 1; i <= 3; ++i) {
      if (i > myGL.ZoomActions.Size()) { std::cout << " 0 0 0 0"; continue; }
      const auto& action = myGL.ZoomActions.Value(i);
      std::cout << ' ' << action.Point.x() << ' ' << action.Point.y() << ' ' << action.Delta << ' ' << action.Flags;
    }
  }
};
int main() {
  std::cout << std::setprecision(17);
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new GestureRecordingDriver());
  auto view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(640, 480); window->SetVirtual(true);
  view->SetWindow(window);
  for (int index = 0; index < 372; ++index) {
    GestureProbe probe; probe.configure(index);
    for (int frame = 0; frame < 3; ++frame) {
      view->View()->Redraw(); probe.advance(index, frame); probe.synthesize(view);
      probe.snapshot(index, frame, view->IsInvalidated());
    }
  }
}
