// Oracle for ShapeFix_WireVertex at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Tool.hxx>
#include <ShapeAnalysis_WireVertex.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_WireVertex.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

static occ::handle<ShapeExtend_WireData> pair() {
  occ::handle<ShapeExtend_WireData> data = new ShapeExtend_WireData;
  data->Add(BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge());
  data->Add(BRepBuilderAPI_MakeEdge(gp_Pnt(1.0001,0,0), gp_Pnt(2,0,0)).Edge());
  return data;
}
static occ::handle<ShapeExtend_WireData> shared_pair() {
  TopoDS_Vertex start = BRepBuilderAPI_MakeVertex(gp_Pnt(0,0,0)).Vertex();
  TopoDS_Vertex shared = BRepBuilderAPI_MakeVertex(gp_Pnt(1,0,0)).Vertex();
  TopoDS_Vertex finish = BRepBuilderAPI_MakeVertex(gp_Pnt(2,0,0)).Vertex();
  occ::handle<ShapeExtend_WireData> data = new ShapeExtend_WireData;
  data->Add(BRepBuilderAPI_MakeEdge(start, shared).Edge());
  data->Add(BRepBuilderAPI_MakeEdge(shared, finish).Edge());
  return data;
}
static void state(const char* label, int count, ShapeFix_WireVertex& fixer, int status) {
  auto data = fixer.WireData();
  TopoDS_Edge a = data->Edge(1), b = data->Edge(2);
  ShapeAnalysis_Edge analysis;
  TopoDS_Vertex va = analysis.LastVertex(a), vb = analysis.FirstVertex(b);
  std::cout << label << ' ' << count << ' ' << fixer.Analyzer().Status(1) << ' '
            << data->NbEdges() << ' ' << va.IsNull() << ' ' << vb.IsNull() << ' '
            << (va.IsSame(vb) ? 1 : 0);
  if (!va.IsNull() && !vb.IsNull()) {
    gp_Pnt pa = BRep_Tool::Pnt(va), pb = BRep_Tool::Pnt(vb);
    std::cout << ' ' << pa.X() << ' ' << pb.X() << ' '
              << BRep_Tool::Tolerance(va) << ' ' << BRep_Tool::Tolerance(vb) << ' '
              << BRep_Tool::Parameter(va, a) << ' ' << BRep_Tool::Parameter(vb, b);
  }
  std::cout << '\n';
}
static ShapeAnalysis_WireVertex initialized(const occ::handle<ShapeExtend_WireData>& data) {
  ShapeAnalysis_WireVertex analysis;
  analysis.Init(data, 0.75);
  analysis.SetPrecision(1e-3);
  analysis.Analyze();
  return analysis;
}
static void fix_same(const char* label, int status) {
  auto data = pair();
  auto analysis = initialized(data);
  if (status == 1) analysis.SetSameCoords(1);
  else analysis.SetClose(1);
  ShapeFix_WireVertex fixer;
  fixer.Init(analysis);
  state(label, fixer.FixSame(), fixer, status);
}
static void fix_status(const char* label, int status) {
  auto data = pair();
  auto analysis = initialized(data);
  if (status == 3) analysis.SetEnd(1, gp_XYZ(1.1,0,0), 0.25);
  if (status == 4) analysis.SetStart(1, gp_XYZ(0.9,0,0), 0.75);
  if (status == 5) analysis.SetInters(1, gp_XYZ(1.2,0,0), 0.6, 0.4);
  ShapeFix_WireVertex fixer;
  fixer.Init(analysis);
  state(label, fixer.Fix(), fixer, status);
}
int main() {
  std::cout << std::setprecision(17);
  fix_same("same-coords", 1);
  fix_same("close", 2);
  auto shared = shared_pair(); auto shared_analysis = initialized(shared);
  shared_analysis.SetSameCoords(1);
  ShapeFix_WireVertex shared_fixer; shared_fixer.Init(shared_analysis);
  state("same-topology-opposite-orientation", shared_fixer.FixSame(), shared_fixer, 1);
  fix_status("end", 3);
  fix_status("start", 4);
  fix_status("intersection", 5);
  auto data = pair(); auto analysis = initialized(data); analysis.SetDisjoined(1);
  ShapeFix_WireVertex fixer; fixer.Init(analysis); state("disjoined", fixer.Fix(), fixer, -1);
  auto all_data = pair(); auto all_analysis = initialized(all_data);
  all_analysis.SetDisjoined(1); all_analysis.SetDisjoined(2);
  ShapeFix_WireVertex all_fixer; all_fixer.Init(all_analysis);
  std::cout << "all-disjoined " << all_fixer.Fix() << ' '
            << all_fixer.Analyzer().Status(1) << ' ' << all_fixer.Analyzer().Status(2) << '\n';
  auto mixed_data = pair(); auto mixed_analysis = initialized(mixed_data);
  mixed_analysis.SetEnd(1, gp_XYZ(1.1,0,0), 0.25); mixed_analysis.SetSameVertex(2);
  ShapeFix_WireVertex mixed_fixer; mixed_fixer.Init(mixed_analysis);
  int mixed_count = mixed_fixer.Fix();
  ShapeAnalysis_Edge mixed_edges;
  TopoDS_Vertex mixed_last = mixed_edges.LastVertex(mixed_data->Edge(2));
  TopoDS_Vertex mixed_first = mixed_edges.FirstVertex(mixed_data->Edge(1));
  std::cout << "mixed-positive " << mixed_count << ' '
            << mixed_fixer.Analyzer().Status(1) << ' ' << mixed_fixer.Analyzer().Status(2) << ' '
            << mixed_last.IsNull() << ' ' << mixed_first.IsNull() << ' '
            << (mixed_last.IsSame(mixed_first) ? 1 : 0) << ' '
            << static_cast<int>(mixed_last.Orientation()) << ' '
            << static_cast<int>(mixed_first.Orientation()) << '\n';
  ShapeFix_WireVertex unloaded; std::cout << "unloaded " << unloaded.Fix() << ' ' << unloaded.FixSame() << '\n';
}
