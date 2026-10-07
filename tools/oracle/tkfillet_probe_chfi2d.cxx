// ChFi2d probes of tools/oracle/tkfillet_probe.cxx; the Rust twin is
// crates/tkfillet/examples/fl_probe/chfi2d.rs.
#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Tool.hxx>
#include <ChFi2d_AnaFilletAlgo.hxx>
#include <ChFi2d_Builder.hxx>
#include <ChFi2d_ChamferAPI.hxx>
#include <ChFi2d_FilletAlgo.hxx>
#include <ChFi2d_FilletAPI.hxx>
#include <GC_MakeArcOfCircle.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>

// An edge: null flag, orientation, curve type, range, vertices (with the orientation of
// the edge) and the point at the middle of the range.
void fl_edge(const TopoDS_Edge& e){
 ga_flag(e.IsNull());if(e.IsNull())return;
 ga_int(int(e.Orientation()));
 GA_TRY(BRepAdaptor_Curve c(e);ga_int(int(c.GetType()));ga_real(c.FirstParameter());ga_real(c.LastParameter());
        ga_pnt(c.Value(0.5*(c.FirstParameter()+c.LastParameter()))));
 TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2,true);
 ga_flag(v1.IsNull());if(!v1.IsNull())ga_pnt(BRep_Tool::Pnt(v1));
 ga_flag(v2.IsNull());if(!v2.IsNull())ga_pnt(BRep_Tool::Pnt(v2));}

// An edge spec: kind 0 the segment P1 P2, 1 the arc of circle through P1 Pm P2; 10 numbers.
TopoDS_Edge fl_edge_spec(GaReader& r){
 int kind=r.integer();gp_Pnt p1=r.pnt(),pm=r.pnt(),p2=r.pnt();
 if(kind==1)return BRepBuilderAPI_MakeEdge(GC_MakeArcOfCircle(p1,pm,p2).Value()).Edge();
 return BRepBuilderAPI_MakeEdge(p1,p2).Edge();}

// `fl_chfi2d_fillet mode E1 E2 radius P iSolution`: the fillet of the two edges in the
// plane z = 0. Mode 0 ChFi2d_FilletAPI on the wire of both edges, 1 on the edges, 2
// ChFi2d_AnaFilletAlgo on the edges, 3 ChFi2d_FilletAlgo on the edges.
void fl_chfi2d_fillet_case(std::istream& in){
 GaReader r{in};int mode=r.integer();TopoDS_Edge e1=fl_edge_spec(r),e2=fl_edge_spec(r);
 double radius=r.real();gp_Pnt p=r.pnt();int isol=r.integer();gp_Pln pln;
 TopoDS_Edge r1,r2,f;
 if(mode<=1){
  GA_TRY(ChFi2d_FilletAPI api;
   if(mode==0)api.Init(BRepBuilderAPI_MakeWire(e1,e2).Wire(),pln);else api.Init(e1,e2,pln);
   ga_flag(api.Perform(radius));ga_int(api.NbResults(p));f=api.Result(p,r1,r2,isol));
 }else if(mode==2){
  GA_TRY(ChFi2d_AnaFilletAlgo algo(e1,e2,pln);ga_flag(algo.Perform(radius));f=algo.Result(r1,r2));
 }else{
  GA_TRY(ChFi2d_FilletAlgo algo(e1,e2,pln);ga_flag(algo.Perform(radius));ga_int(algo.NbResults(p));f=algo.Result(p,r1,r2,isol));
 }
 fl_edge(f);fl_edge(r1);fl_edge(r2);ga_end();}

// `fl_chfi2d_chamfer mode E1 E2 L1 L2`: ChFi2d_ChamferAPI on the wire of both edges (mode
// 0) or the edges.
void fl_chfi2d_chamfer_case(std::istream& in){
 GaReader r{in};int mode=r.integer();TopoDS_Edge e1=fl_edge_spec(r),e2=fl_edge_spec(r);double l1=r.real(),l2=r.real();
 TopoDS_Edge r1,r2,f;
 GA_TRY(ChFi2d_ChamferAPI api;if(mode==0)api.Init(BRepBuilderAPI_MakeWire(e1,e2).Wire());else api.Init(e1,e2);
        ga_flag(api.Perform());f=api.Result(r1,r2,l1,l2));
 fl_edge(f);fl_edge(r1);fl_edge(r2);ga_end();}

// The edges of a face in exploration order.
void fl_face_edges(const TopoDS_Face& face){
 ga_flag(face.IsNull());if(face.IsNull())return;ga_int(int(face.Orientation()));
 int n=0;for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next())n++;
 ga_int(n);for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next())fl_edge(TopoDS::Edge(ex.Current()));}

// `fl_chfi2d_builder n P1..Pn nops (op a b x y)*`: ChFi2d_Builder on the polygon face of
// the points in z = 0. Ops: 1 AddFillet(vertex a, x), 2 AddChamfer(edge a, edge b, x,
// y), 3 AddChamfer(edge a, vertex b, x, y), 4 ModifyFillet(fillet a, x), 5
// RemoveFillet(fillet a), 6 ModifyChamfer(chamfer a, edge b, x, y) (angle), 7
// RemoveChamfer(chamfer a), 8 ModifyChamfer(chamfer a, edge b, edge b, x, y); indices
// (1-based) in the current result (vertices and edges in TopExp::MapShapes order, the
// fillets and chamfers of the builder).
void fl_chfi2d_builder_case(std::istream& in){
 GaReader r{in};int n=r.integer();BRepBuilderAPI_MakePolygon poly;
 for(int i=0;i<n;++i)poly.Add(r.pnt());poly.Close();
 TopoDS_Face face=BRepBuilderAPI_MakeFace(poly.Wire()).Face();
 ChFi2d_Builder b(face);ga_int(int(b.Status()));
 int nops=r.integer();
 for(int k=0;k<nops;++k){
  int op=r.integer(),a=r.integer(),bb=r.integer();double x=r.real(),y=r.real();
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> vs,es;TopExp::MapShapes(b.Result(),TopAbs_VERTEX,vs);TopExp::MapShapes(b.Result(),TopAbs_EDGE,es);
  auto V=[&](int i){return TopoDS::Vertex(vs(i));};auto E=[&](int i){return TopoDS::Edge(es(i));};
  auto F=[&](int i){return TopoDS::Edge(b.FilletEdges()(i));};auto C=[&](int i){return TopoDS::Edge(b.ChamferEdges()(i));};
  GA_TRY(
   if(op==1)fl_edge(b.AddFillet(V(a),x));
   else if(op==2)fl_edge(b.AddChamfer(E(a),E(bb),x,y));
   else if(op==3)fl_edge(b.AddChamfer(E(a),V(bb),x,y));
   else if(op==4)fl_edge(b.ModifyFillet(F(a),x));
   else if(op==5){TopoDS_Vertex v=b.RemoveFillet(F(a));ga_flag(v.IsNull());if(!v.IsNull())ga_pnt(BRep_Tool::Pnt(v));}
   else if(op==6)fl_edge(b.ModifyChamfer(C(a),E(bb),x,y));
   else if(op==7){TopoDS_Vertex v=b.RemoveChamfer(C(a));ga_flag(v.IsNull());if(!v.IsNull())ga_pnt(BRep_Tool::Pnt(v));}
   else fl_edge(b.ModifyChamfer(C(a),E(bb),E(bb),x,y)));
  ga_int(int(b.Status()));ga_int(b.NbFillet());ga_int(b.NbChamfer());
 }
 fl_face_edges(b.Result());ga_end();}
