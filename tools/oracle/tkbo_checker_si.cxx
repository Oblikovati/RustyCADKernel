// Native oracle for BOPAlgo_CheckerSI: the errors, the per-kind interference record counts
// and the interference table (in map iteration order) of shapes checked at several levels.
#include <BOPAlgo_CheckerSI.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_NurbsConvert.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Precision.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Ax3.hxx>
#include <iostream>

static TopoDS_Shape box(double x, double y, double z, double size)
{
  return BRepPrimAPI_MakeBox(gp_Pnt(x, y, z), size, size, size).Shape();
}

static TopoDS_Shape compound(const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  BRep_Builder    builder;
  TopoDS_Compound result;
  builder.MakeCompound(result);
  builder.Add(result, a);
  builder.Add(result, b);
  return result;
}

static void run(const char* label, const TopoDS_Shape& shape, int level)
{
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(shape);
  BOPAlgo_CheckerSI checker;
  checker.SetArguments(arguments);
  checker.SetLevelOfCheck(level);
  checker.SetRunParallel(false);
  checker.Perform();
  std::cout << "case " << label << " level " << level << " errors " << checker.HasErrors()
            << " warnings " << checker.HasWarnings() << '\n';
  BOPDS_DS& ds = *checker.PDS();
  std::cout << "records " << ds.InterfVV().Length() << ' ' << ds.InterfVE().Length() << ' '
            << ds.InterfEE().Length() << ' ' << ds.InterfVF().Length() << ' '
            << ds.InterfEF().Length() << ' ' << ds.InterfFF().Length() << ' '
            << ds.InterfVZ().Length() << ' ' << ds.InterfEZ().Length() << ' '
            << ds.InterfFZ().Length() << ' ' << ds.InterfZZ().Length() << '\n';
  std::cout << "sources " << ds.NbSourceShapes() << " shapes " << ds.NbShapes() << '\n';
  for (NCollection_Map<BOPDS_Pair>::Iterator it(ds.Interferences()); it.More(); it.Next())
  {
    int first = -1, second = -1;
    it.Value().Indices(first, second);
    std::cout << "pair " << first << ' ' << second << ' ' << ds.ShapeInfo(first).ShapeType()
              << ' ' << ds.ShapeInfo(second).ShapeType() << '\n';
  }
}

int main()
{
  const TopoDS_Shape single = box(0, 0, 0, 10);
  const TopoDS_Shape overlapping = compound(single, box(5, 5, 5, 10));
  const int levels[] = {0, 2, 3, 4, 5, 9};
  for (int level : levels)
    run("overlapping-boxes", overlapping, level);
  run("single-box", single, 9);
  run("nested-boxes", compound(single, box(2, 2, 2, 4)), 9);
  run("box-vertex",
      compound(single, BRepBuilderAPI_MakeVertex(gp_Pnt(3, 3, 3)).Vertex()),
      9);
  run("box-edge",
      compound(single, BRepBuilderAPI_MakeEdge(gp_Pnt(5, 5, -5), gp_Pnt(5, 5, 15)).Edge()),
      9);
  run("nurbs-box", BRepBuilderAPI_NurbsConvert(single).Shape(), 9);
  const occ::handle<Geom_ToroidalSurface> spindle = new Geom_ToroidalSurface(gp_Ax3(), 2., 3.);
  run("spindle-torus", BRepBuilderAPI_MakeFace(spindle, Precision::Confusion()).Face(), 5);
  NCollection_List<TopoDS_Shape> two;
  two.Append(single);
  two.Append(single);
  BOPAlgo_CheckerSI checker;
  checker.SetArguments(two);
  checker.Perform();
  std::cout << "case two-arguments errors " << checker.HasErrors() << " ds "
            << (checker.PDS() != nullptr) << '\n';
  return 0;
}
