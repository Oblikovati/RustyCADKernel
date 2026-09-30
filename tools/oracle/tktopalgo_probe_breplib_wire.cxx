// BRepLib wire and solid probe operations (BRepLib_MakePolygon, BRepLib_MakeWire,
// BRepLib_MakeSolid); the Rust twin is crates/tktopalgo/examples/tp_probe/brep_lib_wire.rs.
//
// Edges: `n (edge)...` with an edge `kind vf vl ...` built with BRep_Builder on the
// vertices vf (FORWARD) and vl (REVERSED), -1 for none: kind 0 the line from vf towards vl
// on [0, distance] (tolerance 1e-7); 1 the circle P N R on [a1, a2]; 2 no curve,
// degenerated. An edge reference is `index orientation`.
//
// The first-call readers are templates over the maker, so that the BRepBuilderAPI probe
// runs the same calls on its makers.
// `blm_polygon prec vertices ctor args nops (op)...`: ctor 0 default, 1..3 two to four
// points and (for 2, 3) the close flag, 4..6 two to four vertex references and (for 5, 6)
// the close flag; op 0 Add(P), 1 Add(V), 2 Close(). Prints Added and IsDone after each op,
// then both vertices, the last edge and the wire.
// `blm_wire vertices edges ctor args nops (op)...`: ctor 0 default, 1..4 one to four edge
// references, 5 a wire `n refs`, 6 a wire then an edge reference; op 0 Add(E), 1 Add(W),
// 2 Add(L) of a list `n refs`.
// Prints IsDone and Error after the constructor and each op (Error only once an Add ran),
// then the wire, the last edge and the last vertex.
#include <BRepLib_MakePolygon.hxx>
#include <BRepLib_MakeSolid.hxx>
#include <BRepLib_MakeWire.hxx>
#include <TopoDS_CompSolid.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
std::vector<TopoDS_Edge> tp_edges(GaReader& r,const std::vector<TopoDS_Vertex>& vs){
 std::vector<TopoDS_Edge> edges(r.integer());BRep_Builder B;
 for(TopoDS_Edge& e:edges){
  int kind=r.integer();TopoDS_Vertex vf=tp_vertex_ref(r,vs),vl=tp_vertex_ref(r,vs);
  if(kind==0){gp_Pnt a=BRep_Tool::Pnt(vf),b=BRep_Tool::Pnt(vl);B.MakeEdge(e,new Geom_Line(a,gp_Dir(gp_Vec(a,b))),1e-7);B.Range(e,0.,a.Distance(b));}
  else if(kind==1){gp_Circ c=tp_circ(r);double a1=r.real();double a2=r.real();B.MakeEdge(e,new Geom_Circle(c),1e-7);B.Range(e,a1,a2);}
  else{B.MakeEdge(e);B.Degenerated(e,true);}
  if(!vf.IsNull())B.Add(e,vf.Oriented(TopAbs_FORWARD));
  if(!vl.IsNull())B.Add(e,vl.Oriented(TopAbs_REVERSED));}
 return edges;}
// `index orientation`.
TopoDS_Edge tp_edge_ref(GaReader& r,const std::vector<TopoDS_Edge>& es){
 int i=r.integer();return TopoDS::Edge(es.at(i).Oriented(TopAbs_Orientation(r.integer())));}
// `n refs`: a wire of the referenced edges built with BRep_Builder.
TopoDS_Wire tp_wire_of(GaReader& r,const std::vector<TopoDS_Edge>& es){
 TopoDS_Wire w;int n=r.integer();if(n<0)return w;BRep_Builder B;B.MakeWire(w);
 for(int i=0;i<n;++i)B.Add(w,tp_edge_ref(r,es));
 return w;}
template<class M=BRepLib_MakePolygon> M* tp_polygon_first(GaReader& r,const std::vector<TopoDS_Vertex>& vs){
 int ctor=r.integer();
 if(ctor==0)return new M();
 if(ctor<=3){gp_Pnt p1=r.pnt(),p2=r.pnt();if(ctor==1)return new M(p1,p2);
  gp_Pnt p3=r.pnt();if(ctor==2)return new M(p1,p2,p3,r.flag());
  gp_Pnt p4=r.pnt();return new M(p1,p2,p3,p4,r.flag());}
 TopoDS_Vertex v1=tp_vertex_ref(r,vs),v2=tp_vertex_ref(r,vs);if(ctor==4)return new M(v1,v2);
 TopoDS_Vertex v3=tp_vertex_ref(r,vs);if(ctor==5)return new M(v1,v2,v3,r.flag());
 TopoDS_Vertex v4=tp_vertex_ref(r,vs);return new M(v1,v2,v3,v4,r.flag());}
void tp_blm_polygon_case(std::istream& input){
 GaReader r{input};TpPrecisionScope scope(r.real());std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 try{
  std::unique_ptr<BRepLib_MakePolygon> m(tp_polygon_first(r,vs));ga_flag(m->Added());ga_flag(m->IsDone());
  int n=r.integer();
  for(int i=0;i<n;++i){int op=r.integer();
   if(op==0)m->Add(r.pnt());else if(op==1)m->Add(tp_vertex_ref(r,vs));else m->Close();
   ga_flag(m->Added());ga_flag(m->IsDone());}
  tp_shape(m->FirstVertex());ga_int(tp_vertex_index(m->FirstVertex(),vs));
  tp_shape(m->LastVertex());ga_int(tp_vertex_index(m->LastVertex(),vs));
  tp_shape(m->Edge());GA_TRY(tp_shape(m->Wire()))
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
template<class M=BRepLib_MakeWire> M* tp_wire_first(GaReader& r,const std::vector<TopoDS_Edge>& es,bool& known){
 int ctor=r.integer();known=true;
 if(ctor==0)return new M();
 if(ctor==5||ctor==6){TopoDS_Wire w=tp_wire_of(r,es);known=w.NbChildren()>0;
  if(ctor==5)return new M(w);known=true;return new M(w,tp_edge_ref(r,es));}
 TopoDS_Edge e1=tp_edge_ref(r,es);if(ctor==1)return new M(e1);
 TopoDS_Edge e2=tp_edge_ref(r,es);if(ctor==2)return new M(e1,e2);
 TopoDS_Edge e3=tp_edge_ref(r,es);if(ctor==3)return new M(e1,e2,e3);
 return new M(e1,e2,e3,tp_edge_ref(r,es));}
void tp_blm_wire_case(std::istream& input){
 GaReader r{input};std::vector<TopoDS_Vertex> vs=tp_vertices(r);
 try{
  std::vector<TopoDS_Edge> es=tp_edges(r,vs);bool known=false;
  std::unique_ptr<BRepLib_MakeWire> m(tp_wire_first(r,es,known));ga_flag(m->IsDone());if(known)ga_int(m->Error());
  int n=r.integer();
  for(int i=0;i<n;++i){
   int op=r.integer();
   if(op==0){m->Add(tp_edge_ref(r,es));known=true;}
   else if(op==2){NCollection_List<TopoDS_Shape> l;int k=r.integer();for(int j=0;j<k;++j)l.Append(tp_edge_ref(r,es));m->Add(l);known=true;}
   else{TopoDS_Wire w=tp_wire_of(r,es);known=known||w.NbChildren()>0;m->Add(w);}
   ga_flag(m->IsDone());if(known)ga_int(m->Error());}
  GA_TRY(tp_shape(m->Wire()))tp_shape(m->Edge());tp_shape(m->Vertex());ga_int(tp_vertex_index(m->Vertex(),vs));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `blm_solid nfaces (face)... nshells (n face-indices)... ctor args nops (shell-index)...`:
// faces of the face reader, shells of them built with BRep_Builder (Closed as
// BRep_Tool::IsClosed); ctor 0 default, 1 a compsolid `n shell-indices` (one solid per
// shell), 2..4 one to three shell indices, 5 a solid `n shell-indices`, 6 a solid then a
// shell index; each op adds a shell. Prints IsDone, the solid (for a compsolid, whose
// shell follows a hashed map, the face count, the Closed flag and whether each input face
// is in it) and FaceStatus of every input face.
std::vector<TopoDS_Shell> tp_shells(GaReader& r,const std::vector<TpFace>& faces){
 std::vector<TopoDS_Shell> shells(r.integer());BRep_Builder B;
 for(TopoDS_Shell& s:shells){B.MakeShell(s);int n=r.integer();for(int i=0;i<n;++i)B.Add(s,faces.at(r.integer()).face);s.Closed(BRep_Tool::IsClosed(s));}
 return shells;}
TopoDS_Solid tp_solid_of(GaReader& r,const std::vector<TopoDS_Shell>& shells){
 TopoDS_Solid s;BRep_Builder B;B.MakeSolid(s);int n=r.integer();
 for(int i=0;i<n;++i)B.Add(s,shells.at(r.integer()));
 return s;}
template<class M=BRepLib_MakeSolid> M* tp_solid_first(int ctor,GaReader& r,const std::vector<TopoDS_Shell>& shells){
 if(ctor==0)return new M();
 if(ctor==1){TopoDS_CompSolid cs;BRep_Builder B;B.MakeCompSolid(cs);int n=r.integer();
  for(int i=0;i<n;++i){TopoDS_Solid s;B.MakeSolid(s);B.Add(s,shells.at(r.integer()));B.Add(cs,s);}
  return new M(cs);}
 if(ctor==5)return new M(tp_solid_of(r,shells));
 if(ctor==6){TopoDS_Solid so=tp_solid_of(r,shells);return new M(so,shells.at(r.integer()));}
 const TopoDS_Shell& s1=shells.at(r.integer());if(ctor==2)return new M(s1);
 const TopoDS_Shell& s2=shells.at(r.integer());if(ctor==3)return new M(s1,s2);
 return new M(s1,s2,shells.at(r.integer()));}
// The order-free digest of a solid built from a compsolid.
void tp_merged_solid(const TopoDS_Solid& so,const std::vector<TpFace>& faces){
 ga_int(so.NbChildren());TopoDS_Iterator it(so);if(!it.More())return;
 const TopoDS_Shape& shell=it.Value();ga_int(shell.NbChildren());ga_flag(shell.Closed());
 for(const TpFace& f:faces){bool found=false;for(TopoDS_Iterator fit(shell);fit.More();fit.Next())found=found||fit.Value().IsSame(f.face);ga_flag(found);}}
void tp_blm_solid_case(std::istream& input){
 GaReader r{input};
 try{
  std::vector<TpFace> faces(r.integer());for(TpFace& f:faces)f=tp_face(r);
  std::vector<TopoDS_Shell> shells=tp_shells(r,faces);int ctor=r.integer();
  std::unique_ptr<BRepLib_MakeSolid> m(tp_solid_first(ctor,r,shells));
  int n=r.integer();for(int i=0;i<n;++i)m->Add(shells.at(r.integer()));
  ga_flag(m->IsDone());
  if(ctor==1)tp_merged_solid(m->Solid(),faces);else tp_shape(m->Solid());
  for(const TpFace& f:faces)ga_int(m->FaceStatus(f.face));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
