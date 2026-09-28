// TKGeomBase oracle: the gce package (tkgeombase_probe.cxx). Each request names one class;
// its first number selects the constructor overload (in header order) and the arguments
// follow. A gce_Root algorithm prints its status, IsDone, IsError, Value() and Operator()
// (the conversion operator is Operator()); a transformation maker prints Value() and
// Operator(). The algorithm is heap-allocated, as `new` zeroes it (Standard::Allocate is
// calloc), so the paths whose constructor never assigns the status read gce_Done.
#include <gce_MakeCirc.hxx>
#include <gce_MakeCirc2d.hxx>
#include <gce_MakeCone.hxx>
#include <gce_MakeCylinder.hxx>
#include <gce_MakeDir.hxx>
#include <gce_MakeDir2d.hxx>
#include <gce_MakeElips.hxx>
#include <gce_MakeElips2d.hxx>
#include <gce_MakeHypr.hxx>
#include <gce_MakeHypr2d.hxx>
#include <gce_MakeLin.hxx>
#include <gce_MakeLin2d.hxx>
#include <gce_MakeMirror.hxx>
#include <gce_MakeMirror2d.hxx>
#include <gce_MakeParab.hxx>
#include <gce_MakeParab2d.hxx>
#include <gce_MakePln.hxx>
#include <gce_MakeRotation.hxx>
#include <gce_MakeRotation2d.hxx>
#include <gce_MakeScale.hxx>
#include <gce_MakeScale2d.hxx>
#include <gce_MakeTranslation.hxx>
#include <gce_MakeTranslation2d.hxx>
template<class M,class P> void gb_gce_root(M* maker,P print){
 std::unique_ptr<M> owned(maker);
 gb_int(maker->Status());gb_flag(maker->IsDone());gb_flag(maker->IsError());
 GB_TRY(print(maker->Value()));GB_TRY(print(maker->Operator()));
}
template<class M,class P> void gb_gce_trsf(M* maker,P print){
 std::unique_ptr<M> owned(maker);print(maker->Value());print(maker->Operator());
}
// Builds the maker of overload `k` inside GB_TRY, so a raising constructor prints its type.
#define GB_GCE(PRINT,...) GB_TRY(gb_gce_root(new __VA_ARGS__,PRINT))
#define GB_GCE_TRSF(PRINT,...) GB_TRY(gb_gce_trsf(new __VA_ARGS__,PRINT))
// gce_circ k: 0 (A2, R) | 1 (Circ(A2, R0), Dist) | 2 (Circ(A2, R0), P) | 3 (P1, P2, P3) |
// 4 (Center, Norm, R) | 5 (Center, Pln(P, N), R) | 6 (Center, Ptaxis, R) | 7 (Ax1, R).
void gb_gce_circ_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(a,d));}
 if(k==1||k==2){gp_Ax2 a=r.ax2();double r0=r.real();gp_Circ c(a,r0);
  if(k==1){double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(c,d));}
  else{gp_Pnt p=r.pnt();GB_GCE(gb_circ,gce_MakeCirc(c,p));}}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GCE(gb_circ,gce_MakeCirc(a,b,c));}
 if(k==4){gp_Pnt c=r.pnt();gp_Dir n=r.dir();double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(c,n,d));}
 if(k==5){gp_Pnt c=r.pnt();gp_Pnt o=r.pnt();gp_Dir n=r.dir();double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(c,gp_Pln(o,n),d));}
 if(k==6){gp_Pnt c=r.pnt(),p=r.pnt();double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(c,p,d));}
 if(k==7){gp_Ax1 a=r.ax1();double d=r.real();GB_GCE(gb_circ,gce_MakeCirc(a,d));}
 gb_end();
}
// gce_circ2d k: 0 (Ax2d, R, Sense) | 1 (Ax22d, R) | 2 (Circ(Ax22d, R0), Dist) |
// 3 (Circ(Ax22d, R0), P) | 4 (P1, P2, P3) | 5 (Center, R, Sense) | 6 (Center, P, Sense).
void gb_gce_circ2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2d a=r.ax2d();double d=r.real();bool s=r.flag();GB_GCE(gb_circ2d,gce_MakeCirc2d(a,d,s));}
 if(k==1){gp_Ax22d a=r.ax22d();double d=r.real();GB_GCE(gb_circ2d,gce_MakeCirc2d(a,d));}
 if(k==2||k==3){gp_Ax22d a=r.ax22d();double r0=r.real();gp_Circ2d c(a,r0);
  if(k==2){double d=r.real();GB_GCE(gb_circ2d,gce_MakeCirc2d(c,d));}
  else{gp_Pnt2d p=r.pnt2d();GB_GCE(gb_circ2d,gce_MakeCirc2d(c,p));}}
 if(k==4){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GCE(gb_circ2d,gce_MakeCirc2d(a,b,c));}
 if(k==5){gp_Pnt2d c=r.pnt2d();double d=r.real();bool s=r.flag();GB_GCE(gb_circ2d,gce_MakeCirc2d(c,d,s));}
 if(k==6){gp_Pnt2d c=r.pnt2d(),p=r.pnt2d();bool s=r.flag();GB_GCE(gb_circ2d,gce_MakeCirc2d(c,p,s));}
 gb_end();
}
// gce_cone k: 0 (A2, Ang, R) | 1 (Cone(Ax3, Ang0, R0), P) | 2 (Cone(Ax3, Ang0, R0), Dist) |
// 3 (P1, P2, P3, P4) | 4 (Ax1, P1, P2) | 5 (Lin(Ax1), P1, P2) | 6 (P1, P2, R1, R2).
void gb_gce_cone_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double g=r.real(),d=r.real();GB_GCE(gb_cone,gce_MakeCone(a,g,d));}
 if(k==1||k==2){gp_Ax3 a=r.ax3();double g=r.real(),r0=r.real();gp_Cone c(a,g,r0);
  if(k==1){gp_Pnt p=r.pnt();GB_GCE(gb_cone,gce_MakeCone(c,p));}
  else{double d=r.real();GB_GCE(gb_cone,gce_MakeCone(c,d));}}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt(),d=r.pnt();GB_GCE(gb_cone,gce_MakeCone(a,b,c,d));}
 if(k==4){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt(),q=r.pnt();GB_GCE(gb_cone,gce_MakeCone(a,p,q));}
 if(k==5){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt(),q=r.pnt();GB_GCE(gb_cone,gce_MakeCone(gp_Lin(a),p,q));}
 if(k==6){gp_Pnt a=r.pnt(),b=r.pnt();double r1=r.real(),r2=r.real();GB_GCE(gb_cone,gce_MakeCone(a,b,r1,r2));}
 gb_end();
}
// gce_cylinder k: 0 (A2, R) | 1 (Cyl(Ax3, R0), P) | 2 (Cyl(Ax3, R0), Dist) | 3 (P1, P2, P3) |
// 4 (Ax1, R) | 5 (Circ(A2, R0)).
void gb_gce_cylinder_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double d=r.real();GB_GCE(gb_cylinder,gce_MakeCylinder(a,d));}
 if(k==1||k==2){gp_Ax3 a=r.ax3();double r0=r.real();gp_Cylinder c(a,r0);
  if(k==1){gp_Pnt p=r.pnt();GB_GCE(gb_cylinder,gce_MakeCylinder(c,p));}
  else{double d=r.real();GB_GCE(gb_cylinder,gce_MakeCylinder(c,d));}}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GCE(gb_cylinder,gce_MakeCylinder(a,b,c));}
 if(k==4){gp_Ax1 a=r.ax1();double d=r.real();GB_GCE(gb_cylinder,gce_MakeCylinder(a,d));}
 if(k==5){gp_Ax2 a=r.ax2();double r0=r.real();GB_GCE(gb_cylinder,gce_MakeCylinder(gp_Circ(a,r0)));}
 gb_end();
}
// gce_dir k: 0 (Vec) | 1 (XYZ) | 2 (X, Y, Z) | 3 (P1, P2).
void gb_gce_dir_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec v=r.vec();GB_GCE(gb_dir,gce_MakeDir(v));}
 if(k==1){gp_XYZ v=r.xyz();GB_GCE(gb_dir,gce_MakeDir(v));}
 if(k==2){double x=r.real(),y=r.real(),z=r.real();GB_GCE(gb_dir,gce_MakeDir(x,y,z));}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt();GB_GCE(gb_dir,gce_MakeDir(a,b));}
 gb_end();
}
// gce_dir2d k: 0 (Vec2d) | 1 (XY) | 2 (X, Y) | 3 (P1, P2).
void gb_gce_dir2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec2d v=r.vec2d();GB_GCE(gb_dir2d,gce_MakeDir2d(v));}
 if(k==1){gp_XY v=r.xy();GB_GCE(gb_dir2d,gce_MakeDir2d(v));}
 if(k==2){double x=r.real(),y=r.real();GB_GCE(gb_dir2d,gce_MakeDir2d(x,y));}
 if(k==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GCE(gb_dir2d,gce_MakeDir2d(a,b));}
 gb_end();
}
// gce_elips k: 0 (A2, Major, Minor) | 1 (S1, S2, Center).
void gb_gce_elips_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double m=r.real(),n=r.real();GB_GCE(gb_elips,gce_MakeElips(a,m,n));}
 if(k==1){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GCE(gb_elips,gce_MakeElips(a,b,c));}
 gb_end();
}
// gce_elips2d k: 0 (Ax2d, Major, Minor, Sense) | 1 (Ax22d, Major, Minor) | 2 (S1, S2, Center).
void gb_gce_elips2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2d a=r.ax2d();double m=r.real(),n=r.real();bool s=r.flag();GB_GCE(gb_elips2d,gce_MakeElips2d(a,m,n,s));}
 if(k==1){gp_Ax22d a=r.ax22d();double m=r.real(),n=r.real();GB_GCE(gb_elips2d,gce_MakeElips2d(a,m,n));}
 if(k==2){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GCE(gb_elips2d,gce_MakeElips2d(a,b,c));}
 gb_end();
}
// gce_hypr k: 0 (A2, Major, Minor) | 1 (S1, S2, Center).
void gb_gce_hypr_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double m=r.real(),n=r.real();GB_GCE(gb_hypr,gce_MakeHypr(a,m,n));}
 if(k==1){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GCE(gb_hypr,gce_MakeHypr(a,b,c));}
 gb_end();
}
// gce_hypr2d k: 0 (S1, S2, Center) | 1 (Ax2d, Major, Minor, Sense) | 2 (Ax22d, Major, Minor).
void gb_gce_hypr2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GCE(gb_hypr2d,gce_MakeHypr2d(a,b,c));}
 if(k==1){gp_Ax2d a=r.ax2d();double m=r.real(),n=r.real();bool s=r.flag();GB_GCE(gb_hypr2d,gce_MakeHypr2d(a,m,n,s));}
 if(k==2){gp_Ax22d a=r.ax22d();double m=r.real(),n=r.real();GB_GCE(gb_hypr2d,gce_MakeHypr2d(a,m,n));}
 gb_end();
}
// gce_lin k: 0 (Ax1) | 1 (P, V) | 2 (P1, P2) | 3 (Lin(Ax1), P).
void gb_gce_lin_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax1 a=r.ax1();GB_GCE(gb_lin,gce_MakeLin(a));}
 if(k==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GCE(gb_lin,gce_MakeLin(p,d));}
 if(k==2){gp_Pnt a=r.pnt(),b=r.pnt();GB_GCE(gb_lin,gce_MakeLin(a,b));}
 if(k==3){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt();GB_GCE(gb_lin,gce_MakeLin(gp_Lin(a),p));}
 gb_end();
}
// gce_lin2d k: 0 (Ax2d) | 1 (P, V) | 2 (A, B, C) | 3 (Lin(Ax2d), Dist) | 4 (Lin(Ax2d), P) | 5 (P1, P2).
void gb_gce_lin2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2d a=r.ax2d();GB_GCE(gb_lin2d,gce_MakeLin2d(a));}
 if(k==1){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();GB_GCE(gb_lin2d,gce_MakeLin2d(p,d));}
 if(k==2){double a=r.real(),b=r.real(),c=r.real();GB_GCE(gb_lin2d,gce_MakeLin2d(a,b,c));}
 if(k==3){gp_Ax2d a=r.ax2d();double d=r.real();GB_GCE(gb_lin2d,gce_MakeLin2d(gp_Lin2d(a),d));}
 if(k==4){gp_Ax2d a=r.ax2d();gp_Pnt2d p=r.pnt2d();GB_GCE(gb_lin2d,gce_MakeLin2d(gp_Lin2d(a),p));}
 if(k==5){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GCE(gb_lin2d,gce_MakeLin2d(a,b));}
 gb_end();
}
// gce_parab k: 0 (A2, Focal) | 1 (D, F).
void gb_gce_parab_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double f=r.real();GB_GCE(gb_parab,gce_MakeParab(a,f));}
 if(k==1){gp_Ax1 a=r.ax1();gp_Pnt f=r.pnt();GB_GCE(gb_parab,gce_MakeParab(a,f));}
 gb_end();
}
// gce_parab2d k: 0 (MirrorAxis, Focal, Sense) | 1 (Ax22d, Focal) | 2 (D, F, Sense) | 3 (S, Center, Sense).
void gb_gce_parab2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2d a=r.ax2d();double f=r.real();bool s=r.flag();GB_GCE(gb_parab2d,gce_MakeParab2d(a,f,s));}
 if(k==1){gp_Ax22d a=r.ax22d();double f=r.real();GB_GCE(gb_parab2d,gce_MakeParab2d(a,f));}
 if(k==2){gp_Ax2d a=r.ax2d();gp_Pnt2d f=r.pnt2d();bool s=r.flag();GB_GCE(gb_parab2d,gce_MakeParab2d(a,f,s));}
 if(k==3){gp_Pnt2d a=r.pnt2d(),c=r.pnt2d();bool s=r.flag();GB_GCE(gb_parab2d,gce_MakeParab2d(a,c,s));}
 gb_end();
}
// gce_pln k: 0 (A2) | 1 (P, V) | 2 (A, B, C, D) | 3 (Pln(Ax3), P) | 4 (Pln(Ax3), Dist) |
// 5 (P1, P2, P3) | 6 (P1, P2) | 7 (Ax1).
void gb_gce_pln_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();GB_GCE(gb_pln,gce_MakePln(a));}
 if(k==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GCE(gb_pln,gce_MakePln(p,d));}
 if(k==2){double a=r.real(),b=r.real(),c=r.real(),d=r.real();GB_GCE(gb_pln,gce_MakePln(a,b,c,d));}
 if(k==3){gp_Ax3 a=r.ax3();gp_Pnt p=r.pnt();GB_GCE(gb_pln,gce_MakePln(gp_Pln(a),p));}
 if(k==4){gp_Ax3 a=r.ax3();double d=r.real();GB_GCE(gb_pln,gce_MakePln(gp_Pln(a),d));}
 if(k==5){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GCE(gb_pln,gce_MakePln(a,b,c));}
 if(k==6){gp_Pnt a=r.pnt(),b=r.pnt();GB_GCE(gb_pln,gce_MakePln(a,b));}
 if(k==7){gp_Ax1 a=r.ax1();GB_GCE(gb_pln,gce_MakePln(a));}
 gb_end();
}
// gce_mirror k: 0 (P) | 1 (Ax1) | 2 (Lin(Ax1)) | 3 (P, D) | 4 (Pln(Ax3)) | 5 (Ax2).
void gb_gce_mirror_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt p=r.pnt();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(p));}
 if(k==1){gp_Ax1 a=r.ax1();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(a));}
 if(k==2){gp_Ax1 a=r.ax1();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(gp_Lin(a)));}
 if(k==3){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(p,d));}
 if(k==4){gp_Ax3 a=r.ax3();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(gp_Pln(a)));}
 if(k==5){gp_Ax2 a=r.ax2();GB_GCE_TRSF(gb_trsf,gce_MakeMirror(a));}
 gb_end();
}
// gce_mirror2d k: 0 (P) | 1 (Ax2d) | 2 (Lin(Ax2d)) | 3 (P, D).
void gb_gce_mirror2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt2d p=r.pnt2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeMirror2d(p));}
 if(k==1){gp_Ax2d a=r.ax2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeMirror2d(a));}
 if(k==2){gp_Ax2d a=r.ax2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeMirror2d(gp_Lin2d(a)));}
 if(k==3){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeMirror2d(p,d));}
 gb_end();
}
// gce_rotation k: 0 (Lin(Ax1), Angle) | 1 (Ax1, Angle) | 2 (P, D, Angle);
// gce_rotation2d (P, Angle); gce_scale (P, S); gce_scale2d (P, S).
void gb_gce_rotation_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax1 a=r.ax1();double g=r.real();GB_GCE_TRSF(gb_trsf,gce_MakeRotation(gp_Lin(a),g));}
 if(k==1){gp_Ax1 a=r.ax1();double g=r.real();GB_GCE_TRSF(gb_trsf,gce_MakeRotation(a,g));}
 if(k==2){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double g=r.real();GB_GCE_TRSF(gb_trsf,gce_MakeRotation(p,d,g));}
 gb_end();
}
void gb_gce_rotation2d_case(std::istream& in){
 GbReader r{in};gp_Pnt2d p=r.pnt2d();double g=r.real();GB_GCE_TRSF(gb_trsf2d,gce_MakeRotation2d(p,g));gb_end();
}
void gb_gce_scale_case(std::istream& in){
 GbReader r{in};gp_Pnt p=r.pnt();double s=r.real();GB_GCE_TRSF(gb_trsf,gce_MakeScale(p,s));gb_end();
}
void gb_gce_scale2d_case(std::istream& in){
 GbReader r{in};gp_Pnt2d p=r.pnt2d();double s=r.real();GB_GCE_TRSF(gb_trsf2d,gce_MakeScale2d(p,s));gb_end();
}
// gce_translation k: 0 (Vec) | 1 (P1, P2); gce_translation2d likewise.
void gb_gce_translation_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec v=r.vec();GB_GCE_TRSF(gb_trsf,gce_MakeTranslation(v));}
 if(k==1){gp_Pnt a=r.pnt(),b=r.pnt();GB_GCE_TRSF(gb_trsf,gce_MakeTranslation(a,b));}
 gb_end();
}
void gb_gce_translation2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec2d v=r.vec2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeTranslation2d(v));}
 if(k==1){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GCE_TRSF(gb_trsf2d,gce_MakeTranslation2d(a,b));}
 gb_end();
}
// gce_status: every gce_ErrorType value, to pin the enumerator values.
void gb_gce_status_case(std::istream&){
 for(int e=gce_Done;e<=gce_BadEquation;++e)gb_int(e);
 gb_end();
}
