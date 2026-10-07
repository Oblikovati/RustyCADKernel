// Native oracle for BOPDS_DS pave-block updates, common-block regrouping and face queries.
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_FaceInfo.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iomanip>
#include <iostream>

// Doubles are printed as their IEEE-754 bit patterns so the comparison is bit-exact.
static std::string h(double value)
{
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof bits);
  char text[17];
  std::snprintf(text, sizeof text, "%016llx", static_cast<unsigned long long>(bits));
  return text;
}

static TopoDS_Shape edge(double x1, double y1, double x2, double y2)
{
  return BRepBuilderAPI_MakeEdge(gp_Pnt(x1, y1, 0), gp_Pnt(x2, y2, 0)).Edge();
}

static TopoDS_Shape vertex(double x, double y, double tol)
{
  TopoDS_Vertex v;
  BRep_Builder().MakeVertex(v, gp_Pnt(x, y, 0), tol);
  return v;
}

static void blocks(BOPDS_DS& ds, int e)
{
  std::cout << "edge " << e << " ref " << ds.ShapeInfo(e).Reference();
  for (const auto& pb : ds.PaveBlocks(e))
  {
    int n1, n2;
    double t1, t2;
    pb->Indices(n1, n2);
    pb->Range(t1, t2);
    std::cout << " [" << n1 << ' ' << n2 << ' ' << h(t1) << ' ' << h(t2);
    occ::handle<BOPDS_CommonBlock> cb = ds.CommonBlock(pb);
    if (!cb.IsNull())
    {
      std::cout << " cb " << cb->PaveBlocks().Extent() << ' ' << (cb->PaveBlock1() == pb)
                << ' ' << cb->Faces().Extent() << ' ' << ds.RealPaveBlock(pb)->OriginalEdge();
    }
    std::cout << ']';
  }
  std::cout << '\n';
  NCollection_List<BOPDS_Pave> paves;
  ds.Paves(e, paves);
  std::cout << "paves " << e;
  for (const auto& p : paves)
    std::cout << ' ' << p.Index() << ':' << h(p.Parameter());
  std::cout << '\n';
}

static void commonBlocks(double offset, double fuzz, bool useSD)
{
  NCollection_List<TopoDS_Shape> args;
  args.Append(edge(0, 0, 10, 0));
  args.Append(edge(0, offset, 10, offset));
  args.Append(vertex(4, 0, 1e-7));
  BOPDS_DS ds;
  ds.SetArguments(args);
  ds.Init(fuzz);
  const int e1 = 0, e2 = 3, v = 6;
  if (useSD)
  {
    ds.AddShapeSD(4, 1);
    ds.AddShapeSD(5, 2);
    ds.UpdatePaveBlocksWithSDVertices();
  }
  const occ::handle<BOPDS_PaveBlock> pb1 = ds.ChangePaveBlocks(e1).First();
  const occ::handle<BOPDS_PaveBlock> pb2 = ds.ChangePaveBlocks(e2).First();
  occ::handle<BOPDS_CommonBlock> cb = new BOPDS_CommonBlock;
  cb->AddPaveBlock(pb1);
  cb->AddPaveBlock(pb2);
  cb->AddFace(7);
  ds.SetCommonBlock(pb1, cb);
  ds.SetCommonBlock(pb2, cb);
  pb1->AppendExtPave(BOPDS_Pave(v, 4.));
  pb2->AppendExtPave(BOPDS_Pave(v, 4.));
  pb2->AppendExtPave(BOPDS_Pave(v, 7.));
  ds.UpdateCommonBlock(cb, fuzz);
  std::cout << "case " << h(offset) << ' ' << h(fuzz) << ' ' << useSD << '\n';
  blocks(ds, e1);
  blocks(ds, e2);
  ds.ReleasePaveBlocks();
  blocks(ds, e1);
}

static void plainUpdates()
{
  NCollection_List<TopoDS_Shape> args;
  args.Append(edge(0, 0, 3, 0));
  args.Append(edge(0, 1, 3, 1));
  args.Append(vertex(1, 0, 1e-7));
  args.Append(vertex(2, 0, 1e-7));
  BOPDS_DS ds;
  ds.SetArguments(args);
  ds.Init(0.);
  occ::handle<BOPDS_PaveBlock> pb = ds.ChangePaveBlocks(0).First();
  pb->AppendExtPave(BOPDS_Pave(7, 2.));
  pb->AppendExtPave(BOPDS_Pave(6, 1.));
  ds.UpdatePaveBlocks();
  blocks(ds, 0);
  occ::handle<BOPDS_PaveBlock> last = ds.ChangePaveBlocks(0).Last();
  last->AppendExtPave(BOPDS_Pave(6, 2.5));
  ds.UpdatePaveBlock(last);
  blocks(ds, 0);
  ds.ChangePaveBlocks(3);
  ds.ReleasePaveBlocks();
  blocks(ds, 0);
  blocks(ds, 3);
  std::cout << "sub " << ds.IsSubShape(1, 0) << ' ' << ds.IsSubShape(4, 0) << '\n';
  ds.AddInterf(6, 1);
  std::cout << "interf " << ds.HasInterfShapeSubShapes(6, 0) << ' '
            << ds.HasInterfShapeSubShapes(6, 0, false) << ' ' << ds.HasInterfSubShapes(0, 0)
            << ' ' << ds.HasInterfSubShapes(3, 0) << '\n';
  ds.Dump();
  fflush(stdout);
}

static void boxQueries()
{
  NCollection_List<TopoDS_Shape> args;
  args.Append(BRepPrimAPI_MakeBox(1, 2, 3).Shape());
  BOPDS_DS ds;
  ds.SetArguments(args);
  ds.Init(0.);
  std::vector<int> faces;
  for (int i = 0; i < ds.NbSourceShapes(); ++i)
  {
    if (ds.ShapeInfo(i).ShapeType() == TopAbs_EDGE)
      for (auto& pb : ds.ChangePaveBlocks(i))
        pb->SetEdge(i);
    if (ds.ShapeInfo(i).ShapeType() == TopAbs_FACE)
      faces.push_back(i);
  }
  for (int f : faces)
    ds.ChangeFaceInfo(f);
  for (size_t i = 0; i < faces.size(); ++i)
  {
    for (size_t j = 0; j < faces.size(); ++j)
    {
      NCollection_List<int> edges;
      ds.SharedEdges(faces[i], faces[j], edges, NCollection_BaseAllocator::CommonBaseAllocator());
      std::cout << "shared " << faces[i] << ' ' << faces[j];
      for (int e : edges)
        std::cout << ' ' << e;
      std::cout << '\n';
    }
  }
  // Move the first face's On blocks into its In map to exercise RefineFaceInfoIn.
  BOPDS_FaceInfo& info = ds.ChangeFaceInfo(faces[0]);
  for (int k = 1; k <= info.PaveBlocksOn().Extent(); k += 2)
    info.ChangePaveBlocksIn().Add(info.PaveBlocksOn()(k));
  info.ChangeVerticesIn().Add(99);
  info.ChangeVerticesIn().Add(-3);
  info.ChangeVerticesSc().Add(98);
  NCollection_List<int> alone;
  ds.AloneVertices(faces[0], alone);
  std::cout << "alone";
  for (int v : alone)
    std::cout << ' ' << v;
  std::cout << '\n';
  std::cout << "in-before " << ds.FaceInfo(faces[0]).PaveBlocksIn().Extent() << '\n';
  NCollection_Map<int> mvOnIn, mvCommon;
  NCollection_IndexedMap<occ::handle<BOPDS_PaveBlock>> pbOnIn;
  NCollection_Map<occ::handle<BOPDS_PaveBlock>> pbCommon;
  ds.SubShapesOnIn(faces[0], faces[2], mvOnIn, mvCommon, pbOnIn, pbCommon);
  std::cout << "onin " << mvOnIn.Extent() << ' ' << mvCommon.Extent() << ' ' << pbOnIn.Extent()
            << ' ' << pbCommon.Extent() << " :";
  for (int v : mvOnIn)
    std::cout << ' ' << v;
  std::cout << " :";
  for (int v : mvCommon)
    std::cout << ' ' << v;
  for (int k = 1; k <= pbOnIn.Extent(); ++k)
    std::cout << ' ' << pbOnIn(k)->OriginalEdge();
  std::cout << '\n';
  ds.RefineFaceInfoIn();
  std::cout << "in-after " << ds.FaceInfo(faces[0]).PaveBlocksIn().Extent() << '\n';
  Bnd_Box box;
  int solid = -1;
  for (int i = 0; i < ds.NbSourceShapes(); ++i)
    if (ds.ShapeInfo(i).ShapeType() == TopAbs_SOLID)
      solid = i;
  ds.BuildBndBoxSolid(solid, box, true);
  // A single-solid argument flattens the solid's sub-shapes, so no shell is visited.
  std::cout << "box " << solid << ' ' << box.IsVoid() << ' ' << box.IsWhole() << '\n';

}

int main()
{
  plainUpdates();
  commonBlocks(0., 0., true);
  commonBlocks(1e-6, 1e-5, true);
  commonBlocks(0.5, 0., true);
  commonBlocks(0., 0., false);
  boxQueries();
}
