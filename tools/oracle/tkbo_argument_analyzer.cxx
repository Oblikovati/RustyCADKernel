// Native oracle for BOPAlgo_ArgumentAnalyzer: the check results (status, faulty sub-shapes by
// index in the argument maps, distances) of every test on valid and faulty arguments.
#include <BOPAlgo_ArgumentAnalyzer.hxx>
#include <BRepAlgoAPI_Check.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Builder.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <gp_Pln.hxx>
#include <TopoDS_Compound.hxx>
#include "tkbo_shape_dump.hxx"

static TopoDS_Shape compound(const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  BRep_Builder    builder;
  TopoDS_Compound result;
  builder.MakeCompound(result);
  builder.Add(result, a);
  builder.Add(result, b);
  return result;
}

static void faulty(const char* tag, const TopoDS_Shape& argument,
                   const NCollection_List<TopoDS_Shape>& shapes)
{
  ShapeIndex all;
  if (!argument.IsNull())
    TopExp::MapShapes(argument, all);
  std::cout << tag << ' ' << shapes.Extent() << ':';
  for (NCollection_List<TopoDS_Shape>::Iterator it(shapes); it.More(); it.Next())
    std::cout << ' ' << it.Value().ShapeType() << '/' << all.FindIndex(it.Value()) << o(it.Value().Orientation());
  std::cout << '\n';
}

typedef void (*Configure)(BOPAlgo_ArgumentAnalyzer&);

static void run(const char* label, const TopoDS_Shape& s1, const TopoDS_Shape& s2, Configure configure)
{
  BOPAlgo_ArgumentAnalyzer analyzer;
  if (!s1.IsNull())
    analyzer.SetShape1(s1);
  if (!s2.IsNull())
    analyzer.SetShape2(s2);
  analyzer.SetRunParallel(false);
  configure(analyzer);
  analyzer.Perform();
  const NCollection_List<BOPAlgo_CheckResult>& results = analyzer.GetCheckResult();
  std::cout << "case " << label << " faulty " << analyzer.HasFaulty() << " results " << results.Extent() << '\n';
  for (NCollection_List<BOPAlgo_CheckResult>::Iterator it(results); it.More(); it.Next())
  {
    const BOPAlgo_CheckResult& r = it.Value();
    std::cout << "status " << r.GetCheckStatus() << " shapes " << r.GetShape1().IsNull() << ' '
              << r.GetShape2().IsNull() << " dist " << h(r.GetMaxDistance1()) << ' '
              << h(r.GetMaxParameter1()) << ' ' << h(r.GetMaxDistance2()) << ' '
              << h(r.GetMaxParameter2()) << '\n';
    faulty("f1", s1, r.GetFaultyShapes1());
    faulty("f2", s2, r.GetFaultyShapes2());
  }
}

static void check(const char* label, BRepAlgoAPI_Check&& checker)
{
  const NCollection_List<BOPAlgo_CheckResult>& results = checker.Result();
  std::cout << "check " << label << " valid " << checker.IsValid() << " results " << results.Extent();
  for (NCollection_List<BOPAlgo_CheckResult>::Iterator it(results); it.More(); it.Next())
    std::cout << ' ' << it.Value().GetCheckStatus() << '/' << it.Value().GetShape1().IsNull()
              << it.Value().GetShape2().IsNull() << '/' << it.Value().GetFaultyShapes1().Extent()
              << it.Value().GetFaultyShapes2().Extent();
  std::cout << '\n';
}

static void bopTestModes(BOPAlgo_ArgumentAnalyzer& a)
{
  a.ArgumentTypeMode()   = true;
  a.SelfInterMode()      = true;
  a.SmallEdgeMode()      = true;
  a.RebuildFaceMode()    = true;
  a.ContinuityMode()     = true;
  a.CurveOnSurfaceMode() = true;
}

static void allModes(BOPAlgo_ArgumentAnalyzer& a)
{
  bopTestModes(a);
  a.TangentMode()     = true;
  a.MergeVertexMode() = true;
  a.MergeEdgeMode()   = true;
}

static void fuseAll(BOPAlgo_ArgumentAnalyzer& a)
{
  allModes(a);
  a.OperationType() = BOPAlgo_FUSE;
}

static void cutTypes(BOPAlgo_ArgumentAnalyzer& a)
{
  a.ArgumentTypeMode() = true;
  a.OperationType()    = BOPAlgo_CUT;
}

static void sectionSmall(BOPAlgo_ArgumentAnalyzer& a)
{
  a.SmallEdgeMode() = true;
  a.OperationType() = BOPAlgo_SECTION;
}

static void stopFirst(BOPAlgo_ArgumentAnalyzer& a)
{
  fuseAll(a);
  a.StopOnFirstFaulty() = true;
}

// A planar face whose edge lies 0.01 above the plane while its p-curve is on it.
static TopoDS_Shape liftedEdgeFace()
{
  BRep_Builder builder;
  occ::handle<Geom_Plane> plane = new Geom_Plane(gp::XOY());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(plane, 0., 10., 0., 10., 1.e-7).Face();
  TopoDS_Edge edge;
  builder.MakeEdge(edge, new Geom_Line(gp_Pnt(0, 5, 0.01), gp::DX()), 1.e-7);
  builder.Range(edge, 0., 10.);
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Pnt2d(0, 5), gp_Dir2d(1, 0)), face, 1.e-7);
  builder.Add(edge, BRepBuilderAPI_MakeVertex(gp_Pnt(0, 5, 0.01)).Vertex());
  builder.Add(edge, BRepBuilderAPI_MakeVertex(gp_Pnt(10, 5, 0.01)).Vertex().Reversed());
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  TopoDS_Face internal = face;
  builder.Add(internal, wire.Oriented(TopAbs_INTERNAL));
  return internal;
}

// A planar face bounded by an open wire of three sides of a square.
static TopoDS_Shape openWireFace()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp::XOY()), 1.e-7);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(5, 0, 0)).Edge());
  builder.Add(wire, BRepBuilderAPI_MakeEdge(gp_Pnt(5, 0, 0), gp_Pnt(5, 5, 0)).Edge());
  builder.Add(wire, BRepBuilderAPI_MakeEdge(gp_Pnt(5, 5, 0), gp_Pnt(0, 5, 0)).Edge());
  builder.Add(face, wire);
  return face;
}

static TopoDS_Shape c0Edge()
{
  NCollection_Array1<gp_Pnt> poles(1, 3);
  poles(1) = gp_Pnt(0, 0, 0);
  poles(2) = gp_Pnt(1, 1, 0);
  poles(3) = gp_Pnt(2, 0, 0);
  NCollection_Array1<double> knots(1, 3);
  knots(1) = 0.;
  knots(2) = 1.;
  knots(3) = 2.;
  NCollection_Array1<int> mults(1, 3);
  mults(1) = 2;
  mults(2) = 1;
  mults(3) = 2;
  return BRepBuilderAPI_MakeEdge(new Geom_BSplineCurve(poles, knots, mults, 1)).Edge();
}

int main()
{
  const TopoDS_Shape none;
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(10., 10., 10.).Shape();
  const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(gp_Pnt(0, 0, 0), 10.).Shape();
  const TopoDS_Shape vertex = BRepBuilderAPI_MakeVertex(gp_Pnt(3, 3, 3)).Vertex();
  run("sphere", sphere, none, bopTestModes);
  run("box-vertex", compound(box, vertex), none, bopTestModes);
  run("nothing", none, none, bopTestModes);
  run("single-fuse", box, none, fuseAll);
  run("box-twin-boxes", box, compound(box, BRepPrimAPI_MakeBox(10., 10., 10.).Shape()), fuseAll);
  run("cut-solid-edge", box, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(5, 5, 5)).Edge(), cutTypes);
  const TopoDS_Shape tiny = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1.5e-7, 0, 0)).Edge();
  run("tiny-edge", tiny, none, bopTestModes);
  run("tiny-edge-section-far", tiny, BRepPrimAPI_MakeBox(gp_Pnt(20, 20, 20), 5., 5., 5.).Shape(), sectionSmall);
  run("tiny-edge-section-on", tiny, BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0)).Vertex(), sectionSmall);
  run("lifted-edge", liftedEdgeFace(), box, fuseAll);
  run("c0-edge", c0Edge(), none, bopTestModes);
  run("open-wire-face", openWireFace(), BRepBuilderAPI_MakeFace(gp_Pln(), 0., 1., 0., 1.).Face(), fuseAll);
  run("stop-first", compound(box, vertex), compound(box, BRepPrimAPI_MakeBox(10., 10., 10.).Shape()), stopFirst);
  check("check-box", BRepAlgoAPI_Check(box));
  check("check-null", BRepAlgoAPI_Check(none));
  check("check-box-vertex", BRepAlgoAPI_Check(compound(box, vertex), compound(box, vertex), BOPAlgo_FUSE));
  check("check-open-wire-face", BRepAlgoAPI_Check(openWireFace(), box, BOPAlgo_COMMON, false, false));
  check("check-cut-solid-edge",
        BRepAlgoAPI_Check(box, BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(5, 5, 5)).Edge(), BOPAlgo_CUT));
  return 0;
}
