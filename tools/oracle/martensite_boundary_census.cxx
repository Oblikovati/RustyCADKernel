// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: actual split vertex identity/parameter oracle.
#include <BOPAlgo_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Iterator.hxx>
#include <iostream>
#include <iomanip>
static NCollection_List<TopoDS_Shape> Sources()
{
  NCollection_Array1<gp_Pnt> poles(1,3);
  poles(1)=gp_Pnt(0,0,0); poles(2)=gp_Pnt(.5,0,0); poles(3)=gp_Pnt(1,1,0);
  occ::handle<Geom_BezierCurve> curve=new Geom_BezierCurve(poles);
  NCollection_List<TopoDS_Shape> sources;
  sources.Append(BRepBuilderAPI_MakeEdge(curve).Edge());
  sources.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(-1,.25,0),gp_Pnt(2,.25,0)).Edge());
  return sources;
}
static bool Contains(const TopoDS_Shape& edge,const TopoDS_Shape& vertex)
{
  for (TopoDS_Iterator it(edge);it.More();it.Next()) {
    if (it.Value().IsSame(vertex)) return true;
  }
  return false;
}
static int Incidents(BOPAlgo_Builder& builder,const NCollection_List<TopoDS_Shape>& sources,const TopoDS_Shape& vertex)
{
  int count=0;
  for (const TopoDS_Shape& source:sources) {
    for (const TopoDS_Shape& split:builder.Modified(source)) count+=Contains(split,vertex);
  }
  return count;
}
static double SplitParameter(BOPAlgo_Builder& builder,const TopoDS_Shape& source,const TopoDS_Shape& vertex)
{
  for (const TopoDS_Shape& split:builder.Modified(source)) {
    if (Contains(split,vertex)) return BRep_Tool::Parameter(TopoDS::Vertex(vertex),TopoDS::Edge(split));
  }
  throw Standard_Failure("generated vertex incidents=0; expected an actual modified edge containing vertex");
}
static void Emit(BOPAlgo_Builder& builder,const NCollection_List<TopoDS_Shape>& sources)
{
  int identity=0;
  for (const TopoDS_Shape& source:sources) {
    ++identity;
    for (const TopoDS_Shape& generated:builder.Generated(source)) {
      if (generated.ShapeType()!=TopAbs_VERTEX) continue;
      const gp_Pnt point=BRep_Tool::Pnt(TopoDS::Vertex(generated));
      const double parameter=SplitParameter(builder,source,generated);
      std::cout<<"endpoint "<<identity<<' '<<parameter<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<' '<<Incidents(builder,sources,generated)<<'\n';
    }
  }
}
int main()
{
  const auto sources=Sources();
  BOPAlgo_Builder builder;
  builder.SetArguments(sources); builder.SetNonDestructive(true); builder.Perform();
  if (builder.HasErrors()) throw Standard_Failure("boundary builder HasErrors=true; expected completed two-source split");
  std::cout<<std::setprecision(17);
  Emit(builder,sources);
  return 0;
}
