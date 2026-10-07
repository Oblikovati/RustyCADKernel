// Native oracle for BOPAlgo_CellsBuilder: parts taken into and out of the result with
// materials, internal boundaries of solids removed and of edges and faces unified, containers,
// and the history of the arguments after each step (solids, faces, edges and a mixed case).
#include <BOPAlgo_CellsBuilder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include "tkbo_shape_dump.hxx"

static NCollection_List<TopoDS_Shape> list(const TopoDS_Shape& a, const TopoDS_Shape& b = TopoDS_Shape())
{
  NCollection_List<TopoDS_Shape> shapes;
  if (!a.IsNull())
    shapes.Append(a);
  if (!b.IsNull())
    shapes.Append(b);
  return shapes;
}

static void step(const char* label, BOPAlgo_CellsBuilder& cells, const NCollection_List<TopoDS_Shape>& arguments)
{
  std::cout << "step " << label << " warnings " << cells.HasWarnings() << " type "
            << cells.Shape().ShapeType() << '\n';
  ShapeIndex all;
  dumpResult(cells.Shape(), all);
  for (NCollection_List<TopoDS_Shape>::Iterator it(arguments); it.More(); it.Next())
  {
    std::cout << "history deleted " << cells.IsDeleted(it.Value()) << " modified:";
    for (NCollection_List<TopoDS_Shape>::Iterator m(cells.Modified(it.Value())); m.More(); m.Next())
      std::cout << ' ' << all.FindIndex(m.Value()) << o(m.Value().Orientation());
    std::cout << '\n';
  }
}

static void solids()
{
  const TopoDS_Shape b1 = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  const TopoDS_Shape b2 = BRepPrimAPI_MakeBox(gp_Pnt(5, 0, 0), 10., 10., 10.).Shape();
  const TopoDS_Shape b3 = BRepPrimAPI_MakeBox(gp_Pnt(2.5, 0, 5), 10., 10., 10.).Shape();
  NCollection_List<TopoDS_Shape> args = list(b1, b2);
  args.Append(b3);
  BOPAlgo_CellsBuilder cells;
  cells.SetArguments(args);
  cells.SetRunParallel(false);
  cells.Perform();
  std::cout << "case solids errors " << cells.HasErrors() << '\n';
  ShapeIndex parts;
  dumpResult(cells.GetAllParts(), parts);
  step("empty", cells, args);
  cells.AddToResult(list(b1), list(TopoDS_Shape()), 1, false);
  step("b1", cells, args);
  cells.RemoveInternalBoundaries();
  step("b1-rib", cells, args);
  cells.RemoveAllFromResult();
  cells.AddToResult(list(b1), list(b3), 1, false);
  cells.AddToResult(list(b2), list(b3), 1, false);
  step("b1b2-not-b3", cells, args);
  cells.RemoveInternalBoundaries();
  step("b1b2-not-b3-rib", cells, args);
  cells.RemoveAllFromResult();
  cells.AddToResult(list(b1), list(TopoDS_Shape()), 1, false);
  cells.AddToResult(list(b2), list(b1), 2, false);
  cells.AddToResult(list(b3), list(b1), 2, true);
  step("two-materials-updated", cells, args);
  cells.RemoveFromResult(list(b2, b3), list(TopoDS_Shape()));
  step("removed-b2b3", cells, args);
  cells.AddAllToResult(0, false);
  cells.MakeContainers();
  step("all-containers", cells, args);
  cells.RemoveFromResult(list(b1), list(b2));
  step("containers-removed", cells, args);
  cells.AddAllToResult(3, true);
  step("all-material", cells, args);
}

static void faces()
{
  const TopoDS_Shape f1 = BRepBuilderAPI_MakeFace(gp_Pln(), 0., 10., 0., 10.).Face();
  const TopoDS_Shape f2 = BRepBuilderAPI_MakeFace(gp_Pln(), 5., 15., 5., 15.).Face();
  NCollection_List<TopoDS_Shape> args = list(f1, f2);
  BOPAlgo_CellsBuilder cells;
  cells.SetArguments(args);
  cells.SetRunParallel(false);
  cells.Perform();
  std::cout << "case faces errors " << cells.HasErrors() << '\n';
  cells.AddToResult(list(f1, f2), list(TopoDS_Shape()), 0, false);
  cells.AddToResult(list(f1), list(f2), 0, false);
  step("common-and-f1", cells, args);
  cells.MakeContainers();
  step("shell", cells, args);
  cells.RemoveFromResult(list(f1, f2), list(TopoDS_Shape()));
  step("shell-without-common", cells, args);
}

static void mixed()
{
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  const TopoDS_Shape edge = BRepBuilderAPI_MakeEdge(gp_Pnt(5, 5, -5), gp_Pnt(5, 5, 15)).Edge();
  NCollection_List<TopoDS_Shape> args = list(box, edge);
  BOPAlgo_CellsBuilder cells;
  cells.SetArguments(args);
  cells.SetRunParallel(false);
  cells.Perform();
  std::cout << "case mixed errors " << cells.HasErrors() << '\n';
  cells.AddToResult(list(edge, box), list(TopoDS_Shape()), 0, false);
  step("edge-in-box", cells, args);
  cells.AddToResult(list(edge), list(box), 0, false);
  step("edge-outside", cells, args);
  cells.AddToResult(list(box), list(TopoDS_Shape()), 1, false);
  cells.AddToResult(list(edge), list(box), 1, true);
  step("mixed-material", cells, args);
}

static TopoDS_Shape circleFace(const gp_Pnt& center)
{
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(center, gp::DZ()), 10.)).Edge();
  return BRepBuilderAPI_MakeFace(BRepBuilderAPI_MakeWire(edge).Wire()).Face();
}

// Faces of one material unified by ShapeUpgrade_UnifySameDomain, keeping the material
// boundaries (the configuration of tests/boolean/history/A5).
static void unifiedFaces()
{
  const TopoDS_Shape f1 = circleFace(gp_Pnt(0, 0, 0));
  const TopoDS_Shape f2 = circleFace(gp_Pnt(10, 0, 0));
  const TopoDS_Shape f3 = circleFace(gp_Pnt(10, 10, 0));
  NCollection_List<TopoDS_Shape> args = list(f1, f2);
  args.Append(f3);
  BOPAlgo_CellsBuilder cells;
  cells.SetArguments(args);
  cells.SetRunParallel(false);
  cells.Perform();
  std::cout << "case unified-faces errors " << cells.HasErrors() << '\n';
  cells.AddToResult(list(f1), list(TopoDS_Shape()), 1, false);
  cells.RemoveInternalBoundaries();
  step("f1-rib", cells, args);
  cells.RemoveAllFromResult();
  cells.AddToResult(list(f1), list(f3), 1, false);
  cells.AddToResult(list(f2), list(f3), 1, false);
  cells.RemoveInternalBoundaries();
  step("f1f2-not-f3-rib", cells, args);
  cells.RemoveAllFromResult();
  cells.AddToResult(list(f1), list(TopoDS_Shape()), 1, false);
  cells.AddToResult(list(f2), list(f1), 2, false);
  cells.AddToResult(list(f3), list(f1), 2, false);
  cells.RemoveInternalBoundaries();
  step("two-materials-rib", cells, args);
  cells.MakeContainers();
  step("two-materials-containers", cells, args);
}

// Edges of one material unified, a crossing edge of another material kept apart.
static void unifiedEdges()
{
  const TopoDS_Shape e1 = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(10, 0, 0)).Edge();
  const TopoDS_Shape e2 = BRepBuilderAPI_MakeEdge(gp_Pnt(5, 0, 0), gp_Pnt(15, 0, 0)).Edge();
  const TopoDS_Shape e3 = BRepBuilderAPI_MakeEdge(gp_Pnt(7, -5, 0), gp_Pnt(7, 5, 0)).Edge();
  NCollection_List<TopoDS_Shape> args = list(e1, e2);
  args.Append(e3);
  BOPAlgo_CellsBuilder cells;
  cells.SetArguments(args);
  cells.SetRunParallel(false);
  cells.Perform();
  std::cout << "case unified-edges errors " << cells.HasErrors() << '\n';
  cells.AddToResult(list(e1), list(TopoDS_Shape()), 1, false);
  cells.AddToResult(list(e2), list(TopoDS_Shape()), 1, false);
  step("e1e2", cells, args);
  cells.RemoveInternalBoundaries();
  step("e1e2-rib", cells, args);
  cells.AddToResult(list(e3), list(TopoDS_Shape()), 2, false);
  cells.RemoveInternalBoundaries();
  step("e3-rib", cells, args);
}

int main()
{
  solids();
  faces();
  mixed();
  unifiedFaces();
  unifiedEdges();
  return 0;
}
