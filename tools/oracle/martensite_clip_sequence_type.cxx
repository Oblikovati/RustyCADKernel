// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: genuine parent alias and generated RTTI.
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <Standard_Transient.hxx>
#include <Standard_Type.hxx>
#include <iostream>
#include <type_traits>
int main() {
  occ::handle<Graphic3d_SequenceOfHClipPlane> sequence = new Graphic3d_SequenceOfHClipPlane;
  const auto& descriptor = Graphic3d_SequenceOfHClipPlane::get_type_descriptor();
  const auto& dynamic = sequence->DynamicType();
  std::cout << "parent " << std::is_same_v<Graphic3d_SequenceOfHClipPlane::base_type, Standard_Transient> << '\n';
  std::cout << "name " << Graphic3d_SequenceOfHClipPlane::get_type_name() << '\n';
  std::cout << "descriptor " << descriptor->Name() << ' ' << descriptor->Parent()->Name() << '\n';
  std::cout << "identity " << (descriptor == dynamic) << '\n';
  std::cout << "kind " << sequence->IsKind(STANDARD_TYPE(Standard_Transient)) << ' '
            << sequence->IsInstance(STANDARD_TYPE(Graphic3d_SequenceOfHClipPlane)) << '\n';
  return 0;
}
