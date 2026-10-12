// Protected V3d myView replacement during a genuine virtual clipping getter.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local CPU oracle only.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <sstream>
#include <vector>

class ClipRetargetOwner : public V3d_View {
public:
  ClipRetargetOwner(const occ::handle<V3d_Viewer>& viewer) : V3d_View(viewer) {}
  void ReplaceDriverView(const occ::handle<Graphic3d_CView>& replacement) {
    // The callback changes the real protected field; an external old-view anchor keeps it valid.
    myView = replacement;
  }
};
class ClipRetargetCView : public SvView {
public:
  int tag;
  std::vector<std::string>* rows;
  ClipRetargetOwner* owner = nullptr;
  const occ::handle<Graphic3d_CView>* replacement = nullptr;
  mutable bool replaceOnce = false;
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes = new Graphic3d_SequenceOfHClipPlane;
  ClipRetargetCView(const occ::handle<Graphic3d_StructureManager>& manager, int identity,
                   std::vector<std::string>& trace) : SvView(manager, {}), tag(identity), rows(&trace) {}
  const occ::handle<Graphic3d_SequenceOfHClipPlane>& ClipPlanes() const override {
    std::ostringstream row;
    row << "get " << tag << ' ' << GetRefCount() << ' ' << planes->GetRefCount();
    rows->push_back(row.str());
    if (replaceOnce) {
      replaceOnce = false;
      owner->ReplaceDriverView(*replacement);
    }
    return planes;
  }
  void SetClipPlanes(const occ::handle<Graphic3d_SequenceOfHClipPlane>& incoming) override {
    std::ostringstream row;
    row << "set " << tag << ' ' << GetRefCount() << ' ' << incoming->GetRefCount();
    planes = incoming;
    row << ' ' << incoming->GetRefCount();
    rows->push_back(row.str());
  }
};
class ClipRetargetDriver : public SvDriver {
public:
  ClipRetargetCView* last = nullptr;
  std::vector<std::string> rows;
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    // Driver keeps a raw diagnostic pointer, rather than inventing another native owner.
    last = new ClipRetargetCView(manager, 1, rows);
    return last;
  }
};
static void write_final(const ClipRetargetOwner& owner, const ClipRetargetCView& oldView,
                        const ClipRetargetCView& nextView, const Graphic3d_ClipPlane& plane) {
  std::cout << "final " << (owner.View().get() == &nextView) << ' '
            << (oldView.planes == nextView.planes) << ' ' << oldView.planes->Size() << ' '
            << oldView.GetRefCount() << ' ' << nextView.GetRefCount() << ' '
            << oldView.planes->GetRefCount() << ' ' << plane.GetRefCount() << '\n';
}
static void write_aliases(const ClipRetargetOwner& owner, const ClipRetargetCView& oldView,
                          const occ::handle<Graphic3d_CView>& viewAlias,
                          const occ::handle<Graphic3d_Camera>& cameraAlias) {
  const auto& fresh = owner.Camera();
  std::cout << "aliases " << (&viewAlias == &owner.View()) << ' '
            << (viewAlias == owner.View()) << ' ' << (&cameraAlias != &fresh) << ' '
            << (&cameraAlias == &oldView.Camera()) << ' ' << cameraAlias->GetRefCount()
            << ' ' << fresh->GetRefCount() << '\n';
}
static void capture_operation(const occ::handle<ClipRetargetOwner>& owner,
                              const occ::handle<ClipRetargetCView>& oldView,
                              const occ::handle<ClipRetargetCView>& nextView,
                              const occ::handle<Graphic3d_ClipPlane>& plane,
                              const std::vector<std::string>& rows, bool append) {
  const auto& viewAlias = owner->View();
  const auto& cameraAlias = owner->Camera();
  std::cout << (append ? "add" : "remove") << '\n';
  if (append) owner->AddClipPlane(plane);
  else owner->RemoveClipPlane(plane);
  for (const auto& row : rows) std::cout << row << '\n';
  write_final(*owner, *oldView, *nextView, *plane);
  write_aliases(*owner, *oldView, viewAlias, cameraAlias);
}
static void capture_driver_retarget() {
  occ::handle<ClipRetargetDriver> driver = new ClipRetargetDriver;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  occ::handle<ClipRetargetOwner> owner = new ClipRetargetOwner(viewer);
  occ::handle<ClipRetargetCView> oldView = driver->last;
  occ::handle<ClipRetargetCView> nextView = new ClipRetargetCView(viewer->StructureManager(), 2, driver->rows);
  const occ::handle<Graphic3d_CView> replacement = nextView;
  occ::handle<Graphic3d_ClipPlane> plane = new Graphic3d_ClipPlane;
  oldView->owner = owner.get();
  oldView->replacement = &replacement;
  oldView->replaceOnce = true;
  driver->rows.clear();
  capture_operation(owner, oldView, nextView, plane, driver->rows, true);
  owner->ReplaceDriverView(oldView);
  oldView->replaceOnce = true;
  driver->rows.clear();
  capture_operation(owner, oldView, nextView, plane, driver->rows, false);
}
int main() {
  capture_driver_retarget();
  return 0;
}
