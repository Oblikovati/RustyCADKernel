// Native endpoint classification at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Compound.hxx>
#include <iostream>
#include <Standard_NullObject.hxx>
static TopoDS_Edge edgeAt(double first, double last)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  TopoDS_Vertex a, b;
  builder.MakeEdge(edge);
  builder.MakeVertex(a, gp_Pnt(first,0.,0.), 1.e-7);
  builder.MakeVertex(b, gp_Pnt(last,0.,0.), 1.e-7);
  a.Orientation(TopAbs_FORWARD);
  b.Orientation(TopAbs_REVERSED);
  builder.Add(edge,a);
  builder.Add(edge,b);
  return edge;
}
static void inspect(const char* name, const TopoDS_Shape& shape, double tolerance)
{
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(edgeAt(10.,20.));
  ShapeAnalysis_Wire analyzer;
  analyzer.Load(wire);
  double th=42.,tt=43.,ht=44.,hh=45.;
  const bool found=analyzer.CheckShapeConnect(th,tt,ht,hh,shape,tolerance);
  std::cout << name << " " << found;
  for (ShapeExtend_Status s : {ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,ShapeExtend_DONE5,ShapeExtend_DONE6,ShapeExtend_FAIL1,ShapeExtend_FAIL2})
    std::cout << " " << analyzer.LastCheckStatus(s);
  std::cout << " " << th << " " << tt << " " << ht << " " << hh << " " << analyzer.MinDistance3d() << " " << analyzer.MaxDistance3d() << '\n';
}
static void inspectBadWire()
{
  BRep_Builder builder;
  TopoDS_Wire bad;
  builder.MakeWire(bad);
  builder.Add(bad, edgeAt(20.,30.));
  builder.Add(bad, edgeAt(30.,40.));
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(edgeAt(10.,20.));
  ShapeAnalysis_Wire analyzer;
  analyzer.Load(wire);
  double th=42.,tt=43.,ht=44.,hh=45.;
  try { analyzer.CheckShapeConnect(th,tt,ht,hh,bad,0.); }
  catch (const Standard_NullObject&) {
    std::cout << "disconnected_wire_error 1 " << analyzer.LastCheckStatus(ShapeExtend_DONE1)
              << " " << th << " " << tt << " " << ht << " " << hh << '\n';
  }
}
int main()
{
  std::cout << std::unitbuf;
  inspect("tail_head",edgeAt(20.,30.),0.);
  inspect("tail_tail",edgeAt(30.,20.),0.);
  inspect("head_tail",edgeAt(0.,10.),0.);
  inspect("head_head",edgeAt(10.,0.),0.);
  inspect("equidistant",edgeAt(15.,15.),5.);
  inspect("too_far",edgeAt(0.,1.),0.);
  inspect("null",TopoDS_Shape(),0.);
  BRep_Builder builder;
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  inspect("compound",compound,0.);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire,edgeAt(20.,30.));
  inspect("wire",wire,0.);
  wire.Reverse();
  inspect("reversed_wire",wire,0.);
  inspectBadWire();
}
