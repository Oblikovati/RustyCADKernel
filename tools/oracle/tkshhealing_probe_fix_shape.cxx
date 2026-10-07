// Shape-level ShapeFix probe operation; the Rust twin is
// crates/tkshhealing/examples/sh_probe/fix_shape.rs.
//
// `sh_fix_shape family (input) prec maxtol op`: the input is a `sh_unify` input (family 0),
// a `sh_fix_face` input (family 1), a primitive solid (family 2: type 0 cylinder R H,
// 1 cone R1 R2 H, 2 sphere R, 3 torus R1 R2), or a face of tkshhealing_probe_fix_shape_checks.cxx
// (families 3 to 5). The op, after which the digest of the resulting shape is printed:
//  0 ShapeFix_Shape (precision, maximal tolerance) Perform: the flag and
//    Status(DONE1..DONE8, FAIL1..FAIL3);
//  1, 2 ShapeFix::SameParameter(shape, enforce = op == 2): the flag;
//  3 ShapeFix::FixVertexPosition(shape, maxtol, context): the flag (the context applied);
//  4 ShapeFix::RemoveSmallEdges(shape, maxtol, context);
//  5 ShapeFix_Wireframe (precision, maximal tolerance) FixWireGaps then FixSmallEdges: both
//    flags and StatusWireGaps / StatusSmallEdges (DONE1..DONE8, FAIL1..FAIL3); 11 the same
//    in the drop mode of small edges, 12 the same with the limit angle 0.1;
//  6 ShapeFix_FixSmallFace (precision) Init then Perform;
//  7, 8 ShapeFix_FixSmallSolid (volume threshold maxtol) Remove or Merge with a context;
//  9 ShapeFix_Shell Perform on each shell, 10 ShapeFix_Solid Perform on each solid: the
//    flag, the status flags and the digest of each;
//  13 one ShapeAnalysis_CheckSmallFace (precision): the CheckSpotFace flag of each face,
//    then the CheckStripFace flag of each face and the digests of the two strip edges;
//  14 ShapeFix_FixSmallFace (precision) Init, then on each strip face ReplaceInCaseOfStrip
//    (the flag) and RemoveFacesInCaseOfStrip: the shape with the context applied; 15 the
//    same, then the result of FixShape;
//  16 to 32 the checks and wire repairs of tkshhealing_probe_fix_shape_checks.cxx.
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeFix.hxx>
#include <ShapeFix_FixSmallFace.hxx>
#include <ShapeFix_FixSmallSolid.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeFix_Shell.hxx>
#include <ShapeFix_Solid.hxx>
#include <ShapeFix_Wireframe.hxx>
TopoDS_Shape sfs_solid(GaReader& r){
 int type=r.integer();
 if(type==0){double R=r.real();return BRepPrimAPI_MakeCylinder(R,r.real()).Shape();}
 if(type==1){double R1=r.real(),R2=r.real();return BRepPrimAPI_MakeCone(R1,R2,r.real()).Shape();}
 if(type==2)return BRepPrimAPI_MakeSphere(r.real()).Shape();
 double R1=r.real();return BRepPrimAPI_MakeTorus(R1,r.real()).Shape();}
const ShapeExtend_Status sfs_statuses[11]={ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,
 ShapeExtend_DONE5,ShapeExtend_DONE6,ShapeExtend_DONE7,ShapeExtend_DONE8,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3};
void sfs_shape_fix(const TopoDS_Shape& s,double prec,double maxtol){
 occ::handle<ShapeFix_Shape> f=new ShapeFix_Shape(s);f->SetPrecision(prec);f->SetMaxTolerance(maxtol);
 ga_flag(f->Perform());for(ShapeExtend_Status x:sfs_statuses)ga_flag(f->Status(x));tp_shape(f->Shape());}
void sfs_shells(const TopoDS_Shape& s){
 for(TopExp_Explorer e(s,TopAbs_SHELL);e.More();e.Next()){
  occ::handle<ShapeFix_Shell> f=new ShapeFix_Shell(TopoDS::Shell(e.Current()));
  GA_TRY(ga_flag(f->Perform());for(ShapeExtend_Status x:sfs_statuses)ga_flag(f->Status(x));tp_shape(f->Shape()))}}
void sfs_solids(const TopoDS_Shape& s){
 for(TopExp_Explorer e(s,TopAbs_SOLID);e.More();e.Next()){
  occ::handle<ShapeFix_Solid> f=new ShapeFix_Solid(TopoDS::Solid(e.Current()));
  GA_TRY(ga_flag(f->Perform());for(ShapeExtend_Status x:sfs_statuses)ga_flag(f->Status(x));tp_shape(f->Shape()))}}
void sfs_edge(const TopoDS_Edge& e){ga_flag(!e.IsNull());if(!e.IsNull())tp_shape(e);}
#include "tkshhealing_probe_fix_shape_checks.cxx"
void sfs_small_faces(const TopoDS_Shape& s,double prec){
 ShapeAnalysis_CheckSmallFace a;
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next())GA_TRY(ga_flag(a.CheckSpotFace(TopoDS::Face(e.Current()),prec)))
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next()){
  TopoDS_Face f=TopoDS::Face(e.Current());TopoDS_Edge e1,e2;
  GA_TRY(ga_flag(a.CheckStripFace(f,e1,e2,prec));sfs_edge(e1);sfs_edge(e2))}}
void sfs_strips(const TopoDS_Shape& s,double prec,bool fix){
 occ::handle<ShapeFix_FixSmallFace> fx=new ShapeFix_FixSmallFace;fx->Init(s);fx->SetPrecision(prec);
 for(TopExp_Explorer e(s,TopAbs_FACE);e.More();e.Next()){
  TopoDS_Face f=TopoDS::Face(fx->Context()->Apply(e.Current()));if(f.IsNull())continue;
  ShapeAnalysis_CheckSmallFace a;TopoDS_Edge e1,e2;
  if(!a.CheckStripFace(f,e1,e2,prec))continue;
  bool ok=fx->ReplaceInCaseOfStrip(f,e1,e2,prec);ga_flag(ok);if(ok)fx->RemoveFacesInCaseOfStrip(f);}
 tp_shape(fx->Context()->Apply(s));
 if(fix){fx->Init(fx->Context()->Apply(s));tp_shape(fx->FixShape());}}
void sfs_run(TopoDS_Shape s,double prec,double maxtol,int op){
 occ::handle<ShapeBuild_ReShape> ctx=new ShapeBuild_ReShape;
 if(op==0){sfs_shape_fix(s,prec,maxtol);return;}
 if(op==1||op==2){ga_flag(ShapeFix::SameParameter(s,op==2));tp_shape(s);return;}
 if(op==3){ga_flag(ShapeFix::FixVertexPosition(s,maxtol,ctx));tp_shape(ctx->Apply(s));return;}
 if(op==4){tp_shape(ShapeFix::RemoveSmallEdges(s,maxtol,ctx));return;}
 if(op==5||op==11||op==12){occ::handle<ShapeFix_Wireframe> w=new ShapeFix_Wireframe(s);w->SetPrecision(prec);
  w->SetMaxTolerance(maxtol);if(op==11)w->ModeDropSmallEdges()=true;if(op==12)w->SetLimitAngle(0.1);
  ga_flag(w->FixWireGaps());ga_flag(w->FixSmallEdges());
  for(ShapeExtend_Status x:sfs_statuses){ga_flag(w->StatusWireGaps(x));ga_flag(w->StatusSmallEdges(x));}
  tp_shape(w->Shape());return;}
 if(op==6){occ::handle<ShapeFix_FixSmallFace> f=new ShapeFix_FixSmallFace;f->Init(s);f->SetPrecision(prec);
  f->Perform();tp_shape(f->Shape());return;}
 if(op==7||op==8){occ::handle<ShapeFix_FixSmallSolid> f=new ShapeFix_FixSmallSolid;f->SetVolumeThreshold(maxtol);
  tp_shape(op==7?f->Remove(s,ctx):f->Merge(s,ctx));return;}
 if(op==9){sfs_shells(s);return;}
 if(op==13){sfs_small_faces(s,prec);return;}
 if(op==14||op==15){sfs_strips(s,prec,op==15);return;}
 if(op==16){sfs_small_face_checks(s,prec);return;}
 if(op==17||op==18){sfs_free_properties(s,maxtol,op==18);return;}
 if(op==32){sfs_split_tool(s,prec);return;}
 if(op>=19){sfs_wire_fixes(s,prec,maxtol,op);return;}
 sfs_solids(s);}
TopoDS_Shape sfs_input(GaReader& r){
 int family=r.integer();
 if(family==0)return su_input(r);
 if(family==1)return sff_input(r);
 if(family==2)return sfs_solid(r);
 if(family==3)return sfs_bezier_face(r);
 if(family==4)return sfs_raw_face(r);
 return sfs_sphere_face(r);}
void sh_fix_shape_case(std::istream& input){
 GaReader r{input};TopoDS_Shape s;GA_TRY(s=sfs_input(r))
 double prec=r.real(),maxtol=r.real();int op=r.integer();
 if(!s.IsNull())GA_TRY(sfs_run(s,prec,maxtol,op))
 ga_end();}
