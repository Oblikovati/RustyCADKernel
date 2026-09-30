// BRepPrimAPI and BRepPreviewAPI primitive probe operations; the Rust twin is
// crates/tkprim/examples/pr_probe/brep_prim_api.rs.
//
// Records (every number read through GaReader; frames are pr_ax2 records):
// - box call `op ...`: 0 Shape, 1 Shell, 2 Solid, 3..8 BottomFace, BackFace, FrontFace,
//   LeftFace, RightFace, TopFace, 9 Build, 10 IsDone, 11 Init(dx, dy, dz),
//   12 Init(P, dx, dy, dz), 13 Init(P1, P2), 14 Init(frame, dx, dy, dz), 15 the wedge
//   state (pr_wedge_state of Wedge());
// - wedge call `op`: 0 Shape, 1 Shell, 2 Solid, 3 Build, 4 IsDone, 5 the wedge state;
// - one-axis call `op`: 0 Shape, 1 Face, 2 Shell, 3 Solid, 4 Build, 5 IsDone, 6 Angle,
//   VMin, VMax and Axes of OneAxis(), 7 the same through the typed accessor (Cylinder(),
//   Cone(), Sphere(), Torus(), Revolution()).
// Each call prints its shape (pr_shape) or flag, or the exception it raises.
#include <BRepPreviewAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeRevolution.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepPrimAPI_MakeWedge.hxx>
// One box call; the arguments are read before the call.
void pr_pa_box_call(BRepPrimAPI_MakeBox& b,GaReader& r,int op){
 double v[4]={0.,0.,0.,0.};gp_Pnt p1,p2;gp_Pnt ap;gp_Dir an;gp_XYZ ax;
 if(op==11)for(int i=0;i<3;++i)v[i]=r.real();
 if(op==12){p1=r.pnt();for(int i=0;i<3;++i)v[i]=r.real();}
 if(op==13){p1=r.pnt();p2=r.pnt();}
 if(op==14){ap=r.pnt();an=r.dir();ax=r.xyz();for(int i=0;i<3;++i)v[i]=r.real();}
 GA_TRY(
  switch(op){
   case 0:pr_shape(b.Shape());break;case 1:pr_shape(b.Shell());break;case 2:pr_shape(b.Solid());break;
   case 3:pr_shape(b.BottomFace());break;case 4:pr_shape(b.BackFace());break;
   case 5:pr_shape(b.FrontFace());break;case 6:pr_shape(b.LeftFace());break;
   case 7:pr_shape(b.RightFace());break;case 8:pr_shape(b.TopFace());break;
   case 9:b.Build();break;case 10:ga_flag(b.IsDone());break;
   case 11:b.Init(v[0],v[1],v[2]);break;case 12:b.Init(p1,v[0],v[1],v[2]);break;
   case 13:b.Init(p1,p2);break;
   case 14:b.Init(ax.Modulus()==0.?gp_Ax2(ap,an):gp_Ax2(ap,an,gp_Dir(ax)),v[0],v[1],v[2]);break;
   default:pr_wedge_state(b.Wedge());break;})}
void pr_pa_box_calls(BRepPrimAPI_MakeBox& b,GaReader& r){
 int n=r.integer();for(int i=0;i<n;++i)pr_pa_box_call(b,r,r.integer());}
// `kind [values] ncalls (call)...`: kind 0 MakeBox(), 1 MakeBox(dx, dy, dz),
// 2 MakeBox(P, dx, dy, dz), 3 MakeBox(P1, P2), 4 MakeBox(frame, dx, dy, dz).
void pr_pa_box_case(std::istream& input){
 GaReader r{input};int kind=r.integer();std::unique_ptr<BRepPrimAPI_MakeBox> b;
 try{
  if(kind==0)b.reset(new BRepPrimAPI_MakeBox());
  else if(kind==1){double dx=r.real(),dy=r.real();b.reset(new BRepPrimAPI_MakeBox(dx,dy,r.real()));}
  else if(kind==2){gp_Pnt p=r.pnt();double dx=r.real(),dy=r.real();b.reset(new BRepPrimAPI_MakeBox(p,dx,dy,r.real()));}
  else if(kind==3){gp_Pnt p1=r.pnt();b.reset(new BRepPrimAPI_MakeBox(p1,r.pnt()));}
  else{gp_Ax2 a=pr_ax2(r);double dx=r.real(),dy=r.real();b.reset(new BRepPrimAPI_MakeBox(a,dx,dy,r.real()));}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 pr_pa_box_calls(*b,r);
 ga_end();}
// `ncalls (call)...`: the preview box of BRepPreviewAPI_MakeBox(), set by the Init calls.
void pr_pv_box_case(std::istream& input){
 GaReader r{input};BRepPreviewAPI_MakeBox b;pr_pa_box_calls(b,r);ga_end();}
// `kind [frame] values... ncalls (op)...`: kind 0 MakeWedge(dx, dy, dz, ltx),
// 1 MakeWedge(frame, dx, dy, dz, ltx), 2 MakeWedge(dx, dy, dz, xmin, zmin, xmax, zmax),
// 3 MakeWedge(frame, dx, dy, dz, xmin, zmin, xmax, zmax).
void pr_pa_wedge_case(std::istream& input){
 GaReader r{input};int kind=r.integer();std::unique_ptr<BRepPrimAPI_MakeWedge> w;
 try{
  gp_Ax2 a;if(kind==1||kind==3)a=pr_ax2(r);
  double v[7];int n=kind<2?4:7;for(int i=0;i<n;++i)v[i]=r.real();
  if(kind==0)w.reset(new BRepPrimAPI_MakeWedge(v[0],v[1],v[2],v[3]));
  else if(kind==1)w.reset(new BRepPrimAPI_MakeWedge(a,v[0],v[1],v[2],v[3]));
  else if(kind==2)w.reset(new BRepPrimAPI_MakeWedge(v[0],v[1],v[2],v[3],v[4],v[5],v[6]));
  else w.reset(new BRepPrimAPI_MakeWedge(a,v[0],v[1],v[2],v[3],v[4],v[5],v[6]));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();
  GA_TRY(
   switch(op){
    case 0:pr_shape(w->Shape());break;case 1:pr_shape(w->Shell());break;case 2:pr_shape(w->Solid());break;
    case 3:w->Build();break;case 4:ga_flag(w->IsDone());break;default:pr_wedge_state(w->Wedge());break;})}
 ga_end();}
// Angle, VMin, VMax and Axes of a one-axis algorithm.
void pr_pa_oneaxis_state(BRepPrim_OneAxis& p){ga_real(p.Angle());ga_real(p.VMin());ga_real(p.VMax());pr_print_ax2(p.Axes());}
// The typed accessor of a one-axis maker.
BRepPrim_OneAxis& pr_pa_typed(BRepPrimAPI_MakeOneAxis& m,int maker){
 if(maker==0)return dynamic_cast<BRepPrimAPI_MakeCylinder&>(m).Cylinder();
 if(maker==1)return dynamic_cast<BRepPrimAPI_MakeCone&>(m).Cone();
 if(maker==2)return dynamic_cast<BRepPrimAPI_MakeSphere&>(m).Sphere();
 if(maker==3)return dynamic_cast<BRepPrimAPI_MakeTorus&>(m).Torus();
 return dynamic_cast<BRepPrimAPI_MakeRevolution&>(m).Revolution();}
// The makers of `maker kind values...`: maker 0 cylinder (kinds (R, H), (R, H, angle),
// (frame, R, H), (frame, R, H, angle)), 1 cone (the same with R1, R2, H), 2 sphere
// (kinds 0..3 at the origin: (R), (R, angle), (R, a1, a2), (R, a1, a2, a3); 4..7 the same
// after a center P; 8..11 after a frame), 3 torus (kinds 0..3: (R1, R2), + angle,
// + (a1, a2), + (a1, a2, angle); 4..7 after a frame), 4 revolution (kinds 0..3: (M),
// (M, angle), (M, vmin, vmax), (M, vmin, vmax, angle) with M a Geom curve record; 4..7
// after a frame).
BRepPrimAPI_MakeOneAxis* pr_pa_cylinder(GaReader& r,int kind){
 gp_Ax2 a;if(kind>=2)a=pr_ax2(r);double radius=r.real(),h=r.real();
 if(kind==0)return new BRepPrimAPI_MakeCylinder(radius,h);
 if(kind==2)return new BRepPrimAPI_MakeCylinder(a,radius,h);
 double angle=r.real();
 if(kind==1)return new BRepPrimAPI_MakeCylinder(radius,h,angle);
 return new BRepPrimAPI_MakeCylinder(a,radius,h,angle);}
BRepPrimAPI_MakeOneAxis* pr_pa_cone(GaReader& r,int kind){
 gp_Ax2 a;if(kind>=2)a=pr_ax2(r);double r1=r.real(),r2=r.real(),h=r.real();
 if(kind==0)return new BRepPrimAPI_MakeCone(r1,r2,h);
 if(kind==2)return new BRepPrimAPI_MakeCone(a,r1,r2,h);
 double angle=r.real();
 if(kind==1)return new BRepPrimAPI_MakeCone(r1,r2,h,angle);
 return new BRepPrimAPI_MakeCone(a,r1,r2,h,angle);}
BRepPrimAPI_MakeOneAxis* pr_pa_sphere(GaReader& r,int kind){
 gp_Ax2 a;gp_Pnt c;if(kind>=8)a=pr_ax2(r);else if(kind>=4)c=r.pnt();
 int n=kind%4;double radius=r.real();double v[3];for(int i=0;i<n;++i)v[i]=r.real();
 if(kind<4){
  if(n==0)return new BRepPrimAPI_MakeSphere(radius);
  if(n==1)return new BRepPrimAPI_MakeSphere(radius,v[0]);
  if(n==2)return new BRepPrimAPI_MakeSphere(radius,v[0],v[1]);
  return new BRepPrimAPI_MakeSphere(radius,v[0],v[1],v[2]);}
 if(kind<8){
  if(n==0)return new BRepPrimAPI_MakeSphere(c,radius);
  if(n==1)return new BRepPrimAPI_MakeSphere(c,radius,v[0]);
  if(n==2)return new BRepPrimAPI_MakeSphere(c,radius,v[0],v[1]);
  return new BRepPrimAPI_MakeSphere(c,radius,v[0],v[1],v[2]);}
 if(n==0)return new BRepPrimAPI_MakeSphere(a,radius);
 if(n==1)return new BRepPrimAPI_MakeSphere(a,radius,v[0]);
 if(n==2)return new BRepPrimAPI_MakeSphere(a,radius,v[0],v[1]);
 return new BRepPrimAPI_MakeSphere(a,radius,v[0],v[1],v[2]);}
BRepPrimAPI_MakeOneAxis* pr_pa_torus(GaReader& r,int kind){
 gp_Ax2 a;if(kind>=4)a=pr_ax2(r);
 int n=kind%4;double r1=r.real(),r2=r.real();double v[3];for(int i=0;i<n;++i)v[i]=r.real();
 if(kind<4){
  if(n==0)return new BRepPrimAPI_MakeTorus(r1,r2);
  if(n==1)return new BRepPrimAPI_MakeTorus(r1,r2,v[0]);
  if(n==2)return new BRepPrimAPI_MakeTorus(r1,r2,v[0],v[1]);
  return new BRepPrimAPI_MakeTorus(r1,r2,v[0],v[1],v[2]);}
 if(n==0)return new BRepPrimAPI_MakeTorus(a,r1,r2);
 if(n==1)return new BRepPrimAPI_MakeTorus(a,r1,r2,v[0]);
 if(n==2)return new BRepPrimAPI_MakeTorus(a,r1,r2,v[0],v[1]);
 return new BRepPrimAPI_MakeTorus(a,r1,r2,v[0],v[1],v[2]);}
BRepPrimAPI_MakeOneAxis* pr_pa_revolution(GaReader& r,int kind){
 gp_Ax2 a;if(kind>=4)a=pr_ax2(r);occ::handle<Geom_Curve> m=tp_geom_curve(r);
 int n=kind%4;double v[3];for(int i=0;i<n;++i)v[i]=r.real();
 if(kind<4){
  if(n==0)return new BRepPrimAPI_MakeRevolution(m);
  if(n==1)return new BRepPrimAPI_MakeRevolution(m,v[0]);
  if(n==2)return new BRepPrimAPI_MakeRevolution(m,v[0],v[1]);
  return new BRepPrimAPI_MakeRevolution(m,v[0],v[1],v[2]);}
 if(n==0)return new BRepPrimAPI_MakeRevolution(a,m);
 if(n==1)return new BRepPrimAPI_MakeRevolution(a,m,v[0]);
 if(n==2)return new BRepPrimAPI_MakeRevolution(a,m,v[0],v[1]);
 return new BRepPrimAPI_MakeRevolution(a,m,v[0],v[1],v[2]);}
// `maker kind values... ncalls (op)...`.
void pr_pa_oneaxis_case(std::istream& input){
 GaReader r{input};int maker=r.integer(),kind=r.integer();std::unique_ptr<BRepPrimAPI_MakeOneAxis> m;
 try{
  if(maker==0)m.reset(pr_pa_cylinder(r,kind));
  else if(maker==1)m.reset(pr_pa_cone(r,kind));
  else if(maker==2)m.reset(pr_pa_sphere(r,kind));
  else if(maker==3)m.reset(pr_pa_torus(r,kind));
  else m.reset(pr_pa_revolution(r,kind));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();
  GA_TRY(
   switch(op){
    case 0:pr_shape(m->Shape());break;case 1:pr_shape(m->Face());break;case 2:pr_shape(m->Shell());break;
    case 3:pr_shape(m->Solid());break;case 4:m->Build();break;case 5:ga_flag(m->IsDone());break;
    case 6:pr_pa_oneaxis_state(*(BRepPrim_OneAxis*)m->OneAxis());break;
    default:pr_pa_oneaxis_state(pr_pa_typed(*m,maker));break;})}
 ga_end();}
