// Native source protocols use actual inherited equations/materials and virtual bodies.
// Provenance: OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <Quantity_Color.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>

class ClipProtocolPlane : public Graphic3d_ClipPlane {
  DEFINE_STANDARD_RTTI_INLINE(ClipProtocolPlane, Graphic3d_ClipPlane)
public:
  mutable int cloneCalls = 0;
  mutable int dumpCalls = 0;
  ClipProtocolPlane() : Graphic3d_ClipPlane(Graphic3d_ClipPlane::Equation(1, 2, 3, 4)) {}
  occ::handle<Graphic3d_ClipPlane> Clone() const override {
    ++cloneCalls;
    return Graphic3d_ClipPlane::Clone();
  }
  void DumpJson(Standard_OStream& stream, const int depth = -1) const override {
    ++dumpCalls;
    Graphic3d_ClipPlane::DumpJson(stream, depth);
  }
};
class ClipProtocolSequence : public Graphic3d_SequenceOfHClipPlane {
  DEFINE_STANDARD_RTTI_INLINE(ClipProtocolSequence, Graphic3d_SequenceOfHClipPlane)
public:
  mutable int dumpCalls = 0;
  void DumpJson(Standard_OStream& stream, const int depth = -1) const override {
    ++dumpCalls;
    Graphic3d_SequenceOfHClipPlane::DumpJson(stream, depth);
  }
};
static void write_plane(const char* label, const Graphic3d_ClipPlane& plane) {
  const Graphic3d_ClipPlane::Equation& equation = plane.GetEquation();
  const Quantity_Color& color = plane.CappingColor();
  std::cout << label << ' ' << equation.x() << ' ' << equation.y() << ' '
            << equation.z() << ' ' << equation.w() << ' ' << plane.IsOn() << ' '
            << plane.IsCapping() << ' ' << color.Red() << ' ' << color.Green()
            << ' ' << color.Blue() << ' ' << plane.GetRefCount() << '\n';
}
static void write_material(const char* label, const Graphic3d_ClipPlane& plane) {
  const Quantity_Color& interior = plane.CappingAspect()->InteriorColor();
  const Quantity_Color& diffuse = plane.CappingMaterial().DiffuseColor();
  std::cout << label << ' ' << interior.Red() << ' ' << interior.Green() << ' '
            << interior.Blue() << ' ' << diffuse.Red() << ' ' << diffuse.Green()
            << ' ' << diffuse.Blue() << '\n';
}
static void run_plane() {
  occ::handle<ClipProtocolPlane> plane = new ClipProtocolPlane();
  plane->SetOn(false);
  plane->SetCapping(true);
  plane->SetCappingColor(Quantity_Color(0.2, 0.4, 0.6, Quantity_TOC_RGB));
  occ::handle<Graphic3d_ClipPlane> cloned = plane->Clone();
  std::ostringstream stream;
  plane->DumpJson(stream, -1);
  write_plane("original", *plane);
  write_plane("clone", *cloned);
  write_material("original-material", *plane);
  write_material("clone-material", *cloned);
  std::cout << "dispatch " << plane->cloneCalls << ' ' << plane->dumpCalls << ' '
            << (plane->CappingAspect() != cloned->CappingAspect()) << '\n';
}
static void run_sequence() {
  occ::handle<ClipProtocolSequence> sequence = new ClipProtocolSequence();
  sequence->SetOverrideGlobal(true);
  std::ostringstream stream;
  sequence->DumpJson(stream, -1);
  std::cout << "sequence " << sequence->ToOverrideGlobal() << ' '
            << sequence->dumpCalls << ' ' << sequence->GetRefCount() << '\n';
}
int main() {
  std::cout << std::fixed << std::setprecision(1);
  run_plane();
  run_sequence();
}
