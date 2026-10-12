// Genuine inherited Add/Remove failure ordering with valid old and replacement anchors.
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; ordinary native source oracle only.
#define main clip_retarget_baseline_main
#include "martensite_v3d_clip_view_retarget.cxx"
#undef main
#include <Standard_ProgramError.hxx>
class ClipFailureCView : public ClipRetargetCView {
public:
  int failureStage = 0;
  using ClipRetargetCView::ClipRetargetCView;
  const occ::handle<Graphic3d_SequenceOfHClipPlane>& ClipPlanes() const override {
    const auto& result = ClipRetargetCView::ClipPlanes();
    if (failureStage == 1) throw Standard_ProgramError("clip failure stage=1; expected completion");
    return result;
  }
  void SetClipPlanes(const occ::handle<Graphic3d_SequenceOfHClipPlane>& incoming) override {
    if (failureStage == 2) throw Standard_ProgramError("clip failure stage=2; expected completion");
    ClipRetargetCView::SetClipPlanes(incoming);
    if (failureStage == 3) throw Standard_ProgramError("clip failure stage=3; expected completion");
  }
};
class ClipFailureDriver : public ClipRetargetDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    last = new ClipFailureCView(manager, 1, rows);
    return last;
  }
};
static void write_failure_state(const ClipRetargetOwner& owner, const ClipFailureCView& oldView,
                                const ClipFailureCView& nextView, const Graphic3d_ClipPlane& plane) {
  std::cout << "state " << (owner.View().get() == &nextView) << ' '
            << (oldView.planes == nextView.planes) << ' ' << oldView.planes->Size() << ' '
            << nextView.planes->Size() << ' ' << oldView.GetRefCount() << ' '
            << nextView.GetRefCount() << ' ' << oldView.planes->GetRefCount() << ' '
            << nextView.planes->GetRefCount() << ' ' << plane.GetRefCount() << '\n';
}
static void invoke_failure(ClipRetargetOwner& owner, const occ::handle<Graphic3d_ClipPlane>& plane,
                            const std::vector<std::string>& rows, bool append) {
  try {
    if (append) owner.AddClipPlane(plane);
    else owner.RemoveClipPlane(plane);
    std::cout << "unexpected success\n";
  } catch (const Standard_ProgramError& failure) {
    std::cout << "failure " << failure.what() << '\n';
  }
  for (const auto& row : rows) std::cout << row << '\n';
}
static void configure_failure(ClipFailureCView& oldView, ClipFailureCView& nextView,
                              ClipRetargetOwner& owner,
                              const occ::handle<Graphic3d_CView>& replacement, int stage) {
  oldView.owner = &owner;
  oldView.replacement = &replacement;
  oldView.replaceOnce = true;
  oldView.failureStage = stage == 1 ? 1 : 0;
  nextView.failureStage = stage;
}
static void capture_failure_case(int stage, bool append) {
  occ::handle<ClipFailureDriver> driver = new ClipFailureDriver;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  occ::handle<ClipRetargetOwner> owner = new ClipRetargetOwner(viewer);
  occ::handle<ClipFailureCView> oldView = static_cast<ClipFailureCView*>(driver->last);
  occ::handle<ClipFailureCView> nextView = new ClipFailureCView(viewer->StructureManager(), 2, driver->rows);
  const occ::handle<Graphic3d_CView> replacement = nextView;
  occ::handle<Graphic3d_ClipPlane> plane = new Graphic3d_ClipPlane;
  if (!append) owner->AddClipPlane(plane);
  configure_failure(*oldView, *nextView, *owner, replacement, stage);
  driver->rows.clear();
  std::cout << "case " << (append ? "add" : "remove") << ' ' << stage << '\n';
  invoke_failure(*owner, plane, driver->rows, append);
  write_failure_state(*owner, *oldView, *nextView, *plane);
}
int main() {
  for (bool append : {true, false}) {
    for (int stage : {1, 2, 3}) capture_failure_case(stage, append);
  }
  return 0;
}
