// Martensite actual topology priority matrix, pinned native inline overload.
#include "martensite_colored_expansion_inputs.hxx"
#include <StdSelect_BRepSelectionTool.hxx>
int main()
{
  ColoredExpansionInputs input;
  const TopoDS_Shape shapes[]={input.base,input.compsolid,input.solid,input.shell,input.faces[0],input.wire,input.edges[0],input.vertex};
  for(int index=0;index<8;++index)
  {
    std::cout<<"priority "<<index;
    for(int kind=0;kind<9;++kind)
      std::cout<<' '<<StdSelect_BRepSelectionTool::GetStandardPriority(shapes[index],static_cast<TopAbs_ShapeEnum>(kind));
    std::cout<<'\n';
  }
  for(auto kind:{TopAbs_VERTEX,TopAbs_EDGE,TopAbs_WIRE,TopAbs_FACE})
    std::cout<<"null-priority "<<kind<<' '<<StdSelect_BRepSelectionTool::GetStandardPriority(TopoDS_Shape(),kind)<<'\n';
}
