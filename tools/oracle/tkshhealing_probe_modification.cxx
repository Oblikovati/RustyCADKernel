// ShapeCustom_Modification, ShapeCustom_TrsfModification, ShapeCustom_DirectModification
// and ShapeCustom_ConvertToRevolution probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/modification.rs.
//
// `sc_modif (prelude) program kind [(trsf) copymesh] reg nops (op)...`: the shapes of a
// prelude and a shape program (tkshhealing_probe_shcommon.cxx); kind 0 is a
// ShapeCustom_TrsfModification of the transformation record with IsCopyMesh() set to
// copymesh, 1 a ShapeCustom_DirectModification, 2 a ShapeCustom_ConvertToRevolution; with
// reg a ShapeExtend_MsgRegistrator is set (kinds 1 and 2). ShapeExtend::Init() loads the
// messages first. Each member starts from sentinel outputs (a null geometry, the identity
// location, tolerance -1, RevWires true, RevFace false, the point (7, 7, 7), the
// parameter 0.5) and prints its result and every output:
//  0 f: NewSurface; 1 e: NewCurve; 2 v: NewPoint; 3 e f ne nf: NewCurve2d; 4 v e:
//  NewParameter; 5 e f1 f2: Continuity(e, f1, f2, e, f1, f2); 6 s: BRepTools_Modifier(s,
//  M), IsDone and the digest of ModifiedShape(s); 7: the messages kept for each shape of
//  the table; 8: the dynamic type and IsKind of ShapeCustom_Modification and
//  BRepTools_TrsfModification; 9 s k g: SendMsg(shape, message "m<k>", gravity g);
//  10: MsgRegistrator().IsNull().
#include <BRepTools_Modifier.hxx>
#include <ShapeCustom_ConvertToRevolution.hxx>
#include <ShapeCustom_DirectModification.hxx>
#include <ShapeCustom_TrsfModification.hxx>
#include <ShapeExtend.hxx>
occ::handle<BRepTools_Modification> sc_modification(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Trsf T=shc_trsf(r);occ::handle<ShapeCustom_TrsfModification> m=new ShapeCustom_TrsfModification(T);
  m->IsCopyMesh()=r.flag();return m;}
 if(kind==1)return new ShapeCustom_DirectModification();
 return new ShapeCustom_ConvertToRevolution();}
void sc_member(GaReader& r,const TpShapes& t,const occ::handle<BRepTools_Modification>& m,int op){
 if(op==0){TopoDS_Face f=sh_face(t,r.integer());occ::handle<Geom_Surface> S;TopLoc_Location L;double tol=-1.;bool rw=true,rf=false;
  GA_TRY(ga_flag(m->NewSurface(f,S,L,tol,rw,rf)))shc_surface(S);shc_location(L);ga_real(tol);ga_flag(rw);ga_flag(rf);}
 else if(op==1){TopoDS_Edge e=sh_edge(t,r.integer());occ::handle<Geom_Curve> C;TopLoc_Location L;double tol=-1.;
  GA_TRY(ga_flag(m->NewCurve(e,C,L,tol)))shc_curve(C);shc_location(L);ga_real(tol);}
 else if(op==2){TopoDS_Vertex v=TopoDS::Vertex(sh_shape(t,r.integer()));gp_Pnt P(7.,7.,7.);double tol=-1.;
  GA_TRY(ga_flag(m->NewPoint(v,P,tol)))ga_pnt(P);ga_real(tol);}
 else if(op==3){TopoDS_Edge e=sh_edge(t,r.integer());TopoDS_Face f=sh_face(t,r.integer());
  TopoDS_Edge ne=sh_edge(t,r.integer());TopoDS_Face nf=sh_face(t,r.integer());occ::handle<Geom2d_Curve> C;double tol=-1.;
  GA_TRY(ga_flag(m->NewCurve2d(e,f,ne,nf,C,tol)))shc_curve2d(C);ga_real(tol);}
 else if(op==4){TopoDS_Vertex v=TopoDS::Vertex(sh_shape(t,r.integer()));TopoDS_Edge e=sh_edge(t,r.integer());
  double p=0.5,tol=-1.;GA_TRY(ga_flag(m->NewParameter(v,e,p,tol)))ga_real(p);ga_real(tol);}
 else{TopoDS_Edge e=sh_edge(t,r.integer());TopoDS_Face f1=sh_face(t,r.integer()),f2=sh_face(t,r.integer());
  GA_TRY(ga_int(m->Continuity(e,f1,f2,e,f1,f2)))}}
void sc_registrator(const TpShapes& t,const occ::handle<ShapeCustom_Modification>& cm){
 occ::handle<ShapeExtend_MsgRegistrator> reg=cm.IsNull()?nullptr:occ::down_cast<ShapeExtend_MsgRegistrator>(cm->MsgRegistrator());
 ga_flag(!reg.IsNull());if(reg.IsNull())return;
 ga_int(reg->MapShape().Extent());
 for(const TopoDS_Shape& s:t.shapes){
  ga_flag(reg->MapShape().IsBound(s));if(reg->MapShape().IsBound(s))sh_rg_list(reg->MapShape().Find(s));}}
void sc_other(GaReader& r,const TpShapes& t,const occ::handle<BRepTools_Modification>& m,int op){
 occ::handle<ShapeCustom_Modification> cm=occ::down_cast<ShapeCustom_Modification>(m);
 if(op==6){TopoDS_Shape s=sh_shape(t,r.integer());
  GA_TRY(BRepTools_Modifier M(s,m);ga_flag(M.IsDone());tp_shape(M.ModifiedShape(s)))}
 else if(op==7)sc_registrator(t,cm);
 else if(op==8){ga_text(m->DynamicType()->Name());ga_flag(m->IsKind(STANDARD_TYPE(ShapeCustom_Modification)));
  ga_flag(m->IsKind(STANDARD_TYPE(BRepTools_TrsfModification)));}
 else if(op==9){TopoDS_Shape s=sh_shape(t,r.integer());int k=r.integer();Message_Gravity g=Message_Gravity(r.integer());
  if(!cm.IsNull())cm->SendMsg(s,sh_rg_message(k),g);}
 else ga_flag(!cm.IsNull()&&cm->MsgRegistrator().IsNull());}
void sc_modif_case(std::istream& input){
 ShapeExtend::Init();
 GaReader r{input};TpShapes t=shc_shapes(r);occ::handle<BRepTools_Modification> m=sc_modification(r);
 occ::handle<ShapeCustom_Modification> cm=occ::down_cast<ShapeCustom_Modification>(m);
 if(r.flag()&&!cm.IsNull())cm->SetMsgRegistrator(new ShapeExtend_MsgRegistrator());
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();if(op<=5)sc_member(r,t,m,op);else sc_other(r,t,m,op);}
 ga_end();}
