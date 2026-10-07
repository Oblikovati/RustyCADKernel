// Native oracle for BRepAlgoAPI_Defeaturing / BOPAlgo_RemoveFeatures: a boss, a through
// hole and a pocket removed from boxes, two features at once, a compound with a face, a
// face outside the shape, a feature that cannot be removed, and the history of the input.
#include <BOPAlgo_Alerts.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Defeaturing.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Ax2.hxx>
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

static TopoDS_Shape box(double x, double y, double z, double dx, double dy, double dz)
{
  return BRepPrimAPI_MakeBox(gp_Pnt(x, y, z), dx, dy, dz).Shape();
}

static TopoDS_Shape cylinder(double x, double y, double z, double r, double h)
{
  return BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(x, y, z), gp::DZ()), r, h).Shape();
}

static void run(const char* label, const TopoDS_Shape& input, const NCollection_List<TopoDS_Shape>& faces,
                bool history = true)
{
  BRepAlgoAPI_Defeaturing defeaturing;
  defeaturing.SetShape(input);
  defeaturing.AddFacesToRemove(faces);
  defeaturing.SetToFillHistory(history);
  defeaturing.SetRunParallel(false);
  defeaturing.Build();
  std::cout << "case " << label << " done " << defeaturing.IsDone() << " errors " << defeaturing.HasErrors()
            << " warnings " << defeaturing.HasWarnings() << " no-faces "
            << defeaturing.HasError(STANDARD_TYPE(BOPAlgo_AlertNoFacesToRemove)) << " too-few "
            << defeaturing.HasError(STANDARD_TYPE(BOPAlgo_AlertTooFewArguments)) << " unsupported "
            << defeaturing.HasWarning(STANDARD_TYPE(BOPAlgo_AlertUnsupportedType)) << " unable "
            << defeaturing.HasWarning(STANDARD_TYPE(BOPAlgo_AlertUnableToRemoveTheFeature)) << '\n';
  if (!defeaturing.IsDone())
    return;
  ShapeIndex all;
  dumpResult(defeaturing.Shape(), all);
  std::cout << "history " << defeaturing.HasHistory() << " modified " << defeaturing.HasModified()
            << " generated " << defeaturing.HasGenerated() << " deleted " << defeaturing.HasDeleted() << '\n';
  ShapeIndex inputs;
  TopExp::MapShapes(input, inputs);
  for (int i = 1; i <= inputs.Extent(); ++i)
  {
    const TopAbs_ShapeEnum type = inputs(i).ShapeType();
    if (type != TopAbs_FACE && type != TopAbs_EDGE && type != TopAbs_SOLID)
      continue;
    std::cout << "input " << i << " deleted " << defeaturing.IsDeleted(inputs(i));
    list("modified", defeaturing.Modified(inputs(i)), all);
    list("generated", defeaturing.Generated(inputs(i)), all);
    std::cout << '\n';
  }
}

// The faces of `shape` with an image of a face of `tool` (the faces a Boolean took from it).
static NCollection_List<TopoDS_Shape> faces_from(BRepAlgoAPI_BooleanOperation& op, const TopoDS_Shape& tool)
{
  NCollection_List<TopoDS_Shape> faces;
  ShapeIndex result;
  TopExp::MapShapes(op.Shape(), TopAbs_FACE, result);
  for (TopExp_Explorer it(tool, TopAbs_FACE); it.More(); it.Next())
  {
    for (NCollection_List<TopoDS_Shape>::Iterator m(op.Modified(it.Current())); m.More(); m.Next())
      if (result.Contains(m.Value()))
        faces.Append(m.Value());
    if (result.Contains(it.Current()))
      faces.Append(it.Current());
  }
  return faces;
}

int main()
{
  const TopoDS_Shape base = box(0, 0, 0, 10, 10, 5);
  const TopoDS_Shape boss = cylinder(5, 5, 5, 2, 3);
  BRepAlgoAPI_Fuse fuse(base, boss);
  run("boss", fuse.Shape(), faces_from(fuse, boss));

  const TopoDS_Shape drill = cylinder(5, 5, -1, 2, 7);
  BRepAlgoAPI_Cut hole(base, drill);
  run("hole", hole.Shape(), faces_from(hole, drill));
  run("hole-no-history", hole.Shape(), faces_from(hole, drill), false);

  const TopoDS_Shape pocketTool = box(3, 3, 3, 4, 4, 3);
  BRepAlgoAPI_Cut pocket(base, pocketTool);
  run("pocket", pocket.Shape(), faces_from(pocket, pocketTool));

  const TopoDS_Shape drill2 = cylinder(2, 2, -1, 1, 7);
  BRepAlgoAPI_Cut both(fuse.Shape(), drill2);
  NCollection_List<TopoDS_Shape> twoFeatures = faces_from(both, drill2);
  const NCollection_List<TopoDS_Shape> bossFaces = faces_from(fuse, boss);
  ShapeIndex bothFaces;
  TopExp::MapShapes(both.Shape(), TopAbs_FACE, bothFaces);
  for (NCollection_List<TopoDS_Shape>::Iterator it(bossFaces); it.More(); it.Next())
    if (bothFaces.Contains(it.Value()))
      twoFeatures.Append(it.Value());
  run("two-features", both.Shape(), twoFeatures);

  TopoDS_Compound mixed;
  BRep_Builder().MakeCompound(mixed);
  BRep_Builder().Add(mixed, hole.Shape());
  BRep_Builder().Add(mixed, BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 20), gp::DZ()), 0, 1, 0, 1).Face());
  run("compound-with-face", mixed, faces_from(hole, drill));

  const TopoDS_Shape slotTool = box(4, -1, 2, 2, 12, 4);
  BRepAlgoAPI_Cut slot(base, slotTool);
  run("slot", slot.Shape(), faces_from(slot, slotTool));

  TopoDS_Compound twoSolids;
  BRep_Builder().MakeCompound(twoSolids);
  for (TopExp_Explorer it(hole.Shape(), TopAbs_SOLID); it.More(); it.Next())
    BRep_Builder().Add(twoSolids, it.Current());
  BRep_Builder().Add(twoSolids, box(20, 0, 0, 5, 5, 5));
  run("two-solids", twoSolids, faces_from(hole, drill));

  const TopoDS_Shape drum = cylinder(0, 0, 0, 10, 10);
  const TopoDS_Shape knob = box(8, -2, 3, 6, 4, 4);
  BRepAlgoAPI_Fuse knobbed(drum, knob);
  run("box-on-cylinder", knobbed.Shape(), faces_from(knobbed, knob));

  NCollection_List<TopoDS_Shape> outside;
  outside.Append(BRepBuilderAPI_MakeFace(gp_Pln(), 0, 1, 0, 1).Face());
  run("outside-face", base, outside);
  run("face-input", BRepBuilderAPI_MakeFace(gp_Pln(), 0, 1, 0, 1).Face(), outside);

  NCollection_List<TopoDS_Shape> top;
  for (TopExp_Explorer it(base, TopAbs_FACE); it.More(); it.Next())
    top.Append(it.Current());
  top.RemoveFirst();
  run("five-box-faces", base, top);
  return 0;
}
