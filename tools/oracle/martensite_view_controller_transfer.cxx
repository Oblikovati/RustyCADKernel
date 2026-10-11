// Local Martensite oracle for OCCT input-buffer transfer, without a viewer or GPU.
#include <AIS_ViewController.hxx>
#include <iomanip>
#include <iostream>

static bool flag(int index, int side, int bit) {
  const unsigned pattern = unsigned(index * (side == 0 ? 73 : 151) + side * 37);
  return ((pattern >> bit) & 1) != 0;
}
static void seed_drag(AIS_ViewInputBuffer& b, int mask, int origin) {
  b.Dragging.ToStart = (mask & 1) != 0;
  b.Dragging.ToConfirm = (mask & 2) != 0;
  b.Dragging.ToMove = (mask & 4) != 0;
  b.Dragging.ToStop = (mask & 8) != 0;
  b.Dragging.ToAbort = (mask & 16) != 0;
  b.Dragging.PointStart.SetValues(origin + 9, origin + 10);
  b.Dragging.PointTo.SetValues(origin + 11, origin + 12);
}
static void seed_rotation(AIS_ViewInputBuffer& b, int index, int side, int o) {
  b.OrbitRotation.ToStart = flag(index, side, 7);
  b.OrbitRotation.ToRotate = flag(index, side, 8);
  b.OrbitRotation.PointStart.SetValues(o + 13.25, o + 14.5);
  b.OrbitRotation.PointTo.SetValues(o + 15.75, o + 16.125);
  b.ViewRotation.ToStart = flag(index, side, 9);
  b.ViewRotation.ToRotate = flag(index, side, 10);
  b.ViewRotation.PointStart.SetValues(o + 17.25, o + 18.5);
  b.ViewRotation.PointTo.SetValues(o + 19.75, o + 20.125);
  b.ZRotate.ToRotate = flag(index, side, 11);
  b.ZRotate.Point.SetValues(o + 21, o + 22);
  b.ZRotate.Angle = o + 23.5;
}
static void seed_selection(AIS_ViewInputBuffer& b, int index, int side, int o) {
  b.Selection.Tool = AIS_ViewSelectionTool((index + side) % 4);
  b.Selection.Scheme = AIS_SelectionScheme((index + side * 3) % 7 - 1);
  b.Selection.ToApplyTool = flag(index, side, 4);
  for (int p = 0; p < 2 + side; ++p)
    b.Selection.Points.Append(NCollection_Vec2<int>(o + 30 + p * 2, o + 31 + p * 2));
  for (int z = 0; z < 2 - side; ++z)
    b.ZoomActions.Append(Aspect_ScrollDelta(NCollection_Vec2<int>(o + 40 + z * 2, o + 41 + z * 2),
                                           o + 42.25 + z, Aspect_VKeyFlags(256 * (z + side + 1))));
}
static void seed_buffer(AIS_ViewInputBuffer& b, int index, int side) {
  const int o = index * 100 + side * 1000000;
  b.IsNewGesture = flag(index, side, 0);
  b.Orientation.ToFitAll = flag(index, side, 1);
  b.Orientation.ToSetViewOrient = flag(index, side, 2);
  b.Orientation.ViewOrient = V3d_TypeOfOrientation((index + side * 7) % 26);
  b.MoveTo.ToHilight = flag(index, side, 3);
  b.MoveTo.Point.SetValues(o + 1, o + 2);
  b.Panning.ToStart = flag(index, side, 5);
  b.Panning.ToPan = flag(index, side, 6);
  b.Panning.PointStart.SetValues(o + 3, o + 4);
  b.Panning.Delta.SetValues(o + 5, o + 6);
  seed_drag(b, side == 0 ? index % 32 : index / 32, o);
  seed_rotation(b, index, side, o);
  seed_selection(b, index, side, o);
}
template <typename T> static void emit_point(const NCollection_Vec2<T>& p) {
  std::cout << ' ' << p.x() << ' ' << p.y();
}
static void emit_selection(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << int(b.Selection.Tool) << ' ' << int(b.Selection.Scheme)
            << ' ' << b.Selection.ToApplyTool << ' ' << b.Selection.Points.Size();
  for (int p = 1; p <= 3; ++p) {
    if (p <= b.Selection.Points.Size()) emit_point(b.Selection.Points.Value(p));
    else std::cout << " 0 0";
  }
}
static void emit_drag(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << b.Dragging.ToStart << ' ' << b.Dragging.ToConfirm
            << ' ' << b.Dragging.ToMove << ' ' << b.Dragging.ToStop << ' ' << b.Dragging.ToAbort;
  emit_point(b.Dragging.PointStart);
  emit_point(b.Dragging.PointTo);
}
static void emit_rotation(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << b.OrbitRotation.ToStart;
  emit_point(b.OrbitRotation.PointStart);
  std::cout << ' ' << b.OrbitRotation.ToRotate;
  emit_point(b.OrbitRotation.PointTo);
  std::cout << ' ' << b.ViewRotation.ToStart;
  emit_point(b.ViewRotation.PointStart);
  std::cout << ' ' << b.ViewRotation.ToRotate;
  emit_point(b.ViewRotation.PointTo);
  std::cout << ' ' << b.ZRotate.ToRotate;
  emit_point(b.ZRotate.Point);
  std::cout << ' ' << b.ZRotate.Angle;
}
static void emit_zoom(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << b.ZoomActions.Size();
  for (int z = 1; z <= 2; ++z) {
    if (z > b.ZoomActions.Size()) { std::cout << " 0 0 0 0"; continue; }
    const Aspect_ScrollDelta& a = b.ZoomActions.Value(z);
    emit_point(a.Point);
    std::cout << ' ' << a.Delta << ' ' << a.Flags;
  }
}
static void emit_buffer(const AIS_ViewInputBuffer& b) {
  std::cout << ' ' << b.IsNewGesture << ' ' << b.Orientation.ToFitAll
            << ' ' << b.Orientation.ToSetViewOrient << ' ' << int(b.Orientation.ViewOrient)
            << ' ' << b.MoveTo.ToHilight;
  emit_point(b.MoveTo.Point);
  emit_selection(b);
  std::cout << ' ' << b.Panning.ToStart;
  emit_point(b.Panning.PointStart);
  std::cout << ' ' << b.Panning.ToPan;
  emit_point(b.Panning.Delta);
  emit_drag(b);
  emit_rotation(b);
  emit_zoom(b);
}
class TransferProbe : public AIS_ViewController {
public:
  void seed(int index) {
    seed_buffer(myUI, index, 0);
    seed_buffer(myGL, index, 1);
  }
  void frame(int index, int frame) {
    setAskNextFrame(true);
    flushBuffers(occ::handle<AIS_InteractiveContext>(), occ::handle<V3d_View>());
    std::cout << index << ' ' << frame << ' ' << toAskNextFrame();
    emit_buffer(myUI);
    emit_buffer(myGL);
    std::cout << '\n';
  }
};
int main() {
  std::cout << std::setprecision(17);
  for (int index = 0; index < 1024; ++index) {
    TransferProbe probe;
    probe.seed(index);
    for (int frame = 0; frame < 3; ++frame) probe.frame(index, frame);
  }
}
