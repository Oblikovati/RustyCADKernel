// AIS_ViewController event dispatch over a context of three points in a 200x100 view: point,
// rubber-band, polygon and zoom-window selection, dynamic highlighting, view orientation keys
// and requests, dragging start/stop and redraw requests, the twin of
// crates/tkv3d/examples/v3_probe/ais_view_controller.rs. A request is
// `v3_ais_view_controller Count (Op A B C D)...`.
#include <AIS_AnimationCamera.hxx>
#include <AIS_InteractiveContext.hxx>
#include <AIS_Point.hxx>
#include <AIS_RubberBand.hxx>
#include <AIS_ViewController.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <Geom_CartesianPoint.hxx>

class V3ProbeController : public AIS_ViewController {
public:
  std::vector<int> events;
  void OnSelectionChanged(const occ::handle<AIS_InteractiveContext>&, const occ::handle<V3d_View>&) override { events.push_back(1); }
  void OnObjectDragged(const occ::handle<AIS_InteractiveContext>& c, const occ::handle<V3d_View>& v, AIS_DragAction a) override {
    events.push_back(10 + (int)a); AIS_ViewController::OnObjectDragged(c, v, a);
  }
  void OnSubviewChanged(const occ::handle<AIS_InteractiveContext>&, const occ::handle<V3d_View>&, const occ::handle<V3d_View>&) override { events.push_back(3); }
  const occ::handle<AIS_RubberBand>& band() const { return myRubberBand; }
  bool dragging() const { return !myDragObject.IsNull(); }
  bool askNext() const { return myToAskNextFrame; }
  const occ::handle<AIS_AnimationCamera>& animation() const { return myViewAnimation; }
};

static void v3_controller_state(V3ProbeController& c, const occ::handle<AIS_InteractiveContext>& ctx,
                                const occ::handle<V3d_View>& view, const std::vector<occ::handle<AIS_Point>>& points) {
  ga_int(ctx->NbSelected());
  for (const auto& p : points) ga_flag(ctx->IsSelected(p));
  ga_flag(ctx->HasDetected());
  int detected = -1;
  for (size_t i = 0; i < points.size(); ++i) if (ctx->HasDetected() && ctx->DetectedInteractive() == points[i]) detected = (int)i;
  ga_int(detected);
  ga_flag(ctx->IsDisplayed(c.band()));
  ga_int(c.band()->Points().Size());
  for (const auto& p : c.band()->Points()) { ga_int(p.x()); ga_int(p.y()); }
  ga_flag(c.band()->IsPolygonClosed());
  ga_flag(c.dragging()); ga_flag(c.askNext()); ga_flag(c.HasPreviousMoveTo());
  ga_int(c.PreviousMoveTo().x()); ga_int(c.PreviousMoveTo().y());
  ga_flag(c.animation()->IsStopped()); ga_flag(c.animation()->View() == view);
  // A running animation interpolates at the elapsed time: the view camera is rounded.
  const occ::handle<Graphic3d_Camera>& cam = view->Camera();
  const double values[7] = {cam->Eye().X(), cam->Eye().Y(), cam->Eye().Z(), cam->Up().X(), cam->Up().Y(), cam->Up().Z(), cam->Scale()};
  for (double v : values) ga_real(std::round(v * 100.0) / 100.0 + 0.0);
  for (const occ::handle<Graphic3d_Camera>& end : {c.animation()->CameraStart(), c.animation()->CameraEnd()}) {
    ga_flag(end.IsNull());
    if (!end.IsNull()) { ga_real(end->Eye().X()); ga_real(end->Eye().Y()); ga_real(end->Eye().Z()); ga_real(end->Up().X()); ga_real(end->Up().Y()); ga_real(end->Up().Z()); }
  }
  ga_flag(view->IsInvalidatedImmediate());
  ga_int((int)c.events.size()); for (int e : c.events) ga_int(e); c.events.clear();
}

void v3_ais_view_controller(std::istream& input) {
  GaReader r{input};
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  occ::handle<V3d_View> view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(200, 100); view->SetWindow(window);
  occ::handle<AIS_InteractiveContext> ctx = new AIS_InteractiveContext(viewer);
  std::vector<occ::handle<AIS_Point>> points;
  const double xs[3] = {-10.0, 0.0, 10.0};
  for (double x : xs) { points.push_back(new AIS_Point(new Geom_CartesianPoint(x, 0.0, 0.0))); ctx->Display(points.back(), false); }
  view->SetProj(V3d_TypeOfOrientation_Zup_Top); view->FitAll(0.1, false);
  for (double x : xs) { int px = 0, py = 0; view->Convert(x, 0.0, 0.0, px, py); ga_int(px); ga_int(py); }
  V3ProbeController c;
  v3_controller_state(c, ctx, view, points);
  for (int count = r.integer(); count > 0; --count) {
    const int op = r.integer(), a = r.integer(), b = r.integer(), d = r.integer(), e = r.integer();
    AIS_ViewInputBuffer& ui = c.ChangeInputBuffer(AIS_ViewInputBufferType_UI);
    switch (op) {
      case 0: c.SelectInViewer(NCollection_Vec2<int>(a, b), AIS_SelectionScheme(d)); break;
      case 1: ui.MoveTo.ToHilight = true; ui.MoveTo.Point = NCollection_Vec2<int>(a, b); break;
      case 2: c.UpdateRubberBand(NCollection_Vec2<int>(a, b), NCollection_Vec2<int>(d, e)); break;
      case 3: c.UpdatePolySelection(NCollection_Vec2<int>(a, b), d != 0); break;
      case 4: ui.Selection.ToApplyTool = true; ui.Selection.Scheme = AIS_SelectionScheme(a); break;
      case 5: ui.Selection.Tool = AIS_ViewSelectionTool(a); break;
      case 6: c.KeyDown(Aspect_VKey(a), 0.0); break;
      case 7: c.UpdateViewOrientation(V3d_TypeOfOrientation(a), b != 0); break;
      case 8: GA_TRY(c.FlushViewEvents(ctx, view, a != 0)) break;
      case 9: ui.Dragging.ToStart = true; ui.Dragging.PointStart = NCollection_Vec2<int>(a, b); break;
      case 10: ui.Dragging.ToStop = true; break;
      case 11: c.SetContinuousRedraw(a != 0); break;
      case 12: c.SetAllowHighlight(a != 0); break;
      case 13: c.ResetPreviousMoveTo(); break;
      case 14: c.AbortViewAnimation(); break;
    }
    v3_controller_state(c, ctx, view, points);
  }
  ga_end();
}
