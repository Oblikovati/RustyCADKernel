#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeUpgrade_RemoveInternalWires.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <limits>
#include <string>

struct Fixture { TopoDS_Shape shape; TopoDS_Wire hole; };

static TopoDS_Wire rectangle(double low, double high)
{
  gp_Pnt points[] = {gp_Pnt(low, low, 0), gp_Pnt(high, low, 0),
                     gp_Pnt(high, high, 0), gp_Pnt(low, high, 0)};
  BRepBuilderAPI_MakeWire wire;
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  return wire.Wire();
}

static Fixture makeFixture()
{
  TopoDS_Wire outer = rectangle(0, 10);
  TopoDS_Wire hole = rectangle(2, 3);
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), outer, true).Face();
  BRep_Builder builder;
  builder.Add(face, hole.Oriented(TopAbs_INTERNAL));
  return {face, hole};
}

static Fixture makePartitionFixture(int adjacentFaces = 1)
{
  Fixture fixture = makeFixture();
  BRepBuilderAPI_MakeWire wireBuilder;
  for (TopExp_Explorer edgeExp(fixture.hole, TopAbs_EDGE); edgeExp.More(); edgeExp.Next())
    wireBuilder.Add(TopoDS::Edge(edgeExp.Current()));
  TopoDS_Compound compound;
  BRep_Builder builder;
  builder.MakeCompound(compound);
  builder.Add(compound, fixture.shape);
  for (int index = 0; index < adjacentFaces; ++index) {
    TopoDS_Face filler = BRepBuilderAPI_MakeFace(gp_Pln(), wireBuilder.Wire(), true).Face();
    builder.Add(compound, filler);
  }
  return {compound, fixture.hole};
}

static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind)
{
  int total = 0;
  for (TopExp_Explorer explorer(shape, kind); explorer.More(); explorer.Next()) ++total;
  return total;
}

static void run(const std::string& label, bool performWire, bool removeFaces, double threshold)
{
  Fixture fixture = makeFixture();
  ShapeUpgrade_RemoveInternalWires remover(fixture.shape);
  remover.MinArea() = threshold;
  remover.RemoveFaceMode() = removeFaces;
  bool done = false;
  if (performWire) {
    NCollection_Sequence<TopoDS_Shape> selection;
    selection.Append(fixture.hole);
    done = remover.Perform(selection);
  } else done = remover.Perform();
  TopoDS_Shape result = remover.GetResult();
  std::cout << label << ' ' << done << ' '
            << remover.Status(ShapeExtend_DONE1) << ' '
            << remover.Status(ShapeExtend_DONE2) << ' '
            << remover.Status(ShapeExtend_FAIL1) << ' '
            << remover.Status(ShapeExtend_FAIL2) << ' '
            << remover.RemovedWires().Length() << ' '
            << remover.RemovedFaces().Length() << ' '
            << count(result, TopAbs_FACE) << ' ' << count(result, TopAbs_WIRE) << '\n';
}

static void runFaceCandidate()
{
  Fixture fixture = makePartitionFixture();
  ShapeUpgrade_RemoveInternalWires remover(fixture.shape);
  remover.MinArea() = 2.0;
  bool done = remover.Perform();
  TopoDS_Shape result = remover.GetResult();
  std::cout << "remove_small_face " << done << ' '
            << remover.Status(ShapeExtend_DONE1) << ' '
            << remover.Status(ShapeExtend_DONE2) << ' '
            << remover.Status(ShapeExtend_FAIL1) << ' '
            << remover.Status(ShapeExtend_FAIL2) << ' '
            << remover.RemovedWires().Length() << ' '
            << remover.RemovedFaces().Length() << ' '
            << count(result, TopAbs_FACE) << ' ' << count(result, TopAbs_WIRE) << '\n';
}

static void runMultipleAdjacentCandidates()
{
  Fixture fixture = makePartitionFixture(2);
  ShapeUpgrade_RemoveInternalWires remover(fixture.shape);
  remover.MinArea() = 2.0;
  bool done = remover.Perform();
  TopoDS_Shape result = remover.GetResult();
  std::cout << "multiple_adjacent_faces " << done << ' '
            << remover.Status(ShapeExtend_DONE1) << ' '
            << remover.Status(ShapeExtend_DONE2) << ' '
            << remover.Status(ShapeExtend_FAIL1) << ' '
            << remover.Status(ShapeExtend_FAIL2) << ' '
            << remover.RemovedWires().Length() << ' '
            << remover.RemovedFaces().Length() << ' '
            << count(result, TopAbs_FACE) << ' ' << count(result, TopAbs_WIRE) << '\n';
}

static void runForeignFace()
{
  Fixture input = makeFixture();
  Fixture foreign = makeFixture();
  ShapeUpgrade_RemoveInternalWires remover(input.shape);
  remover.MinArea() = 2.0;
  NCollection_Sequence<TopoDS_Shape> selection;
  selection.Append(foreign.shape);
  bool done = remover.Perform(selection);
  TopoDS_Shape result = remover.GetResult();
  std::cout << "foreign_face " << done << ' '
            << remover.Status(ShapeExtend_DONE1) << ' '
            << remover.Status(ShapeExtend_DONE2) << ' '
            << remover.Status(ShapeExtend_FAIL1) << ' '
            << remover.Status(ShapeExtend_FAIL2) << ' '
            << remover.RemovedWires().Length() << ' '
            << remover.RemovedFaces().Length() << ' '
            << count(result, TopAbs_FACE) << ' ' << count(result, TopAbs_WIRE) << '\n';
}

static void runPersistentContext()
{
  Fixture fixture = makeFixture();
  ShapeUpgrade_RemoveInternalWires remover(fixture.shape);
  remover.MinArea() = 2.0;
  remover.Perform();
  remover.Init(fixture.shape);
  bool done = remover.Perform();
  TopoDS_Shape result = remover.GetResult();
  std::cout << "persistent_context " << done << ' '
            << remover.Status(ShapeExtend_DONE1) << ' '
            << remover.Status(ShapeExtend_DONE2) << ' '
            << remover.Status(ShapeExtend_FAIL1) << ' '
            << remover.Status(ShapeExtend_FAIL2) << ' '
            << remover.RemovedWires().Length() << ' '
            << remover.RemovedFaces().Length() << ' '
            << count(result, TopAbs_FACE) << ' ' << count(result, TopAbs_WIRE) << '\n';
}

class ProtectedStatusProbe : public ShapeUpgrade_RemoveInternalWires
{
public:
  void WriteStatus(int value) { myStatus = value; }
  int ReadStatus() const { return myStatus; }
};

static void runProtectedStatus()
{
  ProtectedStatusProbe probe;
  probe.WriteStatus(0x101);
  std::cout << "protected_status " << probe.ReadStatus() << ' '
            << probe.Status(ShapeExtend_DONE1) << ' '
            << probe.Status(ShapeExtend_FAIL1) << '\n';
}

int main()
{
  run("remove_hole", false, true, 2.0);
  run("remove_hole_no_face_mode", false, false, 2.0);
  run("threshold_keeps_hole", false, true, 0.5);
  run("nan_threshold_keeps_hole", false, true, std::numeric_limits<double>::quiet_NaN());
  run("selected_hole", true, false, 2.0);
  runForeignFace();
  runFaceCandidate();
  runMultipleAdjacentCandidates();
  runPersistentContext();
  runProtectedStatus();
  ShapeUpgrade_RemoveInternalWires empty;
  std::cout << "null_shape " << empty.Perform() << ' '
            << empty.Status(ShapeExtend_FAIL1) << ' '
            << empty.RemovedWires().Length() << '\n';
}
