// Native oracle for BRepAlgoAPI_BuilderAlgo::SimplifyResult: a fuse of coplanar boxes and a
// section through a sewn shell, before and after the simplification, with the history of
// the arguments and the section edges.
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include "tkbo_shape_dump.hxx"
#include <algorithm>
#include <vector>

// Sorted by index: the merged history images follow TShape-address order.
static void list(const char* tag, const NCollection_List<TopoDS_Shape>& shapes, const ShapeIndex& all)
{
  std::vector<int> items;
  for (NCollection_List<TopoDS_Shape>::Iterator it(shapes); it.More(); it.Next())
    items.push_back(all.FindIndex(it.Value()));
  std::sort(items.begin(), items.end());
  std::cout << ' ' << tag << ' ' << shapes.Extent() << ':';
  for (int item : items)
    std::cout << ' ' << item;
}

static void dump(const char* label, BRepAlgoAPI_BooleanOperation& algo, const TopoDS_Shape& input)
{
  std::cout << "step " << label << " errors " << algo.HasErrors() << '\n';
  ShapeIndex all;
  dumpResult(algo.Shape(), all);
  for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next())
  {
    std::cout << "face deleted " << algo.IsDeleted(it.Current());
    list("modified", algo.Modified(it.Current()), all);
    list("generated", algo.Generated(it.Current()), all);
    std::cout << '\n';
  }
  list("section-edges", algo.SectionEdges(), all);
  std::cout << '\n';
}

static TopoDS_Shape planeFace(const gp_Pln& plane, double u1, double u2, double v1, double v2)
{
  return BRepBuilderAPI_MakeFace(plane, u1, u2, v1, v2).Face();
}

int main()
{
  const TopoDS_Shape box1 = BRepPrimAPI_MakeBox(10., 10., 15.).Shape();
  const TopoDS_Shape box2 = BRepPrimAPI_MakeBox(gp_Pnt(3, 7, 0), 10., 10., 15.).Shape();
  BRepAlgoAPI_Fuse fuse(box1, box2);
  dump("fuse", fuse, box1);
  fuse.SimplifyResult(true, true);
  dump("fuse-simplified", fuse, box1);
  fuse.SimplifyResult(false, false);
  dump("fuse-no-unify", fuse, box1);

  const gp_Pln plane(gp_Pnt(0, 0, 0), gp::DZ());
  BRepBuilderAPI_Sewing sewing;
  sewing.Add(planeFace(plane, -10., 10., -10., 10.));
  sewing.Add(planeFace(plane, 10., 30., -10., 10.));
  sewing.Add(planeFace(plane, 30., 50., -10., 10.));
  sewing.Perform();
  const TopoDS_Shape shell = sewing.SewedShape();
  const TopoDS_Shape cutting = planeFace(gp_Pln(gp_Pnt(0, 5, 0), gp::DY()), -100., 100., -100., 100.);
  BRepAlgoAPI_Section section(shell, cutting);
  dump("section", section, shell);
  section.SimplifyResult(true, false);
  dump("section-simplified", section, shell);
  return 0;
}
