// Martensite actual native protected expansion and visibility at OCCT 3d097a0.
#include "martensite_colored_expansion_inputs.hxx"
static void aggregate_cases(const ColoredExpansionInputs& input)
{
  occ::handle<ColoredExpansionProbe> object = new ColoredExpansionProbe(input.base);
  auto first = object->CustomAspects(input.own);
  auto second = object->CustomAspects(input.external);
  for (const auto& item:{std::make_pair("wire",TopoDS_Shape(input.wire)),
      std::make_pair("shell",TopoDS_Shape(input.shell)),std::make_pair("solid",TopoDS_Shape(input.solid)),
      std::make_pair("compsolid",TopoDS_Shape(input.compsolid))}) {
    ExpansionMap output;
    object->bindSubShapes(output,item.second,first);
    expansion_row(item.first,output,input,first,second);
  }
}
static void explicit_cases(const ColoredExpansionInputs& input)
{
  occ::handle<ColoredExpansionProbe> object = new ColoredExpansionProbe(input.base);
  auto first = object->CustomAspects(input.own);
  auto second = object->CustomAspects(input.external);
  ExpansionMap output;
  output.Bind(input.faces[0],second);
  object->bindSubShapes(output,input.solid,first);
  expansion_row("preserved-leaf",output,input,first,second);
  object->bindSubShapes(output,input.faces[0],first);
  expansion_row("explicit-leaf",output,input,first,second);
  object->bindSubShapes(output,input.own,second);
  object->bindSubShapes(output,input.vertex,first);
  expansion_row("compound-vertex",output,input,first,second);
}
static void external_cases(const ColoredExpansionInputs& input)
{
  occ::handle<ColoredExpansionProbe> object = new ColoredExpansionProbe(input.base);
  auto first = object->CustomAspects(input.external);
  auto second = object->CustomAspects(input.edges[0]);
  ExpansionMap output;
  object->fillSubshapeDrawerMap(output);
  expansion_row("external-explicit",output,input,first,second);
  object->ClearCustomAspects();
  object->ChangeCustomAspectsMap().Bind(input.own,first);
  object->fillSubshapeDrawerMap(output);
  expansion_row("own-compound-append",output,input,first,second);
}
static void base_orientation_cases(const ColoredExpansionInputs& input)
{
  occ::handle<ColoredExpansionProbe> object = new ColoredExpansionProbe(input.base);
  auto first = object->CustomAspects(input.base);
  ExpansionMap output;
  object->fillSubshapeDrawerMap(output);
  expansion_row("same-base",output,input,first,occ::handle<AIS_ColoredDrawer>());
  object->ClearCustomAspects();
  object->ChangeCustomAspectsMap().Bind(input.base.Reversed(),first);
  object->fillSubshapeDrawerMap(output);
  expansion_row("reversed-base",output,input,first,occ::handle<AIS_ColoredDrawer>());
}
static void visibility_cases(const ColoredExpansionInputs& input)
{
  occ::handle<ColoredExpansionProbe> object = new ColoredExpansionProbe(input.base);
  std::cout<<"empty-visible "<<object->isShapeEntirelyVisible()<<'\n';
  auto first = object->CustomAspects(input.external);
  std::cout<<"outside-visible "<<object->isShapeEntirelyVisible()<<'\n';
  first->SetHidden(true);
  std::cout<<"outside-hidden "<<object->isShapeEntirelyVisible()<<'\n';
  object->UnsetCustomAspects(input.external,false);
  std::cout<<"replacement-visible "<<object->isShapeEntirelyVisible()<<'\n';
}
int main()
{
  ColoredExpansionInputs input;
  aggregate_cases(input); explicit_cases(input); external_cases(input);
  base_orientation_cases(input); visibility_cases(input);
  return 0;
}
