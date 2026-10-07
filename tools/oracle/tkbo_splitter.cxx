// Native oracle for BOPAlgo_Splitter and BRepAlgoAPI_Splitter: a box split by a plane face,
// by a box, and two arguments split by a cylinder.
#include <BOPAlgo_Splitter.hxx>
#include <BRepAlgoAPI_Splitter.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <gp_Ax2.hxx>
#include <gp_Pln.hxx>
#include "tkbo_shape_dump.hxx"

static NCollection_List<TopoDS_Shape> list(const TopoDS_Shape& a)
{
  NCollection_List<TopoDS_Shape> shapes;
  shapes.Append(a);
  return shapes;
}

static void run(const char* label,
                const NCollection_List<TopoDS_Shape>& arguments,
                const NCollection_List<TopoDS_Shape>& tools)
{
  BOPAlgo_Splitter splitter;
  splitter.SetArguments(arguments);
  splitter.SetTools(tools);
  splitter.Perform();
  std::cout << "case " << label << " errors " << splitter.HasErrors() << " null "
            << splitter.Shape().IsNull() << '\n';
  if (splitter.Shape().IsNull())
    return;
  std::cout << "type " << splitter.Shape().ShapeType() << '\n';
  ShapeIndex all;
  dumpResult(splitter.Shape(), all);
  BRepAlgoAPI_Splitter api;
  api.SetArguments(arguments);
  api.SetTools(tools);
  api.Build();
  ShapeIndex apiAll;
  std::cout << "api done " << api.IsDone() << '\n';
  dumpResult(api.Shape(), apiAll);
}

int main()
{
  TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  TopoDS_Shape plane =
    BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 4), gp_Dir(0, 0, 1)), -5., 15., -5., 15.).Face();
  run("box-plane", list(box), list(plane));
  run("box-box", list(box), list(BRepPrimAPI_MakeBox(gp_Pnt(5, 5, 5), 10., 10., 10.).Shape()));
  NCollection_List<TopoDS_Shape> two = list(box);
  two.Append(BRepPrimAPI_MakeBox(gp_Pnt(20, 0, 0), 10., 10., 10.).Shape());
  run("two-cylinder",
      two,
      list(BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(15, 5, -5), gp_Dir(1, 0, 0)), 2., 20.).Shape()));
  run("lonely", list(box), NCollection_List<TopoDS_Shape>());
  return 0;
}
