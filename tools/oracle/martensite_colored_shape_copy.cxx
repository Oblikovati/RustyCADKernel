#include "martensite_colored_appearance_state.hxx"
#include <AIS_TexturedShape.hxx>
static void copy_state(const char* label,const occ::handle<AIS_ColoredShape>& source)
{
  const occ::handle<AIS_Shape> parent(source.get());
  const occ::handle<AIS_ColoredShape> copy = new AIS_ColoredShape(parent);
  Quantity_Color color; copy->Color(color);
  appearance_parent(label,copy); appearance_color(color);
  std::cout << ' ' << int(copy->Material()) << ' ' << copy->Shape().IsEqual(source->Shape())
    << ' ' << (copy->Attributes() == source->Attributes())
    << ' ' << (copy->Attributes()->LineAspect() == copy->Attributes()->FreeBoundaryAspect()) << '\n';
}
static void copy_owned_cases(const ColoredExpansionInputs& input)
{
  const auto object = appearance_object(input);
  copy_state("default-copy",object);
  object->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  object->SetColor(Quantity_Color(Quantity_NOC_RED)); object->SetWidth(2.5);
  object->SetTransparency(0.4); object->SetCustomColor(input.edges[0],Quantity_Color(Quantity_NOC_BLUE));
  copy_state("owned-copy",object);
  object->UnsetWidth(); object->UnsetTransparency(); copy_state("unset-copy",object);
}
static void facing_queries(const char* label, const AIS_Shape& object)
{
  Quantity_Color color(Quantity_NOC_GREEN); object.Color(color);
  std::cout << label; appearance_color(color);
  std::cout << ' ' << int(object.Material()) << ' ' << appearance_number(object.Transparency()) << '\n';
}
static void copy_facing_cases(const ColoredExpansionInputs& input)
{
  const auto object = appearance_object(input);
  object->SetColor(Quantity_Color(Quantity_NOC_RED));
  object->SetCurrentFacingModel(Aspect_TOFM_BACK_SIDE);
  const auto shading = object->Attributes()->ShadingAspect();
  shading->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Silver),Aspect_TOFM_BACK_SIDE);
  shading->SetColor(Quantity_Color(Quantity_NOC_BLUE),Aspect_TOFM_BACK_SIDE);
  shading->SetTransparency(0.001,Aspect_TOFM_BACK_SIDE); copy_state("tiny-back-copy",object);
  facing_queries("tiny-back-queries",*object);
  shading->SetTransparency(0.25,Aspect_TOFM_BACK_SIDE); copy_state("back-copy",object);
  facing_queries("back-queries",*object);
}
static void textured_facing_queries(const ColoredExpansionInputs& input)
{
  const occ::handle<AIS_TexturedShape> object = new AIS_TexturedShape(input.base);
  object->Attributes()->SetupOwnDefaults(); object->SetCurrentFacingModel(Aspect_TOFM_BACK_SIDE);
  const auto shading=object->Attributes()->ShadingAspect();
  shading->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Silver),Aspect_TOFM_BACK_SIDE);
  shading->SetColor(Quantity_Color(Quantity_NOC_BLUE),Aspect_TOFM_BACK_SIDE);
  shading->SetTransparency(0.001,Aspect_TOFM_BACK_SIDE);
  facing_queries("textured-back-queries",*object);
}
int main()
{
  const ColoredExpansionInputs input;
  copy_owned_cases(input);
  copy_facing_cases(input);
  textured_facing_queries(input);
}
