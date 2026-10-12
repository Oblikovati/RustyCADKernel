// Real finite planar faces and their geometric boundary edges; topology routing proof only.
#include <AIS_ColoredShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_CompSolid.hxx>
#include <gp_Pln.hxx>
#include <gp_Ax3.hxx>
#include <iostream>
class ColoredExpansionProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::bindSubShapes;
  using AIS_ColoredShape::fillSubshapeDrawerMap;
  using AIS_ColoredShape::isShapeEntirelyVisible;
};
using ExpansionMap = NCollection_DataMap<TopoDS_Shape,occ::handle<AIS_ColoredDrawer>,TopTools_ShapeMapHasher>;
struct ColoredExpansionInputs
{
  TopoDS_Shape faces[2], edges[2], vertex;
  TopoDS_Compound base, own, external;
  TopoDS_Wire wire;
  TopoDS_Shell shell;
  TopoDS_Solid solid;
  TopoDS_CompSolid compsolid;
  ColoredExpansionInputs()
  {
    faces[0] = BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),-2,3,-1,4).Shape();
    faces[1] = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,2),gp_Dir(0,0,1)),-2,3,-1,4).Shape();
    for (int i=0;i<2;++i) edges[i]=TopExp_Explorer(faces[i],TopAbs_EDGE).Current();
    vertex = TopExp_Explorer(edges[0],TopAbs_VERTEX).Current();
    build_aggregates();
    build_compounds();
  }
  void build_aggregates()
  {
    BRep_Builder builder;
    builder.MakeWire(wire); builder.Add(wire,edges[0]); builder.Add(wire,edges[1]);
    builder.MakeShell(shell); builder.Add(shell,faces[0]); builder.Add(shell,faces[1]);
    builder.MakeSolid(solid); builder.Add(solid,shell);
    builder.MakeCompSolid(compsolid); builder.Add(compsolid,solid);
  }
  void build_compounds()
  {
    BRep_Builder builder;
    builder.MakeCompound(own); builder.Add(own,wire); builder.Add(own,shell);
    builder.MakeCompound(base); builder.Add(base,own);
    builder.MakeCompound(external); builder.Add(external,wire); builder.Add(external,faces[0]);
  }
};
static int expansion_style(const ExpansionMap& map,const TopoDS_Shape& key,
  const occ::handle<AIS_ColoredDrawer>& first,const occ::handle<AIS_ColoredDrawer>& second)
{
  occ::handle<AIS_ColoredDrawer> value;
  if (!map.Find(key,value)) return 0;
  if (value==first) return 1;
  if (value==second) return 2;
  return 3;
}
static void expansion_row(const char* label,const ExpansionMap& map,
  const ColoredExpansionInputs& input,const occ::handle<AIS_ColoredDrawer>& first,
  const occ::handle<AIS_ColoredDrawer>& second)
{
  std::cout<<label<<' '<<map.Extent();
  for (const auto& key:{input.faces[0],input.faces[1],input.edges[0],input.edges[1],input.vertex,
      TopoDS_Shape(input.own),TopoDS_Shape(input.base),TopoDS_Shape(input.external)})
    std::cout<<' '<<expansion_style(map,key,first,second);
  std::cout<<'\n';
}
