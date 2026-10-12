#include "martensite_colored_appearance_state.hxx"
#include <limits>
static void custom_appearance_cases(const ColoredExpansionInputs& input)
{
  const auto object = appearance_object(input);
  const TopoDS_Shape empty;
  object->SetCustomColor(empty, Quantity_Color(Quantity_NOC_RED));
  object->SetCustomWidth(empty, -2); object->SetCustomTransparency(empty, 2);
  appearance_parent("null",object);
  object->SetCustomColor(input.edges[0],Quantity_Color(Quantity_NOC_RED));
  appearance_drawer("custom-color",object->CustomAspects(input.edges[0]));
  try { object->SetCustomWidth(input.edges[0],-2); }
  catch (const Standard_Failure& failure) { std::cout << "custom-width-failure " << failure.ExceptionType() << '\n'; }
  appearance_drawer("after-width-failure",object->CustomAspects(input.edges[0]));
  object->SetCustomWidth(input.edges[0],2.5);
  try { object->SetCustomTransparency(input.edges[0],2); }
  catch (const Standard_Failure& failure) { std::cout << "custom-transparency-failure " << failure.ExceptionType() << '\n'; }
  appearance_drawer("after-transparency-failure",object->CustomAspects(input.edges[0]));
  object->SetCustomTransparency(input.edges[0],0.4);
  appearance_drawer("custom-repaired",object->CustomAspects(input.edges[0]));
  appearance_parent("custom-parent",object);
}
static void global_appearance_cases(const ColoredExpansionInputs& input)
{
  const auto object = appearance_object(input);
  const auto shared = object->CustomAspects(input.edges[0]);
  const auto explicitStyle = object->CustomAspects(input.edges[1]);
  shared->SetupOwnShadingAspect(); shared->SetOwnLineAspects(); shared->SetupOwnPointAspect();
  object->SetCustomColor(input.edges[1],Quantity_Color(Quantity_NOC_RED));
  object->SetCustomWidth(input.edges[1],2.5); object->SetCustomTransparency(input.edges[1],0.4);
  object->SetColor(Quantity_Color(Quantity_NOC_GREEN)); object->SetWidth(4);
  object->SetTransparency(0.7); object->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  appearance_drawer("global-shared",shared); appearance_drawer("global-explicit",explicitStyle);
  appearance_parent("global-parent",object);
  object->UnsetWidth(); object->UnsetTransparency();
  appearance_drawer("unset-shared",shared); appearance_drawer("unset-explicit",explicitStyle);
  appearance_parent("unset-parent",object);
  explicitStyle->SetOwnMaterial(); object->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Silver));
  appearance_drawer("material-shared",shared); appearance_drawer("material-owned",explicitStyle);
}
static void global_failure_cases(const ColoredExpansionInputs& input)
{
  const auto object = appearance_object(input);
  const auto child = object->CustomAspects(input.edges[0]); child->SetOwnLineAspects();
  try { object->SetWidth(-2); }
  catch (const Standard_Failure& failure) { std::cout << "global-width-failure " << failure.ExceptionType() << '\n'; }
  appearance_parent("before-parent-width",object); appearance_drawer("width-child",child);
  object->SetWidth(3); child->SetupOwnShadingAspect();
  try { object->SetTransparency(2); }
  catch (const Standard_Failure& failure) { std::cout << "global-transparency-failure " << failure.ExceptionType() << '\n'; }
  appearance_parent("before-parent-transparency",object);
  object->SetTransparency(std::numeric_limits<double>::quiet_NaN());
  appearance_drawer("nan-child",child); appearance_parent("nan-parent",object);
  object->SetTransparency(0.25); appearance_drawer("global-repaired",child);
}
int main()
{
  const ColoredExpansionInputs input;
  custom_appearance_cases(input);
  global_appearance_cases(input);
  global_failure_cases(input);
}
