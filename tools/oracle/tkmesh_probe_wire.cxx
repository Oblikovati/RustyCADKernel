// Real topology and unchanged healing algorithms required by BRepMesh_ShapeVisitor.
#include <ShapeAnalysis.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_WireOrder.hxx>
#include <ShapeExtend_WireData.hxx>
#include <tuple>
std::tuple<TopoDS_Wire,TopoDS_Face,occ::handle<ShapeExtend_WireData>> order_fixture(int order,int reverse,int missing) {
  BRep_Builder builder;std::array<gp_Pnt,4> points={gp_Pnt(0,0,0),gp_Pnt(2,0,0),gp_Pnt(2,3,0),gp_Pnt(0,3,0)};
  std::array<TopoDS_Vertex,4> vertices;for(int i=0;i<4;++i)builder.MakeVertex(vertices[i],points[i],1.e-7);
  std::array<TopoDS_Edge,4> edges;
  for(int i=0;i<4;++i){int last=(i+1)%4;builder.MakeEdge(edges[i]);
    if(missing!=1||i!=1){builder.UpdateEdge(edges[i],new Geom_Line(gp_Ax1(points[i],gp_Dir(gp_Vec(points[i],points[last])))),1.e-7);builder.Range(edges[i],0,points[i].Distance(points[last]),false);}
    if(missing!=2||i!=1)builder.Add(edges[i],vertices[i].Oriented(TopAbs_FORWARD));builder.Add(edges[i],vertices[last].Oriented(TopAbs_REVERSED));}
  const std::array<std::array<int,4>,4> indices={{{0,1,2,3},{2,0,3,1},{1,2,3,0},{3,2,1,0}}};
  TopoDS_Wire wire;builder.MakeWire(wire);occ::handle<ShapeExtend_WireData> ordered=new ShapeExtend_WireData;
  for(int index:indices[order]){TopoDS_Edge edge=edges[index];if(reverse&(1<<index))edge.Reverse();ordered->Add(edge,0);builder.Add(wire,edge);}
  TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);builder.Add(face,wire);return {wire,face,ordered};
}
const std::array<ShapeExtend_Status,9> wire_statuses={ShapeExtend_OK,ShapeExtend_DONE,ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,ShapeExtend_DONE5,ShapeExtend_FAIL,ShapeExtend_FAIL2};
void mesh_wire_order(std::istream& input) {
  std::array<int,8> v;for(auto& value:v)input>>value;auto [wire,face,ordered]=order_fixture(v[0],v[1],v[2]);if(!v[3])face.Nullify();
  ShapeAnalysis_Wire analyzer(ordered,face,0.01);ShapeAnalysis_WireOrder order;ga_flag(analyzer.IsLoaded());ga_flag(analyzer.IsReady());ga_int(analyzer.NbEdges());ga_real(analyzer.Precision());
  for(int repeat=0;repeat<=v[7];++repeat){ga_flag(analyzer.CheckOrder(order,v[6],v[4],v[5]));ga_int(order.NbEdges());ga_int(order.Status());for(int i=1;i<=order.NbEdges();++i)ga_int(order.Ordered(i));for(auto status:wire_statuses){ga_flag(analyzer.LastCheckStatus(status));ga_flag(analyzer.StatusOrder(status));}}
  ga_flag(analyzer.CheckOrder(v[6],v[4]));for(auto status:wire_statuses){ga_flag(analyzer.LastCheckStatus(status));ga_flag(analyzer.StatusOrder(status));}
  analyzer.ClearStatuses();ga_flag(analyzer.LastCheckStatus(ShapeExtend_OK));ga_flag(analyzer.StatusOrder(ShapeExtend_OK));ga_end();
}
void mesh_outer_wire(std::istream& input) {
  std::array<int,5> v;for(auto& value:v)input>>value;auto [first,base,unused]=order_fixture(v[0],v[1],v[2]);auto [second,other,unused2]=order_fixture(0,0,0);
  BRep_Builder builder;TopoDS_Face face;builder.MakeFace(face,BRep_Tool::Surface(base),1.e-7);
  first.Orientation(v[3]&1?TopAbs_REVERSED:TopAbs_FORWARD);second.Orientation(v[3]&2?TopAbs_REVERSED:TopAbs_FORWARD);
  if(v[4]>0)builder.Add(face,first);if(v[4]>1)builder.Add(face,second);occ::handle<ShapeExtend_WireData> ordered=new ShapeExtend_WireData(first);
  ga_real(ShapeAnalysis::TotCross2D(ordered,face));TopoDS_Wire found=ShapeAnalysis::OuterWire(face);ga_int(found.IsNull()?0:found.IsSame(first)?1:found.IsSame(second)?2:-1);ga_end();
}
