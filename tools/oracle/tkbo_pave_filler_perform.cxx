// Native oracle for the whole BOPAlgo_PaveFiller::Perform pipeline on solid pairs.
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_Curve.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_FaceInfo.hxx>
#include <BOPDS_Interf.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <gp_Ax2.hxx>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

static std::string h(double value)
{
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof bits);
  char text[17];
  std::snprintf(text, sizeof text, "%016llx", static_cast<unsigned long long>(bits));
  return text;
}

static void block(const BOPDS_DS& ds, const occ::handle<BOPDS_PaveBlock>& pb)
{
  int n1, n2;
  double t1, t2;
  pb->Indices(n1, n2);
  pb->Range(t1, t2);
  std::cout << " [" << n1 << ' ' << n2 << ' ' << h(t1) << ' ' << h(t2) << ' ' << pb->Edge() << ' '
            << pb->OriginalEdge();
  occ::handle<BOPDS_CommonBlock> cb = ds.CommonBlock(pb);
  if (!cb.IsNull())
  {
    std::cout << " cb" << cb->PaveBlocks().Extent() << ':' << cb->Faces().Extent() << ':'
              << h(cb->Tolerance());
  }
  std::cout << ']';
}

static void indexMap(const char* tag, const NCollection_Map<int>& m)
{
  std::cout << ' ' << tag;
  for (int v : m)
    std::cout << ' ' << v;
}

static void dump(BOPAlgo_PaveFiller& pf)
{
  const BOPDS_DS& ds = pf.DS();
  std::cout << "errors " << pf.HasErrors() << " warnings " << pf.HasWarnings() << '\n';
  std::cout << "shapes " << ds.NbSourceShapes() << ' ' << ds.NbShapes() << '\n';
  BOPDS_DS& mds = *pf.PDS();
  std::cout << "interf " << mds.InterfVV().Length() << ' ' << mds.InterfVE().Length() << ' '
            << mds.InterfEE().Length() << ' ' << mds.InterfVF().Length() << ' '
            << mds.InterfEF().Length() << ' ' << mds.InterfFF().Length() << '\n';
  for (int i = 0; i < ds.NbShapes(); ++i)
  {
    const BOPDS_ShapeInfo& si = ds.ShapeInfo(i);
    if (si.ShapeType() == TopAbs_VERTEX)
    {
      int sd = ds.GetSameDomainIndex(i);
      std::cout << "v " << i << ' ' << h(BRep_Tool::Tolerance(TopoDS::Vertex(si.Shape()))) << ' '
                << sd << '\n';
    }
    else if (si.ShapeType() == TopAbs_EDGE)
    {
      std::cout << "e " << i << ' ' << h(BRep_Tool::Tolerance(TopoDS::Edge(si.Shape()))) << ' '
                << si.Reference();
      for (const auto& pb : ds.PaveBlocks(i))
        block(ds, pb);
      std::cout << '\n';
    }
    else if (si.ShapeType() == TopAbs_FACE && ds.HasFaceInfo(i))
    {
      const BOPDS_FaceInfo& fi = ds.FaceInfo(i);
      std::cout << "f " << i;
      for (const auto* m : {&fi.PaveBlocksOn(), &fi.PaveBlocksIn(), &fi.PaveBlocksSc()})
      {
        std::cout << " |";
        for (int k = 1; k <= m->Extent(); ++k)
          std::cout << ' ' << (*m)(k)->Edge();
      }
      indexMap("on", fi.VerticesOn());
      indexMap("in", fi.VerticesIn());
      indexMap("sc", fi.VerticesSc());
      std::cout << '\n';
    }
  }
  for (int i = 0; i < mds.InterfFF().Length(); ++i)
  {
    const BOPDS_InterfFF& ff = mds.InterfFF()(i);
    std::cout << "ff " << ff.Index1() << ' ' << ff.Index2() << ' ' << ff.TangentFaces() << ' '
              << ff.Curves().Length() << ' ' << ff.Points().Length() << '\n';
    for (int c = 0; c < ff.Curves().Length(); ++c)
    {
      const BOPDS_Curve& nc = ff.Curves()(c);
      std::cout << " c " << h(nc.Tolerance());
      for (const auto& pb : nc.PaveBlocks())
        block(ds, pb);
      std::cout << '\n';
    }
  }
}

static void run(const char* label, const TopoDS_Shape& a, const TopoDS_Shape& b)
{
  std::cout << "case " << label << '\n';
  BOPAlgo_PaveFiller pf;
  NCollection_List<TopoDS_Shape> args;
  args.Append(a);
  args.Append(b);
  pf.SetArguments(args);
  pf.Perform();
  dump(pf);
}

int main()
{
  run("box-box", BRepPrimAPI_MakeBox(10, 10, 10).Shape(),
      BRepPrimAPI_MakeBox(gp_Pnt(5, 5, 5), 10, 10, 10).Shape());
  run("box-box-face", BRepPrimAPI_MakeBox(10, 10, 10).Shape(),
      BRepPrimAPI_MakeBox(gp_Pnt(10, 2, 2), 5, 5, 5).Shape());
  run("box-cylinder", BRepPrimAPI_MakeBox(10, 10, 10).Shape(),
      BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 5, -5), gp_Dir(0, 0, 1)), 2, 20).Shape());
  run("box-sphere", BRepPrimAPI_MakeBox(10, 10, 10).Shape(),
      BRepPrimAPI_MakeSphere(gp_Pnt(10, 10, 10), 4).Shape());
}
