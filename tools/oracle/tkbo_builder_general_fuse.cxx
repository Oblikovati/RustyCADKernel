// Native oracle for the whole BOPAlgo_Builder (General Fuse) on solid pairs: the result
// topology with its points and tolerances, the images and the history of every source shape.
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include "tkbo_shape_dump.hxx"
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Ax2.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

static void run(const char* label, const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  BOPAlgo_Builder builder;
  builder.AddArgument(a);
  builder.AddArgument(b);
  builder.Perform();
  std::cout << "case " << label << " errors " << builder.HasErrors() << " warnings "
            << builder.HasWarnings() << '\n';
  ShapeIndex all;
  dumpResult(builder.Shape(), all);
  const BOPDS_DS& ds = *builder.PDS();
  for (int i = 0; i < ds.NbSourceShapes(); ++i)
  {
    const TopoDS_Shape& s = ds.Shape(i);
    std::cout << "i " << i << ' ' << s.ShapeType();
    const NCollection_List<TopoDS_Shape>* images = builder.Images().Seek(s);
    if (images)
    {
      std::cout << " im";
      for (const TopoDS_Shape& im : *images)
        std::cout << ' ' << all.FindIndex(im) << o(im.Orientation());
    }
    std::cout << " h " << builder.Modified(s).Extent() << ' ' << builder.Generated(s).Extent()
              << ' ' << builder.IsDeleted(s) << '\n';
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
