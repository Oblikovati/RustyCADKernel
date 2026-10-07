// Native oracle for BOPAlgo_Tools::EdgesToWires and WiresToFaces: box and cylinder edges
// (shared and intersected), parallel circles with lines, a planar B-spline, a single edge and
// no edge; the wires and the faces made of them are dumped.
#include <BOPAlgo_Tools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Builder.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include "tkbo_shape_dump.hxx"

static TopoDS_Shape edgesOf(const TopoDS_Shape& shape)
{
  BRep_Builder    builder;
  TopoDS_Compound edges;
  builder.MakeCompound(edges);
  for (TopExp_Explorer it(shape, TopAbs_EDGE); it.More(); it.Next())
    builder.Add(edges, it.Current());
  return edges;
}

static void run(const char* label, const TopoDS_Shape& edges, bool shared)
{
  TopoDS_Shape wires;
  const int    error = BOPAlgo_Tools::EdgesToWires(edges, wires, shared);
  std::cout << "case " << label << " shared " << shared << " error " << error << " null "
            << wires.IsNull() << '\n';
  if (wires.IsNull())
    return;
  int count = 0;
  for (TopoDS_Iterator it(wires); it.More(); it.Next())
  {
    int edgesInWire = 0;
    for (TopoDS_Iterator e(it.Value()); e.More(); e.Next())
      ++edgesInWire;
    std::cout << "wire " << count++ << " edges " << edgesInWire << '\n';
  }
  ShapeIndex all;
  dumpResult(wires, all);
  TopoDS_Shape faces;
  const bool   built = BOPAlgo_Tools::WiresToFaces(wires, faces);
  std::cout << "faces built " << built << '\n';
  ShapeIndex faceAll;
  dumpResult(faces, faceAll);
}

int main()
{
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  run("box", edgesOf(box), true);
  run("box-intersected", edgesOf(box), false);
  run("cylinder", edgesOf(BRepPrimAPI_MakeCylinder(5., 10.).Shape()), true);
  BRep_Builder    builder;
  TopoDS_Compound mixed;
  builder.MakeCompound(mixed);
  builder.Add(mixed, BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), gp::DZ()), 3.)).Edge());
  builder.Add(mixed, BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 5), gp::DZ()), 2.)).Edge());
  builder.Add(mixed, BRepBuilderAPI_MakeEdge(gp_Pnt(-5, 0, 0), gp_Pnt(5, 0, 0)).Edge());
  builder.Add(mixed, BRepBuilderAPI_MakeEdge(gp_Pnt(0, -5, 5), gp_Pnt(0, 5, 5)).Edge());
  run("circles-lines", mixed, false);
  NCollection_Array1<gp_Pnt> poles(1, 4);
  poles(1) = gp_Pnt(0, 0, 0);
  poles(2) = gp_Pnt(1, 2, 0);
  poles(3) = gp_Pnt(3, 2, 0);
  poles(4) = gp_Pnt(4, 0, 0);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.;
  knots(2) = 1.;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 4;
  mults(2) = 4;
  TopoDS_Compound spline;
  builder.MakeCompound(spline);
  builder.Add(spline, BRepBuilderAPI_MakeEdge(new Geom_BSplineCurve(poles, knots, mults, 3)).Edge());
  builder.Add(spline, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(4, 0, 0)).Edge());
  run("spline-chord", spline, false);
  run("single", BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)).Edge(), true);
  TopoDS_Compound empty;
  builder.MakeCompound(empty);
  run("empty", empty, true);
  return 0;
}
