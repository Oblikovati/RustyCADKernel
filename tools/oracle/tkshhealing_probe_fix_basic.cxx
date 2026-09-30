// ShapeFix_Root, ShapeFix_ShapeTolerance and ShapeFix_EdgeConnect probe operations; the Rust
// twin is crates/tkshhealing/examples/sh_probe/fix_basic.rs.
//
// `sh_fixroot program nops (op)...` on a ShapeFix_Root: 0 v: SetPrecision; 1 v:
// SetMinTolerance; 2 v: SetMaxTolerance; 3 v: LimitTolerance (printed); 4 p min max: Set
// from another root with that precision and those limits; 5: SetContext(new
// ShapeBuild_ReShape); 6 kind: SetMsgRegistrator (0 null, 1 ShapeExtend_BasicMsgRegistrator,
// 2 ShapeExtend_MsgRegistrator); 7 s k which: SendMsg (which 0), SendWarning (1) or
// SendFail (2) of message `m<k>` on shape s; 8 k which: the same on myShape; 9 f d: NeedFix
// (printed). Then Precision, MinTolerance, MaxTolerance, Context().IsNull(), the
// registrator type name (or null) and, for a ShapeExtend_MsgRegistrator, its maps
// (sh_rg_maps).
//
// `sh_fixtol program nops (op)...`: 0 s tmin tmax styp: LimitTolerance (printed); 1 s preci
// styp: SetTolerance; then the tolerance of every vertex, edge and face of the table.
//
// `sh_edgeconnect program nops (op)... s`: 0 e1 e2: Add(edge, edge); 1 s: Add(shape); 2:
// Build; 3: Clear; then, for each edge of shape s, its count of vertices and its first and
// last vertices (IsSame index, point, tolerance). Build walks a hashed map, so the order of
// the vertices inside an edge that two groups change depends on addresses: the digest
// reads the vertices by orientation, not by position.
#include <ShapeFix_EdgeConnect.hxx>
#include <ShapeFix_Root.hxx>
#include <ShapeFix_ShapeTolerance.hxx>
#include <ShapeBuild_ReShape.hxx>
// NeedFix is protected: a subclass reaches it.
struct ShProbeRoot:ShapeFix_Root{static bool Need(int f,bool d){return NeedFix(f,d);}};
void sh_fr_send(const occ::handle<ShapeFix_Root>& root,const TopoDS_Shape* s,int k,int which){
 Message_Msg m=sh_rg_message(k);
 if(s){if(which==0)root->SendMsg(*s,m);else if(which==1)root->SendWarning(*s,m);else root->SendFail(*s,m);}
 else{if(which==0)root->SendMsg(m);else if(which==1)root->SendWarning(m);else root->SendFail(m);}}
void sh_fr_op(GaReader& r,const TpShapes& t,const occ::handle<ShapeFix_Root>& root,int op){
 if(op==0)root->SetPrecision(r.real());
 else if(op==1)root->SetMinTolerance(r.real());
 else if(op==2)root->SetMaxTolerance(r.real());
 else if(op==3)ga_real(root->LimitTolerance(r.real()));
 else if(op==4){occ::handle<ShapeFix_Root> o=new ShapeFix_Root();o->SetPrecision(r.real());o->SetMinTolerance(r.real());
  o->SetMaxTolerance(r.real());root->Set(o);}
 else if(op==5)root->SetContext(new ShapeBuild_ReShape());
 else if(op==6){int k=r.integer();occ::handle<ShapeExtend_BasicMsgRegistrator> reg;
  if(k==1)reg=new ShapeExtend_BasicMsgRegistrator();else if(k==2)reg=new ShapeExtend_MsgRegistrator();root->SetMsgRegistrator(reg);}
 else if(op==7){TopoDS_Shape s=sh_shape(t,r.integer());int k=r.integer();sh_fr_send(root,&s,k,r.integer());}
 else if(op==8){int k=r.integer();sh_fr_send(root,nullptr,k,r.integer());}
 else{int f=r.integer();ga_flag(ShProbeRoot::Need(f,r.flag()));}}
void sh_fixroot_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);occ::handle<ShapeFix_Root> root=new ShapeFix_Root();int nops=r.integer();
 for(int k=0;k<nops;++k)sh_fr_op(r,t,root,r.integer());
 ga_real(root->Precision());ga_real(root->MinTolerance());ga_real(root->MaxTolerance());ga_flag(root->Context().IsNull());
 occ::handle<ShapeExtend_BasicMsgRegistrator> reg=root->MsgRegistrator();
 ga_text(reg.IsNull()?"null":reg->DynamicType()->Name());
 occ::handle<ShapeExtend_MsgRegistrator> keeper=occ::down_cast<ShapeExtend_MsgRegistrator>(reg);
 if(!keeper.IsNull())sh_rg_maps(t,keeper,{});
 ga_end();}
// The tolerance of every vertex, edge and face of the table.
void sh_tolerances(const TpShapes& t){
 for(const TopoDS_Shape& s:t.shapes){
  if(s.ShapeType()==TopAbs_VERTEX)ga_real(BRep_Tool::Tolerance(TopoDS::Vertex(s)));
  else if(s.ShapeType()==TopAbs_EDGE)ga_real(BRep_Tool::Tolerance(TopoDS::Edge(s)));
  else if(s.ShapeType()==TopAbs_FACE)ga_real(BRep_Tool::Tolerance(TopoDS::Face(s)));}}
void sh_fixtol_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);ShapeFix_ShapeTolerance tool;int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();TopoDS_Shape s=sh_shape(t,r.integer());
  if(op==0){double a=r.real(),b=r.real();TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());GA_TRY(ga_flag(tool.LimitTolerance(s,a,b,ty)))}
  else{double p=r.real();TopAbs_ShapeEnum ty=TopAbs_ShapeEnum(r.integer());GA_TRY(tool.SetTolerance(s,p,ty))}}
 sh_tolerances(t);ga_end();}
void sh_ec_vertex(const TpShapes& t,const TopoDS_Vertex& v){
 ga_int(tp_same_index(t,v));if(!v.IsNull()){ga_pnt(BRep_Tool::Pnt(v));ga_real(BRep_Tool::Tolerance(v));}}
void sh_ec_digest(const TpShapes& t,const TopoDS_Shape& s){
 for(TopExp_Explorer ex(s,TopAbs_EDGE);ex.More();ex.Next()){const TopoDS_Edge& e=TopoDS::Edge(ex.Current());
  ga_int(e.NbChildren());TopoDS_Vertex a,b;TopExp::Vertices(e,a,b);sh_ec_vertex(t,a);sh_ec_vertex(t,b);}}
void sh_edgeconnect_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);ShapeFix_EdgeConnect tool;int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0){TopoDS_Edge a=sh_edge(t,r.integer());GA_TRY(tool.Add(a,sh_edge(t,r.integer())))}
  else if(op==1)GA_TRY(tool.Add(sh_shape(t,r.integer())))
  else if(op==2)GA_TRY(tool.Build())
  else tool.Clear();}
 GA_TRY(sh_ec_digest(t,sh_shape(t,r.integer())))
 ga_end();}
