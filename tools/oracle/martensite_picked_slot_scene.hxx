// Named rendering I/O double preserves genuine native view/window/subview state.
#include "tkservice_probe_null_driver.cxx"
#include <AIS_InteractiveContext.hxx>
#include <AIS_ViewController.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
class PickedSlotRecordingView : public SvView {
public:
  explicit PickedSlotRecordingView(const occ::handle<Graphic3d_StructureManager>& manager)
    : SvView(manager, {}) {}
  void SetWindow(const occ::handle<Graphic3d_CView>& parent,
      const occ::handle<Aspect_Window>& window, const Aspect_RenderingContext context) override {
    SvView::SetWindow(parent, window, context);
    myParentView = parent.get();
    if (!parent.IsNull()) {
      parent->AddSubview(this);
      SubviewResized(occ::handle<Aspect_NeutralWindow>::DownCast(window));
    }
  }
};
class PickedSlotRecordingDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {
    return new PickedSlotRecordingView(manager);
  }
};
struct PickedSlotScene {
  occ::handle<V3d_Viewer> viewer;
  occ::handle<V3d_View> parent, child;
  occ::handle<AIS_InteractiveContext> context;
};
static PickedSlotScene picked_slot_scene() {
  PickedSlotScene scene;
  scene.viewer = new V3d_Viewer(new PickedSlotRecordingDriver);
  scene.parent = scene.viewer->CreateView(); scene.child = scene.viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow;
  window->SetSize(640,480); scene.parent->SetWindow(window);
  scene.child->SetWindow(scene.parent, NCollection_Vec2<double>(.5, 1.));
  scene.context = new AIS_InteractiveContext(scene.viewer);
  return scene;
}
static void picked_slot_scene_close(PickedSlotScene& scene) {
  scene.context.Nullify();
  scene.child->Remove(); scene.parent->Remove();
  scene.child.Nullify(); scene.parent.Nullify(); scene.viewer.Nullify();
}
