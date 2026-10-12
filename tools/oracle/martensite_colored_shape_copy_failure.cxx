// Real source getter overrides throw during native construction; ownership must repair on unwind.
#include "martensite_colored_appearance_state.hxx"
class ColoredCopyGetterProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  mutable int failure=0;
  void Color(Quantity_Color& color) const override
  {
    if (failure==4) throw Standard_Failure("copy color stage=4; expected successful getter");
    AIS_ColoredShape::Color(color);
  }
  Graphic3d_NameOfMaterial Material() const override
  {
    if (failure==3) throw Standard_Failure("copy material stage=3; expected successful getter");
    return AIS_ColoredShape::Material();
  }
  double Transparency() const override
  {
    if (failure==5) throw Standard_Failure("copy transparency stage=5; expected successful getter");
    return AIS_ColoredShape::Transparency();
  }
};
static void copy_getter_case(const occ::handle<ColoredCopyGetterProbe>& source,int stage)
{
  const occ::handle<AIS_Shape> parent(source.get());
  source->failure=stage;
  try { const occ::handle<AIS_ColoredShape> copy = new AIS_ColoredShape(parent); }
  catch (const Standard_Failure& error) { std::cout << "copy-failure " << stage << ' ' << error.ExceptionType(); }
  std::cout << ' ' << source->Shape().TShape()->GetRefCount() << ' ' << source->Attributes()->GetRefCount()
    << ' ' << source->GetRefCount() << '\n';
  source->failure=0;
  const occ::handle<AIS_ColoredShape> copy = new AIS_ColoredShape(parent);
  std::cout << "copy-repaired " << stage << ' ' << source->Shape().TShape()->GetRefCount()
    << ' ' << copy->CustomAspectsMap().Extent() << ' ' << copy->HasColor() << '\n';
}
int main()
{
  const ColoredExpansionInputs input;
  const occ::handle<ColoredCopyGetterProbe> source = new ColoredCopyGetterProbe(input.base);
  source->Attributes()->SetupOwnDefaults();
  source->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  source->SetColor(Quantity_Color(Quantity_NOC_RED)); source->SetTransparency(0.4);
  for (int stage : {3,4,5}) copy_getter_case(source,stage);
}
