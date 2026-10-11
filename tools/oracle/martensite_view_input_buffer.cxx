// OCCT 3d097a0 input buffer oracle; committed numeric fixtures keep Rust tests independent.
#include <AIS_ViewInputBuffer.hxx>
#include <iomanip>
#include <iostream>
static void flags(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << b.IsNewGesture << ' ' << b.Orientation.ToFitAll << ' ' << b.Orientation.ToSetViewOrient
    << ' ' << b.MoveTo.ToHilight << ' ' << b.Selection.ToApplyTool << ' ' << b.Panning.ToStart << ' ' << b.Panning.ToPan
    << ' ' << b.Dragging.ToStart << ' ' << b.Dragging.ToConfirm << ' ' << b.Dragging.ToMove << ' ' << b.Dragging.ToStop
    << ' ' << b.Dragging.ToAbort << ' ' << b.OrbitRotation.ToStart << ' ' << b.OrbitRotation.ToRotate
    << ' ' << b.ViewRotation.ToStart << ' ' << b.ViewRotation.ToRotate << ' ' << b.ZRotate.ToRotate;
}
template<class T> static void point(const NCollection_Vec2<T>& p) {
  std::cout << ' ' << p.x() << ' ' << p.y();
}
static void values(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << int(b.Orientation.ViewOrient) << ' ' << int(b.Selection.Tool) << ' ' << int(b.Selection.Scheme);
  point(b.MoveTo.Point); point(b.Panning.PointStart); point(b.Panning.Delta);
  point(b.Dragging.PointStart); point(b.Dragging.PointTo);
  point(b.OrbitRotation.PointStart); point(b.OrbitRotation.PointTo);
  point(b.ViewRotation.PointStart); point(b.ViewRotation.PointTo); point(b.ZRotate.Point);
  std::cout << ' ' << b.ZRotate.Angle << ' ' << b.Selection.Points.Length();
  for (const auto& p : b.Selection.Points) point(p);
  std::cout << ' ' << b.ZoomActions.Length();
  for (const auto& z : b.ZoomActions) { point(z.Point); std::cout << ' ' << z.Delta << ' ' << z.Flags; }
}
static void dump(const char* name, const AIS_ViewInputBuffer& b) {
  std::cout << name; flags(b); values(b); std::cout << '\n';
}
static void pending(AIS_ViewInputBuffer& b) {
  b.IsNewGesture = b.Orientation.ToFitAll = b.Orientation.ToSetViewOrient = b.MoveTo.ToHilight = b.Selection.ToApplyTool = true;
  b.Panning.ToStart = b.Panning.ToPan = b.Dragging.ToStart = b.Dragging.ToConfirm = b.Dragging.ToMove = b.Dragging.ToStop = b.Dragging.ToAbort = true;
  b.OrbitRotation.ToStart = b.OrbitRotation.ToRotate = b.ViewRotation.ToStart = b.ViewRotation.ToRotate = b.ZRotate.ToRotate = true;
}
static void positions(AIS_ViewInputBuffer& b) {
  b.MoveTo.Point = NCollection_Vec2<int>(17,19); b.Panning.PointStart = NCollection_Vec2<int>(23,-29); b.Panning.Delta = NCollection_Vec2<int>(-3,7);
  b.Dragging.PointStart = NCollection_Vec2<int>(31,37); b.Dragging.PointTo = NCollection_Vec2<int>(41,43);
  b.OrbitRotation.PointStart = NCollection_Vec2<double>(.25,-.5); b.OrbitRotation.PointTo = NCollection_Vec2<double>(1.5,2.25);
  b.ViewRotation.PointStart = NCollection_Vec2<double>(-.75,3.5); b.ViewRotation.PointTo = NCollection_Vec2<double>(4.25,-5.5);
  b.ZRotate.Point = NCollection_Vec2<int>(47,53); b.ZRotate.Angle = .75;
}
static AIS_ViewInputBuffer seeded() {
  AIS_ViewInputBuffer b; pending(b); positions(b);
  b.Orientation.ViewOrient = V3d_Zneg; b.Selection.Tool = AIS_ViewSelectionTool_Polygon; b.Selection.Scheme = AIS_SelectionScheme_XOR;
  b.Selection.Points.Append(NCollection_Vec2<int>(27,-14)); b.Selection.Points.Append(NCollection_Vec2<int>(61,67));
  b.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(71,73),.5,Aspect_VKeyFlags_SHIFT));
  b.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(79,83),-1.25,Aspect_VKeyFlags_CTRL));
  return b;
}
int main() {
  std::cout << std::setprecision(17);
  std::cout << "enums " << AIS_ViewSelectionTool_Picking << ' ' << AIS_ViewSelectionTool_RubberBand << ' ' << AIS_ViewSelectionTool_Polygon << ' ' << AIS_ViewSelectionTool_ZoomWindow << ' ' << AIS_ViewInputBufferType_UI << ' ' << AIS_ViewInputBufferType_GL << '\n';
  AIS_ViewInputBuffer idle; dump("default",idle);
  auto b = seeded(); dump("pending",b); auto copy = b; b.Reset(); dump("reset",b); b.Reset(); dump("reset_twice",b); dump("copy",copy);
  copy.Selection.Points.Clear(); copy.ZoomActions.Clear(); dump("copy_modified",copy);
}
