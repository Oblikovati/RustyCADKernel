// Native oracle for BOPAlgo_BOP: common, fuse, cut and cut21 of solid pairs, with the
// result topology, points and tolerances and the history of the arguments.
#include <BOPAlgo_BOP.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <gp_Ax2.hxx>
#include "tkbo_shape_dump.hxx"

static void history(BOPAlgo_BOP& bop, const TopoDS_Shape& argument, const ShapeIndex& all)
{
  ShapeIndex faces;
  TopExp::MapShapes(argument, TopAbs_FACE, faces);
  for (int i = 1; i <= faces.Extent(); ++i)
  {
    std::cout << "h " << i << " m";
    for (const TopoDS_Shape& m : bop.Modified(faces(i)))
      std::cout << ' ' << all.FindIndex(m) << o(m.Orientation());
    std::cout << " g " << bop.Generated(faces(i)).Extent() << " d " << bop.IsDeleted(faces(i))
              << '\n';
  }
}

static void run(const char* label, const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  const BOPAlgo_Operation operations[4] = {BOPAlgo_COMMON, BOPAlgo_FUSE, BOPAlgo_CUT, BOPAlgo_CUT21};
  for (BOPAlgo_Operation operation : operations)
  {
    BOPAlgo_BOP bop;
    bop.AddArgument(a);
    bop.AddTool(b);
    bop.SetOperation(operation);
    bop.Perform();
    std::cout << "case " << label << ' ' << operation << " errors " << bop.HasErrors()
              << " warnings " << bop.HasWarnings() << '\n';
    ShapeIndex all;
    dumpResult(bop.Shape(), all);
    history(bop, a, all);
    history(bop, b, all);
  }
}

static TopoDS_Shape box(const gp_Pnt& origin, double size)
{
  return BRepPrimAPI_MakeBox(origin, size, size, size).Shape();
}

int main()
{
  TopoDS_Shape unit = box(gp_Pnt(0, 0, 0), 10.);
  run("box-box", unit, box(gp_Pnt(5, 5, 5), 10.));
  run("box-box-face", unit, BRepPrimAPI_MakeBox(gp_Pnt(10, 2, 2), 5., 5., 5.).Shape());
  run("box-cylinder",
      unit,
      BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 5, -5), gp_Dir(0, 0, 1)), 2., 20.).Shape());
  run("box-sphere", unit, BRepPrimAPI_MakeSphere(gp_Pnt(10, 10, 10), 4.).Shape());
  run("box-inner", unit, box(gp_Pnt(3, 3, 3), 2.));
  return 0;
}
