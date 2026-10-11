#include <Graphic3d_Structure.hxx>
#include <iostream>
class MartensiteInheritedStructure : public Graphic3d_Structure {};
int main() {
  const auto& base = Graphic3d_Structure::get_type_descriptor();
  const auto& child = MartensiteInheritedStructure::get_type_descriptor();
  std::cout << child->Name() << " " << child->Parent()->Name()
            << " " << (base == child) << "\n";
  return child->Parent()->Name() == std::string("Standard_Transient") && base == child ? 0 : 1;
}
