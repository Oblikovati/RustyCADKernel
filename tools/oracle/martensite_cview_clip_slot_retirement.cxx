// Actual OpenGl_View const-handle field assignment: no narrowed adapter or fake counter.
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; original/replacement anchors exclude UB.
#include <Aspect_DisplayConnection.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <iostream>
#include <sstream>
#include <vector>
class CViewClipRetiringSequence : public Graphic3d_SequenceOfHClipPlane {
public:
  Graphic3d_CView* view;
  occ::handle<Graphic3d_SequenceOfHClipPlane>* caller;
  const occ::handle<Graphic3d_SequenceOfHClipPlane>* replacement;
  const occ::handle<Graphic3d_SequenceOfHClipPlane>* alias;
  std::vector<std::string>* rows;
  CViewClipRetiringSequence(Graphic3d_CView& target,
                            occ::handle<Graphic3d_SequenceOfHClipPlane>& source,
                            const occ::handle<Graphic3d_SequenceOfHClipPlane>& next,
                            const occ::handle<Graphic3d_SequenceOfHClipPlane>& retained,
                            std::vector<std::string>& trace)
      : view(&target), caller(&source), replacement(&next), alias(&retained), rows(&trace) {}
  ~CViewClipRetiringSequence() override {
    std::ostringstream row;
    const auto& fresh = view->ClipPlanes();
    row << "retire " << (fresh.get() == this) << ' ' << GetRefCount() << ' '
        << (&fresh == alias) << ' ' << (*caller)->GetRefCount();
    rows->push_back(row.str());
    *caller = *replacement;
  }
};
static void write_slot_state(const char* label, const Graphic3d_CView& view,
                             const occ::handle<Graphic3d_SequenceOfHClipPlane>& alias,
                             const occ::handle<Graphic3d_SequenceOfHClipPlane>& original,
                             const occ::handle<Graphic3d_SequenceOfHClipPlane>& replacement) {
  std::cout << label << ' ' << (&alias == &view.ClipPlanes()) << ' '
            << (alias == original) << ' ' << original->GetRefCount() << ' '
            << replacement->GetRefCount() << '\n';
}
static void capture_null_restore(Graphic3d_CView& view,
                                 const occ::handle<Graphic3d_SequenceOfHClipPlane>& firstAlias,
                                 const occ::handle<Graphic3d_SequenceOfHClipPlane>& caller,
                                 const occ::handle<Graphic3d_SequenceOfHClipPlane>& original,
                                 const occ::handle<Graphic3d_SequenceOfHClipPlane>& replacement) {
  view.SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  std::cout << "null " << firstAlias.IsNull() << ' ' << original->GetRefCount() << ' '
            << replacement->GetRefCount() << '\n';
  view.SetClipPlanes(caller);
  std::cout << "restore " << (firstAlias == replacement) << ' '
            << replacement->GetRefCount() << ' ' << original->GetRefCount() << '\n';
}
static void capture_clip_assignment(Graphic3d_CView& view) {
  occ::handle<Graphic3d_SequenceOfHClipPlane> original = new Graphic3d_SequenceOfHClipPlane;
  occ::handle<Graphic3d_SequenceOfHClipPlane> replacement = new Graphic3d_SequenceOfHClipPlane;
  occ::handle<Graphic3d_SequenceOfHClipPlane> caller = original;
  const auto& firstAlias = view.ClipPlanes();
  const auto& secondAlias = view.ClipPlanes();
  std::vector<std::string> rows;
  view.SetClipPlanes(new CViewClipRetiringSequence(view, caller, replacement, firstAlias, rows));
  view.SetClipPlanes(caller);
  for (const auto& row : rows) std::cout << row << '\n';
  write_slot_state("assigned", view, firstAlias, original, replacement);
  view.SetClipPlanes(secondAlias);
  write_slot_state("self", view, firstAlias, original, replacement);
  capture_null_restore(view, firstAlias, caller, original, replacement);
}
int main() {
  occ::handle<OpenGl_GraphicDriver> driver = new OpenGl_GraphicDriver(new Aspect_DisplayConnection, false);
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  occ::handle<V3d_View> view = viewer->CreateView();
  capture_clip_assignment(*view->View());
  return 0;
}
