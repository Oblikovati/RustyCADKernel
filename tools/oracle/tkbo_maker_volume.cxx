// Native oracle for BOPAlgo_MakerVolume: the volumes of box faces, of intersecting solids,
// without intersection, with an internal edge (kept or avoided), and without arguments.
#include <BOPAlgo_MakerVolume.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <TopExp_Explorer.hxx>
#include "tkbo_shape_dump.hxx"

static NCollection_List<TopoDS_Shape> faces(const TopoDS_Shape& shape)
{
  NCollection_List<TopoDS_Shape> result;
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next())
    result.Append(it.Current());
  return result;
}

static void run(const char* label, const NCollection_List<TopoDS_Shape>& arguments, bool intersect, bool avoid)
{
  BOPAlgo_MakerVolume maker;
  maker.SetArguments(arguments);
  maker.SetIntersect(intersect);
  maker.SetAvoidInternalShapes(avoid);
  maker.SetRunParallel(false);
  maker.Perform();
  std::cout << "case " << label << " errors " << maker.HasErrors() << " warnings " << maker.HasWarnings()
            << " faces " << maker.Faces().Extent() << " null " << maker.Shape().IsNull() << '\n';
  if (maker.HasErrors() || maker.Shape().IsNull())
    return;
  std::cout << "type " << maker.Shape().ShapeType() << '\n';
  ShapeIndex boxAll;
  dumpResult(maker.Box(), boxAll);
  ShapeIndex all;
  dumpResult(maker.Shape(), all);
}

int main()
{
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  run("box-faces", faces(box), true, false);
  run("box-faces-no-intersect", faces(box), false, false);
  NCollection_List<TopoDS_Shape> solids;
  solids.Append(box);
  solids.Append(BRepPrimAPI_MakeSphere(gp_Pnt(10, 10, 10), 5.).Shape());
  run("box-sphere", solids, true, false);
  NCollection_List<TopoDS_Shape> withEdge = faces(box);
  withEdge.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(2, 2, 2), gp_Pnt(8, 8, 8)).Edge());
  run("internal-edge", withEdge, true, false);
  run("internal-edge-avoided", withEdge, true, true);
  run("nothing", NCollection_List<TopoDS_Shape>(), true, false);
  return 0;
}
