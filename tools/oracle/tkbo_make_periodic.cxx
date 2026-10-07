// Native oracle for BOPAlgo_MakePeriodic: a stepped profile made XYZ-periodic, a box trimmed
// to shifted periods, the twins and history of the faces, repetitions and their clearing.
#include <BOPAlgo_MakePeriodic.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopExp_Explorer.hxx>
#include "tkbo_shape_dump.hxx"
#include <algorithm>
#include <utility>
#include <vector>

static TopoDS_Shape profile()
{
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0, 0, 0));
  polygon.Add(gp_Pnt(10, 0, 0));
  polygon.Add(gp_Pnt(10, 0, 10));
  polygon.Add(gp_Pnt(5, 0, 10));
  polygon.Add(gp_Pnt(5, 0, 5));
  polygon.Add(gp_Pnt(0, 0, 5));
  polygon.Close();
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(polygon.Wire());
  return BRepPrimAPI_MakePrism(face, gp_Vec(0, 5, 0)).Shape();
}

// The lists are printed sorted by result index: BRepTools_History::Merge appends the
// images collected in an NCollection_Map, whose bucket order follows the hashes of the
// TShape addresses, so the native list order is an allocation artefact, not a result.
// The twins print no orientation: a twin group keeps the first oriented copy met in the
// same address order, so the orientation varies between two native runs.
static void list(const char* tag, const NCollection_List<TopoDS_Shape>& shapes, const ShapeIndex& all, bool oriented)
{
  std::vector<std::pair<int, char>> items;
  for (NCollection_List<TopoDS_Shape>::Iterator it(shapes); it.More(); it.Next())
    items.emplace_back(all.FindIndex(it.Value()), oriented ? o(it.Value().Orientation()) : '-');
  std::sort(items.begin(), items.end());
  std::cout << tag << ' ' << shapes.Extent() << ':';
  for (const auto& item : items)
    std::cout << ' ' << item.first << item.second;
  std::cout << '\n';
}

static void faces(const char* label, BOPAlgo_MakePeriodic& maker, const TopoDS_Shape& input, const TopoDS_Shape& result)
{
  ShapeIndex all;
  TopExp::MapShapes(result, all);
  std::cout << "faces " << label << '\n';
  for (TopExp_Explorer it(input, TopAbs_FACE); it.More(); it.Next())
  {
    list("modified", maker.History()->Modified(it.Current()), all, true);
    list("generated", maker.History()->Generated(it.Current()), all, true);
  }
  for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next())
    list("twins", maker.GetTwins(it.Current()), all, false);
  // UpdateTwins walks the twin DataMap in address order and skips a group met through a
  // done member, so the repeated edge twins change between two native runs.
  if (std::strcmp(label, "repeated") == 0)
    return;
  for (TopExp_Explorer it(result, TopAbs_EDGE); it.More(); it.Next())
    list("edge-twins", maker.GetTwins(it.Current()), all, false);
}

static void run(const char* label, BOPAlgo_MakePeriodic& maker, const TopoDS_Shape& input)
{
  maker.SetShape(input);
  maker.SetRunParallel(false);
  maker.Perform();
  std::cout << "case " << label << " errors " << maker.HasErrors() << " warnings " << maker.HasWarnings()
            << '\n';
  if (maker.HasErrors())
    return;
  ShapeIndex all;
  dumpResult(maker.Shape(), all);
  faces("periodic", maker, input, maker.Shape());
}

int main()
{
  const TopoDS_Shape shape = profile();
  BOPAlgo_MakePeriodic maker;
  maker.MakeXPeriodic(true, 10.0);
  maker.MakeZPeriodic(true, 10.0);
  maker.MakeYPeriodic(true, 5.0);
  run("profile", maker, shape);
  maker.XRepeat(1);
  maker.YRepeat(2);
  std::cout << "repeated warnings " << maker.HasWarnings() << '\n';
  ShapeIndex all;
  dumpResult(maker.RepeatedShape(), all);
  faces("repeated", maker, shape, maker.RepeatedShape());
  maker.ClearRepetitions();
  std::cout << "cleared null " << maker.RepeatedShape().IsNull() << '\n';
  faces("cleared", maker, shape, maker.Shape());
  maker.ZRepeat(-1);
  ShapeIndex down;
  dumpResult(maker.RepeatedShape(), down);

  BOPAlgo_MakePeriodic trimmed;
  trimmed.MakeXPeriodic(true, 5.0);
  trimmed.MakeYPeriodic(true, 8.0);
  trimmed.MakeZPeriodic(true, 12.0);
  trimmed.SetTrimmed(0, false, 2.0);
  trimmed.SetTrimmed(1, false, 1.0);
  trimmed.SetTrimmed(2, false, -1.0);
  run("trimmed-box", trimmed, BRepPrimAPI_MakeBox(10., 10., 10.).Shape());
  trimmed.XRepeat(2);
  ShapeIndex trimmedAll;
  dumpResult(trimmed.RepeatedShape(), trimmedAll);

  BOPAlgo_MakePeriodic none;
  run("no-periodicity", none, BRepPrimAPI_MakeBox(1., 1., 1.).Shape());
  std::cout << "direction " << BOPAlgo_MakePeriodic::ToDirectionID(-4) << ' '
            << BOPAlgo_MakePeriodic::ToDirectionID(5) << '\n';
  return 0;
}
