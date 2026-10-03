// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <Precision.hxx>
#include <iostream>
using Incidence = NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher>;
using ShapeMap = NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher>;
using VertexMap = NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
static TopoDS_Edge edge(const TopoDS_Vertex& first,const TopoDS_Vertex& last) {
  BRep_Builder builder; TopoDS_Edge result; builder.MakeEdge(result);
  builder.Add(result,first.Oriented(TopAbs_FORWARD));
  builder.Add(result,last.Oriented(TopAbs_REVERSED)); return result;
}
static void observe(ShapeAnalysis_Wire& analyzer,VertexMap& loops,Incidence& incidence,
                    ShapeMap& small,ShapeMap& seams,const TopoDS_Vertex* vertices) {
  std::cout << ' ' << analyzer.CheckLoop(loops,incidence,small,seams);
  for(auto flag : {ShapeExtend_DONE1,ShapeExtend_FAIL1,ShapeExtend_FAIL2})
    std::cout << ' ' << analyzer.LastCheckStatus(flag) << ' ' << analyzer.StatusLoop(flag);
  std::cout << ' ' << (analyzer.Precision()==Precision::Infinite()) << ' ' << loops.Extent()
            << ' ' << incidence.Extent() << ' ' << small.Extent() << ' ' << seams.Extent();
  for(int i=0;i<4;++i) std::cout << ' ' << loops.FindIndex(vertices[i]) << ' '
    << (incidence.IsBound(vertices[i]) ? incidence.Find(vertices[i]).Extent() : 0);
}
static void inspect(const char* name,int mode) {
  BRep_Builder builder; TopoDS_Vertex vertices[4];
  for(int i=0;i<4;++i) builder.MakeVertex(vertices[i],gp_Pnt(i,0.,0.),1.e-7);
  TopoDS_Edge edges[]={edge(vertices[0],vertices[1]),edge(vertices[0],vertices[2]),edge(vertices[0],vertices[3])};
  occ::handle<ShapeExtend_WireData> wire = new ShapeExtend_WireData;
  if(mode!=0) wire->Add(edges[0]);
  if(mode>=2) wire->Add(mode==4 ? edges[0] : edges[1]);
  if(mode==3 || mode==4 || mode==6) wire->Add(mode==4 ? edges[0] : edges[2]);
  if(mode==5) wire->Add(TopoDS::Edge(edges[0].Reversed()));
  if(mode==6) builder.Degenerated(edges[2],true);
  if(mode==7) wire->Add(edge(vertices[0],vertices[0]));
  if(mode==8) { TopoDS_Edge empty; builder.MakeEdge(empty); wire->Add(empty); }
  ShapeAnalysis_Wire analyzer; analyzer.Load(wire); analyzer.SetPrecision(0.125);
  VertexMap loops; Incidence incidence; ShapeMap small,seams;
  if(mode==9) loops.Add(vertices[3]);
  std::cout << name;
  observe(analyzer,loops,incidence,small,seams,vertices);
  observe(analyzer,loops,incidence,small,seams,vertices);
  analyzer.ClearStatuses(); std::cout << ' ' << analyzer.StatusLoop(ShapeExtend_OK) << '\n';
}
int main() {
  const char* names[]={"empty","single","chain","branch","repeated","seam","degenerated","same_vertex","missing_vertex","prefilled"};
  for(int mode=0;mode<10;++mode) inspect(names[mode],mode);
}
