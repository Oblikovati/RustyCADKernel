// BRepBuilderAPI maker probe operations (the package class, MakeVertex, MakeEdge,
// MakeEdge2d, MakePolygon, MakeWire, MakeFace, MakeShell, MakeSolid); the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_builder_api_makers.rs.
//
// Each maker request has the layout of the BRepLib request of the same maker
// (tktopalgo_probe_breplib_edge.cxx, _wire.cxx, _face.cxx and blp_shell of _package.cxx)
// and runs the same calls on the BRepBuilderAPI maker; it prints what the BRepLib request
// prints (IsDone and Error being the BRepBuilderAPI members), then `tp_bba_made`: the
// virtual IsDone, Check (1 or the exception type), and Shape() (Build and Check when
// IsDone is false) with the history of that shape (Generated and Modified counts,
// IsDeleted).
// `bba_vertex prec x y z`: BRepBuilderAPI_MakeVertex, tp_bba_made and the vertex.
// `bba_package prec plane(0 | 1 P N)`: BRepBuilderAPI::Precision(P) when prec > 0 and
//   BRepBuilderAPI::Plane(Geom_Plane(P, N)) when set, then Precision() of both packages,
//   whether Plane() is the BRepLib plane, and its location, axis and X direction; the
//   defaults are restored after.
// `bba_edge`, `bba_edge2d`, `bba_polygon`, `bba_wire`, `bba_face`: the layouts of
//   blm_edge, blm_edge2d, blm_polygon, blm_wire and blm_face.
// `bba_solid`: the layout of blm_solid; IsDeleted of every input face follows FaceStatus,
//   and the digest of Shape() is left out for a compsolid (its shell follows a hashed map).
// `bba_shell surface bounded (u1 u2 v1 v2) segment mode`: the layout of blp_shell.
#include <BRepBuilderAPI.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeEdge2d.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeShell.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
template<> struct TpEdgePoint<BRepBuilderAPI_MakeEdge>{static gp_Pnt read(GaReader& r){return r.pnt();}};
template<> struct TpEdgePoint<BRepBuilderAPI_MakeEdge2d>{static gp_Pnt2d read(GaReader& r){return r.pnt2d();}};
// The history of a built shape: Generated and Modified counts, IsDeleted.
void tp_bba_history(BRepBuilderAPI_MakeShape& m,const TopoDS_Shape& s){
 GA_TRY(ga_int(m.Generated(s).Extent()))GA_TRY(ga_int(m.Modified(s).Extent()))GA_TRY(ga_flag(m.IsDeleted(s)))}
// The virtual IsDone, Check, and Shape() (its digest when `digest`, else its null flag)
// with its history.
void tp_bba_made(BRepBuilderAPI_MakeShape& m,bool digest=true){
 ga_flag(m.IsDone());GA_TRY(m.Check();ga_int(1))
 GA_TRY(const TopoDS_Shape& s=m.Shape();if(digest)tp_shape(s);else ga_flag(s.IsNull());if(!s.IsNull())tp_bba_history(m,s))}
void tp_bba_vertex_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());gp_Pnt p=r.pnt();
 try{BRepBuilderAPI_MakeVertex m(p);tp_bba_made(m);tp_shape(m.Vertex());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_package_case(std::istream& input){
 GaReader r{input};double prec=r.real();
 if(prec>0.)BRepBuilderAPI::Precision(prec);
 if(r.flag()){gp_Pnt p=r.pnt();BRepBuilderAPI::Plane(new Geom_Plane(p,r.dir()));}
 ga_real(BRepBuilderAPI::Precision());ga_real(BRepLib::Precision());
 const occ::handle<Geom_Plane>& plane=BRepBuilderAPI::Plane();ga_flag(plane==BRepLib::Plane());
 ga_pnt(plane->Location());ga_dir(plane->Axis().Direction());ga_dir(plane->Position().XDirection());
 BRepBuilderAPI::Precision(Precision::Confusion());BRepBuilderAPI::Plane(occ::handle<Geom_Plane>());
 ga_end();}
void tp_bba_edge_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 int n=r.integer();
 try{
  std::unique_ptr<BRepBuilderAPI_MakeEdge> m(tp_edge_first<BRepBuilderAPI_MakeEdge>(r.integer(),r,vs));
  for(int i=1;i<n;++i)tp_edge_init(*m,r.integer(),r,vs);
  tp_edge_result(*m,true,vs);tp_bba_made(*m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_edge2d_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());TpPlaneScope plane(r);std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 int n=r.integer();
 try{
  std::unique_ptr<BRepBuilderAPI_MakeEdge2d> m(tp_edge2d_first<BRepBuilderAPI_MakeEdge2d>(r.integer(),r,vs));
  for(int i=1;i<n;++i){int id=r.integer();occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);tp_curve_init(*m,id-30,r,vs,c);}
  tp_edge_result(*m,!m->IsDone(),vs);tp_bba_made(*m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_polygon_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 try{
  std::unique_ptr<BRepBuilderAPI_MakePolygon> m(tp_polygon_first<BRepBuilderAPI_MakePolygon>(r,vs));ga_flag(m->Added());ga_flag(m->IsDone());
  int n=r.integer();
  for(int i=0;i<n;++i){int op=r.integer();
   if(op==0)m->Add(r.pnt());else if(op==1)m->Add(tp_vertex_ref(r,vs));else m->Close();
   ga_flag(m->Added());ga_flag(m->IsDone());}
  tp_shape(m->FirstVertex());ga_int(tp_vertex_index(m->FirstVertex(),vs));
  tp_shape(m->LastVertex());ga_int(tp_vertex_index(m->LastVertex(),vs));
  tp_shape(m->Edge());GA_TRY(tp_shape(m->Wire()))tp_bba_made(*m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_wire_case(std::istream& input){
 GaReader r{input};std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 try{
  std::vector<TopoDS_Edge> es=tp_edges(r,vs);bool known=false;
  std::unique_ptr<BRepBuilderAPI_MakeWire> m(tp_wire_first<BRepBuilderAPI_MakeWire>(r,es,known));ga_flag(m->IsDone());if(known)ga_int(m->Error());
  int n=r.integer();
  for(int i=0;i<n;++i){
   int op=r.integer();
   if(op==0){m->Add(tp_edge_ref(r,es));known=true;}
   else if(op==2){NCollection_List<TopoDS_Shape> l;int k=r.integer();for(int j=0;j<k;++j)l.Append(tp_edge_ref(r,es));m->Add(l);known=true;}
   else{TopoDS_Wire w=tp_wire_of(r,es);known=known||w.NbChildren()>0;m->Add(w);}
   ga_flag(m->IsDone());if(known)ga_int(m->Error());}
  GA_TRY(tp_shape(m->Wire()))tp_shape(m->Edge());tp_shape(m->Vertex());ga_int(tp_vertex_index(m->Vertex(),vs));tp_bba_made(*m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_solid_case(std::istream& input){
 GaReader r{input};
 try{
  std::vector<TpFace> faces(r.integer());for(TpFace& f:faces)f=tp_face(r);
  std::vector<TopoDS_Shell> shells=tp_shells(r,faces);int ctor=r.integer();
  std::unique_ptr<BRepBuilderAPI_MakeSolid> m(tp_solid_first<BRepBuilderAPI_MakeSolid>(ctor,r,shells));
  int n=r.integer();for(int i=0;i<n;++i)m->Add(shells.at(r.integer()));
  ga_flag(m->IsDone());
  if(ctor==1)tp_merged_solid(m->Solid(),faces);else tp_shape(m->Solid());
  for(const TpFace& f:faces)ga_flag(m->IsDeleted(f.face));
  tp_bba_made(*m,ctor!=1);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// IsDone, Error and the face of a BRepBuilderAPI_MakeFace.
void tp_bba_face_maker(const BRepBuilderAPI_MakeFace& m){ga_flag(m.IsDone());ga_int(m.Error());GA_TRY(tp_shape(m.Face()))}
void tp_bba_face_case(std::istream& input){
 GaReader r{input};
 try{BRepBuilderAPI_MakeFace m;
  if(!tp_face_request(r,m,tp_bba_face_maker)){ga_int(-2);ga_end();return;}
  tp_bba_face_maker(m);tp_bba_made(m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_bba_shell_case(std::istream& input){
 GaReader r{input};
 try{occ::handle<Geom_Surface> s=tp_surface(r);bool bounded=r.flag();double u1=0.,u2=0.,v1=0.,v2=0.;
  if(bounded){u1=r.real();u2=r.real();v1=r.real();v2=r.real();}bool segment=r.flag();int mode=r.integer();
  std::unique_ptr<BRepBuilderAPI_MakeShell> m;
  if(mode==1){m.reset(new BRepBuilderAPI_MakeShell());ga_flag(m->IsDone());ga_int(m->Error());
   if(!bounded)s->Bounds(u1,u2,v1,v2);m->Init(s,u1,u2,v1,v2,segment);}
  else if(bounded)m.reset(new BRepBuilderAPI_MakeShell(s,u1,u2,v1,v2,segment));else m.reset(new BRepBuilderAPI_MakeShell(s,segment));
  ga_flag(m->IsDone());ga_int(m->Error());tp_shape(m->Shell());tp_bba_made(*m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
