// Native oracle for BOPAlgo_MakeConnected: three glued solids, their materials and origins,
// then made periodic, repeated and cleared; the alerts of the refused inputs.
#include <BOPAlgo_Alerts.hxx>
#include <BOPAlgo_MakeConnected.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopExp_Explorer.hxx>
#include "tkbo_shape_dump.hxx"
#include <algorithm>
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

// The lists are printed sorted by index: the history images come from map iteration in
// TShape-address order (see tkbo_make_periodic.cxx).
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

// Per face of the result: the materials on each side (as argument indices) and the origins
// (as indices among the input sub-shapes).
static void materials(const char* label, BOPAlgo_MakeConnected& maker, const TopoDS_Shape& result,
                      const ShapeIndex& arguments, const ShapeIndex& inputs)
{
  std::cout << "materials " << label << " warnings " << maker.HasWarnings() << '\n';
  ShapeIndex all;
  dumpResult(result, all);
  for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next())
  {
    std::cout << "face " << all.FindIndex(it.Current());
    list("positive", maker.MaterialsOnPositiveSide(it.Current()), arguments);
    list("negative", maker.MaterialsOnNegativeSide(it.Current()), arguments);
    list("origins", maker.GetOrigins(it.Current()), inputs);
    std::cout << '\n';
  }
  for (int i = 1; i <= inputs.Extent(); ++i)
  {
    if (inputs(i).ShapeType() != TopAbs_FACE)
      continue;
    std::cout << "input " << i;
    list("modified", maker.GetModified(inputs(i)), all);
    list("generated", maker.History()->Generated(inputs(i)), all);
    std::cout << '\n';
  }
}

static void refused(const char* label, const NCollection_List<TopoDS_Shape>& arguments)
{
  BOPAlgo_MakeConnected maker;
  maker.SetArguments(arguments);
  maker.SetRunParallel(false);
  maker.Perform();
  std::cout << "refused " << label << " errors " << maker.HasErrors() << " too-few "
            << maker.HasError(STANDARD_TYPE(BOPAlgo_AlertTooFewArguments)) << " multi-dim "
            << maker.HasError(STANDARD_TYPE(BOPAlgo_AlertMultiDimensionalArguments)) << '\n';
}

int main()
{
  const TopoDS_Shape box1 = profile();
  const TopoDS_Shape box2 = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 5), 5, 5, 5).Shape();
  const TopoDS_Shape box3 = BRepPrimAPI_MakeBox(gp_Pnt(0, 5, 0), 10, 5, 10).Shape();
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(box3);
  arguments.Append(box1);
  arguments.Append(box2);
  ShapeIndex argumentIndex, inputs;
  for (NCollection_List<TopoDS_Shape>::Iterator it(arguments); it.More(); it.Next())
  {
    argumentIndex.Add(it.Value());
    TopExp::MapShapes(it.Value(), inputs);
  }

  BOPAlgo_MakeConnected maker;
  maker.SetArguments(arguments);
  maker.SetRunParallel(false);
  maker.RepeatShape(0, 1);
  std::cout << "early repeat warnings " << maker.HasWarnings() << '\n';
  maker.Perform();
  std::cout << "glued errors " << maker.HasErrors() << '\n';
  materials("glued", maker, maker.Shape(), argumentIndex, inputs);

  BOPAlgo_MakePeriodic::PeriodicityParams params;
  params.myPeriodic[0] = params.myPeriodic[1] = params.myPeriodic[2] = true;
  params.myPeriod[0] = params.myPeriod[1] = params.myPeriod[2] = 10.0;
  maker.MakePeriodic(params);
  materials("periodic", maker, maker.PeriodicShape(), argumentIndex, inputs);
  maker.RepeatShape(0, 1);
  maker.RepeatShape(2, -1);
  materials("repeated", maker, maker.PeriodicShape(), argumentIndex, inputs);
  maker.ClearRepetitions();
  materials("cleared", maker, maker.PeriodicShape(), argumentIndex, inputs);

  NCollection_List<TopoDS_Shape> single;
  single.Append(box2);
  BOPAlgo_MakeConnected one;
  one.SetArguments(single);
  one.SetRunParallel(false);
  one.Perform();
  std::cout << "single errors " << one.HasErrors() << " same " << one.Shape().IsSame(box2) << '\n';
  ShapeIndex singleArguments, singleInputs;
  singleArguments.Add(box2);
  TopExp::MapShapes(box2, singleInputs);
  materials("single", one, one.Shape(), singleArguments, singleInputs);
  one.Clear();
  std::cout << "cleared shape null " << one.Shape().IsNull() << " arguments " << one.Arguments().Extent()
            << '\n';

  refused("empty", NCollection_List<TopoDS_Shape>());
  NCollection_List<TopoDS_Shape> mixed;
  mixed.Append(box2);
  TopExp_Explorer face(box3, TopAbs_FACE);
  mixed.Append(face.Current());
  refused("mixed", mixed);
  return 0;
}
