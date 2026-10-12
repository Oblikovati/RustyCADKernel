// Original const handle references are reread after getters at OCCT 3d097a0, AIS_ColoredShape.cxx75–101.
#include "martensite_colored_appearance_state.hxx"
#include <stdexcept>
class ColoredCopySourceProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  occ::handle<AIS_Shape>* slot = nullptr;
  const occ::handle<AIS_Shape>* incoming = nullptr;
  int stage = 0, failure = 0;
  void replace(int callback) const
  {
    if (stage != callback) return;
    std::cout << "getter-before " << callback << ' ' << GetRefCount()
      << ' ' << (*incoming)->GetRefCount() << ' ' << Shape().TShape()->GetRefCount() << '\n';
    *slot = *incoming;
    std::cout << "getter-after " << callback << ' ' << GetRefCount()
      << ' ' << (*incoming)->GetRefCount() << ' ' << Shape().TShape()->GetRefCount() << '\n';
    if (failure == 1) throw Standard_Failure("source failure=1; expected successful getter");
    if (failure == 2) throw std::runtime_error("source failure=2; expected successful getter");
  }
  Graphic3d_NameOfMaterial Material() const override
  {
    const auto material = AIS_ColoredShape::Material();
    replace(3);
    return material;
  }
  void Color(Quantity_Color& color) const override
  {
    AIS_ColoredShape::Color(color);
    replace(4);
  }
  double Transparency() const override
  {
    const double transparency = AIS_ColoredShape::Transparency();
    replace(5);
    return transparency;
  }
};
static void copy_source_appearance(const occ::handle<AIS_Shape>& object, bool incoming)
{
  object->Attributes()->SetupOwnDefaults();
  object->SetMaterial(Graphic3d_MaterialAspect(incoming ? Graphic3d_NameOfMaterial_Silver : Graphic3d_NameOfMaterial_Gold));
  object->SetColor(Quantity_Color(incoming ? Quantity_NOC_BLUE : Quantity_NOC_RED));
  object->SetWidth(incoming ? 5.0 : 2.5);
  object->SetTransparency(incoming ? 0.7 : 0.4);
}
static void copy_source_result(const char* label, const occ::handle<AIS_ColoredShape>& copy,
  const ColoredExpansionInputs& input)
{
  std::cout << label << ' ' << copy->Shape().IsEqual(input.base) << ' ' << copy->Shape().IsEqual(input.external)
    << ' ' << copy->HasMaterial() << ' ' << copy->HasColor() << ' ' << copy->HasWidth()
    << ' ' << appearance_number(copy->Width()) << ' ' << appearance_number(copy->Transparency());
  Quantity_Color color; copy->Color(color); appearance_color(color);
  std::cout << ' ' << int(copy->Material()) << ' ' << copy->CustomAspectsMap().Extent() << '\n';
}
