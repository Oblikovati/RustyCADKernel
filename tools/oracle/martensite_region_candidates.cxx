// SPDX-License-Identifier: GPL-2.0-only
// OCCT3d097a0 fixture preserves mapped generation; sampled native classification is an oracle, not a certificate.
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPAlgo_BuilderFace.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRepLib.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepTopAdaptor_FClass2d.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <string>
class NonemptyGenerator : public BOPAlgo_Builder {
  void CheckData() override
  {
    // Admission only differs; RunArrangement independently rejects the filler report.
  }
};
static NCollection_List<TopoDS_Shape> Cubic(bool rational, bool reversed)
{
  NCollection_Array1<gp_Pnt> poles(1,4);
  poles(1)=gp_Pnt(0,0,0); poles(2)=gp_Pnt(4,0,0);
  poles(3)=gp_Pnt(0,4,0); poles(4)=gp_Pnt(0,0,0);
  NCollection_Array1<double> weights(1,4);
  weights(1)=1; weights(2)=2; weights(3)=2; weights(4)=1;
  occ::handle<Geom_BezierCurve> curve = rational
    ? new Geom_BezierCurve(poles,weights) : new Geom_BezierCurve(poles);
  TopoDS_Shape edge=BRepBuilderAPI_MakeEdge(curve).Edge();
  if (reversed) edge.Reverse();
  NCollection_List<TopoDS_Shape> roots; roots.Append(edge); return roots;
}
static void Rectangle(NCollection_List<TopoDS_Shape>& roots,double x,double y,double size)
{
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(x,y,0)); polygon.Add(gp_Pnt(x+size,y,0));
  polygon.Add(gp_Pnt(x+size,y+size,0)); polygon.Add(gp_Pnt(x,y+size,0));
  polygon.Close();
  for (TopoDS_Iterator it(polygon.Wire());it.More();it.Next()) roots.Append(it.Value());
}
static TopoDS_Shape RunArrangement(const NCollection_List<TopoDS_Shape>& roots)
{
  BRep_Builder constructor; TopoDS_Compound batch; constructor.MakeCompound(batch);
  for (const TopoDS_Shape& root:roots) constructor.Add(batch,root);
  BRepBuilderAPI_Copy copy(batch,true,true); NCollection_List<TopoDS_Shape> arguments;
  for (TopoDS_Iterator it(copy.Shape());it.More();it.Next()) arguments.Append(it.Value());
  BOPAlgo_PaveFiller filler; filler.SetArguments(arguments); filler.SetNonDestructive(true); filler.Perform();
  if (filler.HasErrors()) throw Standard_Failure("fixture filler report");
  NonemptyGenerator builder; builder.SetArguments(arguments); builder.SetNonDestructive(true);
  builder.PerformWithFiller(filler);
  if (builder.HasErrors()) throw Standard_Failure("fixture builder report");
  return builder.Shape();
}
static NCollection_List<TopoDS_Shape> Generate(const NCollection_List<TopoDS_Shape>& roots)
{
  BRepBuilderAPI_Copy copy(RunArrangement(roots),true,true);
  TopoDS_Face plane=BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY())).Face();
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> edges; TopExp::MapShapes(copy.Shape(),TopAbs_EDGE,edges);
  NCollection_List<TopoDS_Shape> inputs;
  for (int i=1;i<=edges.Extent();++i) {
    const TopoDS_Edge edge=TopoDS::Edge(edges(i)); BRepLib::BuildPCurveForEdgeOnPlane(edge,plane);
    inputs.Append(edge.Oriented(TopAbs_FORWARD)); inputs.Append(edge.Oriented(TopAbs_REVERSED));
  }
  BOPAlgo_BuilderFace builder; builder.SetFace(plane); builder.SetShapes(inputs); builder.Perform();
  if (builder.HasErrors()) throw Standard_Failure("fixture candidate report");
  return builder.Areas();
}
static void Capture(const char* label,const NCollection_List<TopoDS_Shape>& areas,double x,double y)
{
  int inside=0,on=0,outside=0,maxWires=0,finiteIn=0,finiteOn=0;
  for (const TopoDS_Shape& shape:areas) {
    NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> wires; TopExp::MapShapes(shape,TopAbs_WIRE,wires);
    maxWires=std::max(maxWires,wires.Extent());
    BRepClass_FaceClassifier classification(TopoDS::Face(shape),gp_Pnt2d(x,y),1e-7);
    const TopAbs_State state=classification.State();
    inside+=state==TopAbs_IN; on+=state==TopAbs_ON; outside+=state==TopAbs_OUT;
    BRepTopAdaptor_FClass2d infinite(TopoDS::Face(shape),1e-7);
    if (infinite.PerformInfinitePoint()==TopAbs_OUT) {finiteIn+=state==TopAbs_IN; finiteOn+=state==TopAbs_ON;}
  }
  std::cout<<label<<' '<<x<<' '<<y<<' '<<areas.Size()<<' '<<maxWires<<' '
    <<inside<<' '<<on<<' '<<outside<<' '<<finiteIn<<' '<<finiteOn<<'\n';
}
static void CaptureCubic(bool rational,bool reversed,const char* label)
{
  const NCollection_List<TopoDS_Shape> areas=Generate(Cubic(rational,reversed));
  Capture(label,areas,1,1); Capture(label,areas,-1,1); Capture(label,areas,0,0);
}
int main()
{
  CaptureCubic(false,false,"cubic"); CaptureCubic(false,true,"reverse");
  CaptureCubic(true,false,"rational"); NCollection_List<TopoDS_Shape> nested;
  Rectangle(nested,0,0,8); Rectangle(nested,2,2,4);
  const NCollection_List<TopoDS_Shape> nestedAreas=Generate(nested);
  Capture("nested",nestedAreas,1,1); Capture("nested",nestedAreas,3,3); Capture("nested",nestedAreas,2,3);
  NCollection_List<TopoDS_Shape> touching; Rectangle(touching,0,0,2); Rectangle(touching,2,2,2);
  const NCollection_List<TopoDS_Shape> touchingAreas=Generate(touching);
  Capture("touching",touchingAreas,1,1); Capture("touching",touchingAreas,2,2);
  return 0;
}
