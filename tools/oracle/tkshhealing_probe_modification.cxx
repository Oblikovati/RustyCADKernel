// ShapeCustom_Modification, ShapeCustom_TrsfModification, ShapeCustom_DirectModification
// and ShapeCustom_ConvertToRevolution probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/modification.rs.
//
// `sc_modif (prelude) program kind [(trsf) copymesh] reg nops (op)...`: the shapes of a
// prelude and a shape program (tkshhealing_probe_shcommon.cxx); kind 0 is a
// ShapeCustom_TrsfModification of the transformation record with IsCopyMesh() set to
// copymesh, 1 a ShapeCustom_DirectModification, 2 a ShapeCustom_ConvertToRevolution, 3 a
// ShapeCustom_ConvertToBSpline with the flags extrusion revolution offset plane (each set
// through its Set...Mode), 4 a ShapeCustom_SweptToElementary, 5 a
// ShapeCustom_BSplineRestriction(surf c3d c2d tol3d tol2d cont3d cont2d maxdeg maxseg deg
// rational) whose restriction parameters then get `nfields (field value)...` as in
// sc_restriction; with reg a ShapeExtend_MsgRegistrator is set (kinds 1 to 5). ShapeExtend::Init() loads the
// messages first. Each member starts from sentinel outputs (a null geometry, the identity
// location, tolerance -1, RevWires true, RevFace false, the point (7, 7, 7), the
// parameter 0.5) and prints its result and every output:
//  0 f: NewSurface; 1 e: NewCurve; 2 v: NewPoint; 3 e f ne nf: NewCurve2d; 4 v e:
//  NewParameter; 5 e f1 f2: Continuity(e, f1, f2, e, f1, f2); 6 s: BRepTools_Modifier(s,
//  M), IsDone and the digest of ModifiedShape(s); 7: the messages kept for each shape of
//  the table; 8: the dynamic type and IsKind of ShapeCustom_Modification and
//  BRepTools_TrsfModification; 9 s k g: SendMsg(shape, message "m<k>", gravity g);
//  10: MsgRegistrator().IsNull(); 11: for a B-spline restriction MaxErrors (the three
//  errors), NbOfSpan and the three error accessors; 12 k s: the ShapeCustom package
//  function k on s (0 DirectFaces, 1 ScaleShape 2.5, 2 BSplineRestriction(1e-3, 1e-6, 3,
//  2, C1, C1, true, false) with ConvertPlane, 3 ConvertToRevolution, 4 SweptToElementary,
//  5 ConvertToBSpline all modes, 6 ConvertToBSpline no mode) and the result digest;
//  13 s: ShapeCustom::ApplyModifier(s, M) with a new ShapeBuild_ReShape, then the result,
//  the context size and the reshape image of s.
#include <BRepTools_Modifier.hxx>
#include <ShapeCustom_BSplineRestriction.hxx>
#include <ShapeCustom_ConvertToBSpline.hxx>
#include <ShapeCustom_ConvertToRevolution.hxx>
#include <ShapeCustom_DirectModification.hxx>
#include <ShapeCustom_SweptToElementary.hxx>
#include <ShapeCustom_TrsfModification.hxx>
#include <ShapeCustom.hxx>
#include <ShapeExtend.hxx>
occ::handle<BRepTools_Modification> sc_modification(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Trsf T=shc_trsf(r);occ::handle<ShapeCustom_TrsfModification> m=new ShapeCustom_TrsfModification(T);
  m->IsCopyMesh()=r.flag();return m;}
 if(kind==1)return new ShapeCustom_DirectModification();
 if(kind==3){occ::handle<ShapeCustom_ConvertToBSpline> m=new ShapeCustom_ConvertToBSpline();
  m->SetExtrusionMode(r.flag());m->SetRevolutionMode(r.flag());m->SetOffsetMode(r.flag());m->SetPlaneMode(r.flag());
  return m;}
 if(kind==4)return new ShapeCustom_SweptToElementary();
 if(kind==5){bool sf=r.flag(),c3=r.flag(),c2=r.flag();double t3=r.real(),t2=r.real();
  GeomAbs_Shape k3=GeomAbs_Shape(r.integer()),k2=GeomAbs_Shape(r.integer());int deg=r.integer(),seg=r.integer();
  bool pri=r.flag(),rat=r.flag();
  occ::handle<ShapeCustom_BSplineRestriction> m=new ShapeCustom_BSplineRestriction(sf,c3,c2,t3,t2,k3,k2,deg,seg,pri,rat);
  int n=r.integer();for(int k=0;k<n;++k){int field=r.integer();sc_restriction_set(*m->GetRestrictionParameters(),field,r.integer());}
  return m;}
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
TopoDS_Shape sc_package(int k,const TopoDS_Shape& s){
 if(k==0)return ShapeCustom::DirectFaces(s);
 if(k==1)return ShapeCustom::ScaleShape(s,2.5);
 if(k==2){occ::handle<ShapeCustom_RestrictionParameters> p=new ShapeCustom_RestrictionParameters();p->ConvertPlane()=true;
  return ShapeCustom::BSplineRestriction(s,1e-3,1e-6,3,2,GeomAbs_C1,GeomAbs_C1,true,false,p);}
 if(k==3)return ShapeCustom::ConvertToRevolution(s);
 if(k==4)return ShapeCustom::SweptToElementary(s);
 return ShapeCustom::ConvertToBSpline(s,k==5,k==5,k==5,k==5);}
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
 else if(op==11){occ::handle<ShapeCustom_BSplineRestriction> b=occ::down_cast<ShapeCustom_BSplineRestriction>(m);
  double e3=-1.,e2=-1.;ga_real(b->MaxErrors(e3,e2));ga_real(e3);ga_real(e2);ga_int(b->NbOfSpan());
  ga_real(b->SurfaceError());ga_real(b->Curve3dError());ga_real(b->Curve2dError());}
 else if(op==12){int k=r.integer();TopoDS_Shape s=sh_shape(t,r.integer());GA_TRY(tp_shape(sc_package(k,s)))}
 else if(op==13){TopoDS_Shape s=sh_shape(t,r.integer());
  NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> context;BRepTools_Modifier MD;
  occ::handle<ShapeBuild_ReShape> rs=new ShapeBuild_ReShape();
  GA_TRY(tp_shape(ShapeCustom::ApplyModifier(s,m,context,MD,Message_ProgressRange(),rs));ga_int(context.Extent());
   tp_shape(rs->Apply(s)))}
 else ga_flag(!cm.IsNull()&&cm->MsgRegistrator().IsNull());}
void sc_modif_case(std::istream& input){
 ShapeExtend::Init();
 GaReader r{input};TpShapes t=shc_shapes(r);occ::handle<BRepTools_Modification> m=sc_modification(r);
 occ::handle<ShapeCustom_Modification> cm=occ::down_cast<ShapeCustom_Modification>(m);
 if(r.flag()&&!cm.IsNull())cm->SetMsgRegistrator(new ShapeExtend_MsgRegistrator());
 int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();if(op<=5)sc_member(r,t,m,op);else sc_other(r,t,m,op);}
 ga_end();}
