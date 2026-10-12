// Paired with derived_clip_scene.rs; actual native subclasses preserve inherited rendering fields.
// Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871, local graphical oracle only.
#include <V3d_View.hxx>
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
class RenderClipDerivedPlane : public Graphic3d_ClipPlane {
public:
  RenderClipDerivedPlane() : Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,0)) {}
  DEFINE_STANDARD_RTTIEXT(RenderClipDerivedPlane, Graphic3d_ClipPlane)
};
IMPLEMENT_STANDARD_RTTIEXT(RenderClipDerivedPlane, Graphic3d_ClipPlane)
class RenderClipDerivedSequence : public Graphic3d_SequenceOfHClipPlane {
public:
  DEFINE_STANDARD_RTTIEXT(RenderClipDerivedSequence, Graphic3d_SequenceOfHClipPlane)
};
IMPLEMENT_STANDARD_RTTIEXT(RenderClipDerivedSequence, Graphic3d_SequenceOfHClipPlane)
static void configure_derived_clipping(const occ::handle<V3d_View>& view, const std::string& mode) {
  occ::handle<Graphic3d_ClipPlane> plane = new RenderClipDerivedPlane;
  occ::handle<Graphic3d_SequenceOfHClipPlane> sequence = new RenderClipDerivedSequence;
  sequence->Append(plane);
  view->SetClipPlanes(sequence);
  if (mode == "clip-derived-alias-clear") {
    view->Redraw();
    const auto& alias = view->ClipPlanes();
    alias->Clear();
  }
}
