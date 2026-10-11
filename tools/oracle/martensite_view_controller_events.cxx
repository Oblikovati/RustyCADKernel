// Local-only OCCT 3d097a0 oracle; Rust tests replay committed assets without native dependencies.
#include <AIS_ViewController.hxx>
#include <iomanip>
#include <iostream>
static void parameter_group_0(const AIS_ViewController& c) {
  std::cout << ' ' << static_cast<double>(c.ToPauseObjectsAnimation());
  std::cout << ' ' << static_cast<double>(c.IsContinuousRedraw());
  std::cout << ' ' << static_cast<double>(c.ToShowPanAnchorPoint());
  std::cout << ' ' << static_cast<double>(c.ToShowRotateCenter());
  std::cout << ' ' << static_cast<double>(c.ToLockOrbitZUp());
  std::cout << ' ' << static_cast<double>(c.ToAllowTouchZRotation());
  std::cout << ' ' << static_cast<double>(c.ToAllowRotation());
  std::cout << ' ' << static_cast<double>(c.ToAllowPanning());
  std::cout << ' ' << static_cast<double>(c.ToAllowZooming());
}
static void parameter_group_1(const AIS_ViewController& c) {
  std::cout << ' ' << static_cast<double>(c.ToAllowZFocus());
  std::cout << ' ' << static_cast<double>(c.ToAllowHighlight());
  std::cout << ' ' << static_cast<double>(c.ToAllowDragging());
  std::cout << ' ' << static_cast<double>(c.ToStickToRayOnZoom());
  std::cout << ' ' << static_cast<double>(c.ToStickToRayOnRotation());
  std::cout << ' ' << static_cast<double>(c.ToInvertPitch());
  std::cout << ' ' << static_cast<double>(c.ToDisplayXRAuxDevices());
  std::cout << ' ' << static_cast<double>(c.ToDisplayXRHands());
  std::cout << ' ' << static_cast<double>(c.MouseAcceleration());
}
static void parameter_group_2(const AIS_ViewController& c) {
  std::cout << ' ' << static_cast<double>(c.OrbitAcceleration());
  std::cout << ' ' << static_cast<double>(c.WalkSpeedAbsolute());
  std::cout << ' ' << static_cast<double>(c.WalkSpeedRelative());
  std::cout << ' ' << static_cast<double>(c.ThrustSpeed());
  std::cout << ' ' << static_cast<double>(c.TouchToleranceScale());
  std::cout << ' ' << static_cast<double>(c.MouseDoubleClickInterval());
  std::cout << ' ' << static_cast<double>(c.MinZoomDistance());
  std::cout << ' ' << static_cast<double>(c.RotationMode());
}
static void parameters(const char* name, const AIS_ViewController& c) {
  std::cout << name;
  parameter_group_0(c); parameter_group_1(c); parameter_group_2(c);
  std::cout << '\n';
}
static void event(const char* name, const AIS_ViewController& c, int result = -1,
                  AIS_ViewInputBufferType role = AIS_ViewInputBufferType_UI) {
  const auto& b = c.InputBuffer(role);
  std::cout << name << ' ' << result << ' ' << int(b.Selection.Tool) << ' ' << int(b.Selection.Scheme)
    << ' ' << b.Selection.ToApplyTool << ' ' << b.Selection.Points.Length();
  for (const auto& p : b.Selection.Points) std::cout << ' ' << p.x() << ' ' << p.y();
  std::cout << ' ' << b.ZoomActions.Length();
  for (const auto& z : b.ZoomActions) std::cout << ' ' << z.Point.x() << ' ' << z.Point.y() << ' ' << z.Delta << ' ' << z.Flags;
  std::cout << ' ' << b.ZRotate.ToRotate << ' ' << b.ZRotate.Angle << ' ' << b.OrbitRotation.ToStart
    << ' ' << b.OrbitRotation.ToRotate << ' ' << b.ViewRotation.ToStart << ' ' << b.ViewRotation.ToRotate
    << ' ' << int(c.NavigationMode()) << ' ' << c.HasPreviousMoveTo()
    << ' ' << c.PreviousMoveTo().x() << ' ' << c.PreviousMoveTo().y() << '\n';
}
static void selection() {
  AIS_ViewController c; event("default-ui", c); event("default-gl", c, -1, AIS_ViewInputBufferType_GL);
  c.SelectInViewer(NCollection_Vec2<int>(1, 2), AIS_SelectionScheme_Add); event("pick-first", c);
  c.SelectInViewer(NCollection_Vec2<int>(3, 4), AIS_SelectionScheme_XOR); event("pick-second", c);
  c.UpdateRubberBand(NCollection_Vec2<int>(5, 6), NCollection_Vec2<int>(7, 8)); event("rectangle", c);
  c.UpdatePolySelection(NCollection_Vec2<int>(9, 10), false); event("poly-first", c);
  c.UpdatePolySelection(NCollection_Vec2<int>(9, 10), true); event("poly-duplicate", c);
  c.UpdatePolySelection(NCollection_Vec2<int>(11, 12), true); event("poly-append", c);
  c.UpdatePolySelection(NCollection_Vec2<int>(13, 14), false); event("poly-replace", c);
  c.SelectInViewer(NCollection_Vec2<int>(15, 16), AIS_SelectionScheme_Remove); event("pick-transition", c);
  event("selection-gl", c, -1, AIS_ViewInputBufferType_GL);
}
static void selection_sequences() {
  for (int length = 0; length <= 3; ++length) {
    AIS_ViewController c; NCollection_Sequence<NCollection_Vec2<int>> points;
    for (int i=0; i<length; ++i) points.Append(NCollection_Vec2<int>(i, -i));
    c.SelectInViewer(points, AIS_SelectionScheme_ReplaceExtra); points.Clear();
    std::string name="sequence-"+std::to_string(length); event(name.c_str(), c);
    c.UpdateRubberBand(NCollection_Vec2<int>(5,6), NCollection_Vec2<int>(7,8));
    name="sequence-rectangle-"+std::to_string(length); event(name.c_str(), c);
    c.UpdatePolySelection(NCollection_Vec2<int>(9,10), false);
    name="sequence-polygon-"+std::to_string(length); event(name.c_str(), c);
    c.SelectInViewer(NCollection_Vec2<int>(15,16), AIS_SelectionScheme_Remove);
    name="sequence-pick-"+std::to_string(length); event(name.c_str(), c);
  }
}
static void zoom() {
  AIS_ViewController c;
  event("zoom-first", c, c.UpdateZoom(Aspect_ScrollDelta(NCollection_Vec2<int>(1,2), .5, Aspect_VKeyFlags_SHIFT)));
  event("zoom-merge", c, c.UpdateZoom(Aspect_ScrollDelta(NCollection_Vec2<int>(1,2), -.5, Aspect_VKeyFlags_CTRL)));
  event("zoom-next", c, c.UpdateZoom(Aspect_ScrollDelta(NCollection_Vec2<int>(3,4), 2., Aspect_VKeyFlags_ALT)));
  event("zoom-revisit", c, c.UpdateZoom(Aspect_ScrollDelta(NCollection_Vec2<int>(1,2), .25, Aspect_VKeyFlags_CTRL)));
  event("scroll-merge", c, c.UpdateMouseScroll(Aspect_ScrollDelta(NCollection_Vec2<int>(1,2), .25, Aspect_VKeyFlags_SHIFT)));
  event("scroll-undefined", c, c.UpdateMouseScroll(Aspect_ScrollDelta(-.5, Aspect_VKeyFlags_NONE)));
  c.SetAllowZooming(false);
  event("zoom-disabled", c, c.UpdateZoom(Aspect_ScrollDelta(-.25, Aspect_VKeyFlags_CTRL)));
  event("scroll-disabled", c, c.UpdateMouseScroll(Aspect_ScrollDelta(.5, Aspect_VKeyFlags_ALT)));
}
static void navigation(AIS_ViewController& c);
static void rotation() {
  AIS_ViewController c; event("z-disabled", c, c.UpdateZRotation(.5)); c.SetAllowTouchZRotation(true);
  event("z-first", c, c.UpdateZRotation(.5)); event("z-merge", c, c.UpdateZRotation(-.25));
  c.SetAllowTouchZRotation(false); event("z-paused", c, c.UpdateZRotation(1.));
  c.ChangeInputBuffer(AIS_ViewInputBufferType_UI).Reset(); c.SetAllowTouchZRotation(true);
  event("z-restarted", c, c.UpdateZRotation(-.5));
  navigation(c);
}
static void navigation(AIS_ViewController& c) {
  for (int mode=0; mode<=2; ++mode) {
    for (auto role : {AIS_ViewInputBufferType_UI, AIS_ViewInputBufferType_GL}) {
      auto& b=c.ChangeInputBuffer(role);
      b.OrbitRotation.ToStart=b.OrbitRotation.ToRotate=b.ViewRotation.ToStart=b.ViewRotation.ToRotate=true;
    }
    c.SetNavigationMode(static_cast<AIS_NavigationMode>(mode));
    std::string name="navigation-ui-"+std::to_string(mode); event(name.c_str(), c);
    name="navigation-gl-"+std::to_string(mode); event(name.c_str(), c, -1, AIS_ViewInputBufferType_GL);
    c.SetNavigationMode(static_cast<AIS_NavigationMode>(mode));
    name="navigation-repeat-"+std::to_string(mode); event(name.c_str(), c);
  }
  c.ResetPreviousMoveTo(); event("previous-reset", c);
}
static void settings_group_0(AIS_ViewController& c) {
  c.SetPauseObjectsAnimation(!c.ToPauseObjectsAnimation()); parameters("parameter-0", c); c.SetPauseObjectsAnimation(false);
  c.SetContinuousRedraw(!c.IsContinuousRedraw()); parameters("parameter-1", c); c.SetContinuousRedraw(false);
  c.SetShowPanAnchorPoint(!c.ToShowPanAnchorPoint()); parameters("parameter-2", c); c.SetShowPanAnchorPoint(true);
  c.SetShowRotateCenter(!c.ToShowRotateCenter()); parameters("parameter-3", c); c.SetShowRotateCenter(true);
  c.SetLockOrbitZUp(!c.ToLockOrbitZUp()); parameters("parameter-4", c); c.SetLockOrbitZUp(false);
  c.SetAllowTouchZRotation(!c.ToAllowTouchZRotation()); parameters("parameter-5", c); c.SetAllowTouchZRotation(false);
  c.SetAllowRotation(!c.ToAllowRotation()); parameters("parameter-6", c); c.SetAllowRotation(true);
  c.SetAllowPanning(!c.ToAllowPanning()); parameters("parameter-7", c); c.SetAllowPanning(true);
  c.SetAllowZooming(!c.ToAllowZooming()); parameters("parameter-8", c); c.SetAllowZooming(true);
  c.SetAllowZFocus(!c.ToAllowZFocus()); parameters("parameter-9", c); c.SetAllowZFocus(true);
  c.SetAllowHighlight(!c.ToAllowHighlight()); parameters("parameter-10", c); c.SetAllowHighlight(true);
  c.SetAllowDragging(!c.ToAllowDragging()); parameters("parameter-11", c); c.SetAllowDragging(true);
  c.SetStickToRayOnZoom(!c.ToStickToRayOnZoom()); parameters("parameter-12", c); c.SetStickToRayOnZoom(true);
}
static void settings_group_1(AIS_ViewController& c) {
  c.SetStickToRayOnRotation(!c.ToStickToRayOnRotation()); parameters("parameter-13", c); c.SetStickToRayOnRotation(true);
  c.SetInvertPitch(!c.ToInvertPitch()); parameters("parameter-14", c); c.SetInvertPitch(false);
  c.SetDisplayXRAuxDevices(!c.ToDisplayXRAuxDevices()); parameters("parameter-15", c); c.SetDisplayXRAuxDevices(false);
  c.SetDisplayXRHands(!c.ToDisplayXRHands()); parameters("parameter-16", c); c.SetDisplayXRHands(true);
  c.SetMouseAcceleration(-18.25); parameters("parameter-17", c); c.SetMouseAcceleration(1.0);
  c.SetOrbitAcceleration(-19.25); parameters("parameter-18", c); c.SetOrbitAcceleration(1.0);
  c.SetWalkSpeedAbsolute(-20.25); parameters("parameter-19", c); c.SetWalkSpeedAbsolute(1.5);
  c.SetWalkSpeedRelative(-21.25); parameters("parameter-20", c); c.SetWalkSpeedRelative(0.1);
  c.SetThrustSpeed(-22.25); parameters("parameter-21", c); c.SetThrustSpeed(0.0);
  c.SetTouchToleranceScale(-23.25); parameters("parameter-22", c); c.SetTouchToleranceScale(1.0);
  c.SetMouseDoubleClickInterval(-24.25); parameters("parameter-23", c); c.SetMouseDoubleClickInterval(0.4);
  c.SetMinZoomDistance(-25.25); parameters("parameter-24", c); c.SetMinZoomDistance(1.0);
  c.SetRotationMode(AIS_RotationMode_PickCenter); parameters("parameter-25", c); c.SetRotationMode(AIS_RotationMode_BndBoxActive);
}
static void settings() {
  AIS_ViewController c; parameters("parameters-default", c);
  settings_group_0(c); settings_group_1(c);
  parameters("parameters-restored", c);
}
class PreviousMoveToProbe : public AIS_ViewController {
public:
  void SeedPrevious() { myPrevMoveTo = NCollection_Vec2<int>(4, 8); }
};
static void previous() {
  PreviousMoveToProbe c; c.SeedPrevious();
  std::cout << "previous-defined " << c.HasPreviousMoveTo() << ' ' << c.PreviousMoveTo().x() << ' ' << c.PreviousMoveTo().y() << '\n';
  c.ResetPreviousMoveTo();
  std::cout << "previous-cleared " << c.HasPreviousMoveTo() << ' ' << c.PreviousMoveTo().x() << ' ' << c.PreviousMoveTo().y() << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  settings(); selection(); selection_sequences(); zoom(); rotation(); previous();
}
