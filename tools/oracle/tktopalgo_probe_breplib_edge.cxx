// BRepLib edge probe operations (BRepLib_MakeVertex, BRepLib_MakeEdge,
// BRepLib_MakeEdge2d); the Rust twin is crates/tktopalgo/examples/tp_probe/brep_lib_edge.rs.
//
// `blm_edge prec vertices ncalls (call)...`: the first call is a constructor, the later ones
// Init members of the same maker. A call is `id args`: 0 the default constructor; 1 V1 V2;
// 2 P1 P2; 10 + 4k + j a gp curve of kind k (0..4, its record without the kind) then j = 0
// nothing, 1 p1 p2, 2 P1 P2, 3 V1 V2; 30 + j a Geom curve record and 40 + j a Geom2d curve
// record and a surface, then j = 0 nothing, 1 p1 p2, 2 P1 P2, 3 V1 V2, 4 P1 P2 p1 p2,
// 5 V1 V2 p1 p2 (later calls: ids 30.. and 40.. only). `prec` > 0 sets BRepLib::Precision
// for the request (the default is restored after it).
// `blm_edge2d prec plane(0 | 1 P N) vertices ncalls (call)...`: the same for
// BRepLib_MakeEdge2d on 2d points and curves, on BRepLib::Plane() (the default or the
// plane P N, restored after); ids 1, 2, 10 + 4k + j and 30 + j as above (no default).
// Both print IsDone, Error, then the edge and both vertices (digests and the index of
// the input vertex they are the same as).
#include <BRepLib.hxx>
#include <BRepLib_MakeEdge.hxx>
#include <BRepLib_MakeEdge2d.hxx>
#include <BRepLib_MakeVertex.hxx>
#include <Geom_Plane.hxx>
// The index of the input vertex `v` is the same as, -1 for none (or a null vertex).
int tp_vertex_index(const TopoDS_Shape& v,const std::vector<TopoDS_Vertex>& vertices){
 for(size_t i=0;i<vertices.size();++i)if(!v.IsNull()&&v.IsSame(vertices[i]))return int(i);
 return -1;}
// Sets BRepLib::Precision for one request and restores the default after it.
struct TpPrecisionScope{
 explicit TpPrecisionScope(double p){if(p>0.)BRepLib::Precision(p);}
 ~TpPrecisionScope(){BRepLib::Precision(Precision::Confusion());}};
// The point type of a maker: gp_Pnt for BRepLib_MakeEdge, gp_Pnt2d for BRepLib_MakeEdge2d.
template<class M> struct TpEdgePoint;
template<> struct TpEdgePoint<BRepLib_MakeEdge>{static gp_Pnt read(GaReader& r){return r.pnt();}};
template<> struct TpEdgePoint<BRepLib_MakeEdge2d>{static gp_Pnt2d read(GaReader& r){return r.pnt2d();}};
// The constructor `M(a..., rest)` where `rest` is selected by `j` (see the header); the
// gp curves (`Full` false) have no constructor with both points or vertices and parameters.
template<class M,bool Full,class... A> M* tp_curve_ctor(int j,GaReader& r,const std::vector<TopoDS_Vertex>& vs,const A&... a){
 if(j==0)return new M(a...);
 if(j==1){double p1=r.real();return new M(a...,p1,r.real());}
 if(j==2||j==4){auto p=TpEdgePoint<M>::read(r);auto q=TpEdgePoint<M>::read(r);
  if constexpr(Full){if(j==4){double p1=r.real();return new M(a...,p,q,p1,r.real());}}
  return new M(a...,p,q);}
 TopoDS_Vertex v1=tp_vertex_ref(r,vs),v2=tp_vertex_ref(r,vs);
 if constexpr(Full){if(j==5){double p1=r.real();return new M(a...,v1,v2,p1,r.real());}}
 return new M(a...,v1,v2);}
// The Init member `m.Init(a..., rest)` where `rest` is selected by `j`.
template<class M,class... A> void tp_curve_init(M& m,int j,GaReader& r,const std::vector<TopoDS_Vertex>& vs,const A&... a){
 if(j==0){m.Init(a...);return;}
 if(j==1){double p1=r.real();m.Init(a...,p1,r.real());return;}
 if(j==2||j==4){auto p=TpEdgePoint<M>::read(r);auto q=TpEdgePoint<M>::read(r);if(j==2){m.Init(a...,p,q);return;}double p1=r.real();m.Init(a...,p,q,p1,r.real());return;}
 TopoDS_Vertex v1=tp_vertex_ref(r,vs),v2=tp_vertex_ref(r,vs);
 if(j==3){m.Init(a...,v1,v2);return;}double p1=r.real();m.Init(a...,v1,v2,p1,r.real());}
// A later call of `blm_edge`: an Init on a Geom curve (30..) or a curve on surface (40..).
void tp_edge_init(BRepLib_MakeEdge& m,int id,GaReader& r,const std::vector<TopoDS_Vertex>& vs){
 if(id<40){occ::handle<Geom_Curve> c=tp_geom_curve(r);tp_curve_init(m,id-30,r,vs,c);return;}
 occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);occ::handle<Geom_Surface> s=tp_surface(r);tp_curve_init(m,id-40,r,vs,c,s);}
BRepLib_MakeEdge* tp_edge_first(int id,GaReader& r,const std::vector<TopoDS_Vertex>& vs){
 using M=BRepLib_MakeEdge;
 if(id==0)return new M();
 if(id==1){TopoDS_Vertex v1=tp_vertex_ref(r,vs);return new M(v1,tp_vertex_ref(r,vs));}
 if(id==2){gp_Pnt a=r.pnt();return new M(a,r.pnt());}
 if(id>=40){occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);occ::handle<Geom_Surface> s=tp_surface(r);return tp_curve_ctor<M,true>(id-40,r,vs,c,s);}
 if(id>=30){occ::handle<Geom_Curve> c=tp_geom_curve(r);return tp_curve_ctor<M,true>(id-30,r,vs,c);}
 int k=(id-10)/4,j=(id-10)%4;
 if(k==0)return tp_curve_ctor<M,false>(j,r,vs,tp_lin(r));
 if(k==1)return tp_curve_ctor<M,false>(j,r,vs,tp_circ(r));
 if(k==2)return tp_curve_ctor<M,false>(j,r,vs,tp_elips(r));
 if(k==3)return tp_curve_ctor<M,false>(j,r,vs,tp_hypr(r));
 return tp_curve_ctor<M,false>(j,r,vs,tp_parab(r));}
BRepLib_MakeEdge2d* tp_edge2d_first(int id,GaReader& r,const std::vector<TopoDS_Vertex>& vs){
 using M=BRepLib_MakeEdge2d;
 if(id==1){TopoDS_Vertex v1=tp_vertex_ref(r,vs);return new M(v1,tp_vertex_ref(r,vs));}
 if(id==2){gp_Pnt2d a=r.pnt2d();return new M(a,r.pnt2d());}
 if(id>=30){occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);return tp_curve_ctor<M,true>(id-30,r,vs,c);}
 int k=(id-10)/4,j=(id-10)%4;
 if(k==0)return tp_curve_ctor<M,false>(j,r,vs,tp_lin2d(r));
 if(k==1)return tp_curve_ctor<M,false>(j,r,vs,tp_circ2d(r));
 if(k==2)return tp_curve_ctor<M,false>(j,r,vs,tp_elips2d(r));
 if(k==3)return tp_curve_ctor<M,false>(j,r,vs,tp_hypr2d(r));
 return tp_curve_ctor<M,false>(j,r,vs,tp_parab2d(r));}
// IsDone, Error (for MakeEdge2d only when not done: a success leaves it uninitialized),
// the edge and both vertices.
template<class M> void tp_edge_result(M& m,bool error_known,const std::vector<TopoDS_Vertex>& vs){
 ga_flag(m.IsDone());if(error_known)ga_int(m.Error());
 GA_TRY(tp_shape(m.Edge()))
 GA_TRY(tp_shape(m.Vertex1());ga_int(tp_vertex_index(m.Vertex1(),vs)))
 GA_TRY(tp_shape(m.Vertex2());ga_int(tp_vertex_index(m.Vertex2(),vs)))}
void tp_blm_edge_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 int n=r.integer();
 try{
  std::unique_ptr<BRepLib_MakeEdge> m(tp_edge_first(r.integer(),r,vs));
  for(int i=1;i<n;++i)tp_edge_init(*m,r.integer(),r,vs);
  tp_edge_result(*m,true,vs);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// Sets BRepLib::Plane for one request (0, or 1 P N) and restores the default after it.
struct TpPlaneScope{
 explicit TpPlaneScope(GaReader& r){if(r.flag()){gp_Pnt p=r.pnt();BRepLib::Plane(new Geom_Plane(p,r.dir()));}}
 ~TpPlaneScope(){BRepLib::Plane(occ::handle<Geom_Plane>());}};
void tp_blm_edge2d_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());TpPlaneScope plane(r);std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 int n=r.integer();
 try{
  std::unique_ptr<BRepLib_MakeEdge2d> m(tp_edge2d_first(r.integer(),r,vs));
  for(int i=1;i<n;++i){int id=r.integer();occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);tp_curve_init(*m,id-30,r,vs,c);}
  tp_edge_result(*m,!m->IsDone(),vs);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `blm_vertex prec x y z`: BRepLib_MakeVertex, IsDone and the vertex.
void tp_blm_vertex_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());gp_Pnt p=r.pnt();
 BRepLib_MakeVertex m(p);ga_flag(m.IsDone());GA_TRY(tp_shape(m.Vertex()))
 ga_end();}
