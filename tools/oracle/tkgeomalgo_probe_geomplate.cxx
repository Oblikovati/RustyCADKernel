// GeomPlate probe operations (tkgeomalgo_probe.cxx): the scene readers, the constraints,
// the plate surface, the average plane and HalfSpace. The builder and the approximation
// are in tkgeomalgo_probe_geomplate_build.cxx; the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_plate*.rs.
#include <GeomPlate_Aij.hxx>
#include <GeomPlate_BuildAveragePlane.hxx>
#include <GeomPlate_CurveConstraint.hxx>
#include <GeomPlate_PointConstraint.hxx>
#include <GeomPlate_Surface.hxx>
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <GeomLProp_SLProps.hxx>
#include <Law_Linear.hxx>
#include <Plate_PinpointConstraint.hxx>
#include <Plate_Plate.hxx>
#include <ProjLib_HCompProjectedCurve.hxx>
#include <NCollection_Array2.hxx>
// `kind ...`: -1 none, 0 plane, 1 cylinder, 2 sphere (point, axis, radius), 3 Bezier 3x3.
occ::handle<Geom_Surface> ga_gp_surface(GaReader& r){
 int kind=r.integer();if(kind<0)return occ::handle<Geom_Surface>();
 if(kind==3){NCollection_Array2<gp_Pnt> poles(1,3,1,3);for(int i=1;i<=3;i++)for(int j=1;j<=3;j++)poles(i,j)=r.pnt();
  return new Geom_BezierSurface(poles);}
 gp_Pnt p=r.pnt();gp_Dir n=r.dir();double radius=r.real();gp_Ax3 ax(p,n);
 if(kind==0)return new Geom_Plane(ax);
 if(kind==1)return new Geom_CylindricalSurface(ax,radius);
 return new Geom_SphericalSurface(ax,radius);
}
// `kind geometry`: 0 Bezier (n poles), 1 line (p d t0 t1), 2 circle (c n r t0 t1), 3 a 2d
// line (p d t0 t1) on a surface, 4 a 2d Bezier (n poles) on a surface.
occ::handle<Adaptor3d_Curve> ga_gp_curve(GaReader& r,int kind){
 if(kind==0){int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;i++)poles(i)=r.pnt();
  return new GeomAdaptor_Curve(new Geom_BezierCurve(poles));}
 if(kind==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double t0=r.real(),t1=r.real();return new GeomAdaptor_Curve(new Geom_Line(p,d),t0,t1);}
 if(kind==2){gp_Pnt c=r.pnt();gp_Dir n=r.dir();double radius=r.real(),t0=r.real(),t1=r.real();
  return new GeomAdaptor_Curve(new Geom_Circle(gp_Ax2(c,n),radius),t0,t1);}
 occ::handle<Geom_Surface> s=ga_gp_surface(r);occ::handle<Adaptor2d_Curve2d> c2d;
 if(kind==3){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double t0=r.real(),t1=r.real();c2d=new Geom2dAdaptor_Curve(new Geom2d_Line(p,d),t0,t1);}
 else{int n=r.integer();NCollection_Array1<gp_Pnt2d> poles(1,n);for(int i=1;i<=n;i++)poles(i)=r.pnt2d();
  c2d=new Geom2dAdaptor_Curve(new Geom2d_BezierCurve(poles));}
 return new Adaptor3d_CurveOnSurface(c2d,new GeomAdaptor_Surface(s));
}
occ::handle<Law_Function> ga_gp_law(double first,double last,double v0,double v1){
 occ::handle<Law_Linear> law=new Law_Linear();law->Set(first,v0,last,v1);return law;
}
// `kind order npts tolDist tolAng tolCurv crit [a b [c d e f]] geometry`: a curve constraint
// with, for crit 1, a linear G0 criterion law from a to b, and for crit 2 also G1 and G2 laws.
occ::handle<GeomPlate_CurveConstraint> ga_gp_curve_constraint(GaReader& r){
 int kind=r.integer(),order=r.integer(),npts=r.integer();double td=r.real(),ta=r.real(),tc=r.real();int crit=r.integer();
 double law[6]={0,0,0,0,0,0};if(crit>=1){law[0]=r.real();law[1]=r.real();}
 if(crit>=2)for(int i=2;i<6;i++)law[i]=r.real();
 occ::handle<Adaptor3d_Curve> c=ga_gp_curve(r,kind);
 occ::handle<GeomPlate_CurveConstraint> cc=new GeomPlate_CurveConstraint(c,order,npts,td,ta,tc);
 double f=c->FirstParameter(),l=c->LastParameter();
 if(crit>=1)cc->SetG0Criterion(ga_gp_law(f,l,law[0],law[1]));
 if(crit>=2){cc->SetG1Criterion(ga_gp_law(f,l,law[2],law[3]));cc->SetG2Criterion(ga_gp_law(f,l,law[4],law[5]));}
 return cc;
}
// `kind order ... has2d [u v]`: 0 a point (p tolDist), 1 a surface point (u v surface
// tolDist tolAng tolCurv); an optional parameter on the initial surface.
occ::handle<GeomPlate_PointConstraint> ga_gp_point_constraint(GaReader& r,int* kindOut=nullptr){
 int kind=r.integer(),order=r.integer();if(kindOut)*kindOut=kind;occ::handle<GeomPlate_PointConstraint> pc;
 if(kind==0){gp_Pnt p=r.pnt();pc=new GeomPlate_PointConstraint(p,order,r.real());}
 else{double u=r.real(),v=r.real();occ::handle<Geom_Surface> s=ga_gp_surface(r);double td=r.real(),ta=r.real(),tc=r.real();
  pc=new GeomPlate_PointConstraint(u,v,s,order,td,ta,tc);}
 if(r.flag())pc->SetPnt2dOnSurf(r.pnt2d());
 return pc;
}
void ga_gp_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
// The local properties the constraints return: the point, D1U, D1V and the normal status.
void ga_gp_lprop(GeomLProp_SLProps& p){ga_pnt(p.Value());ga_gp_vec(p.D1U());ga_gp_vec(p.D1V());ga_flag(p.IsNormalDefined());}
// `point...`: every member of GeomPlate_PointConstraint, then the setters.
void ga_gp_point_case(std::istream& input){
 GaReader r{input};
 int kind=0;GA_TRY(occ::handle<GeomPlate_PointConstraint> pc=ga_gp_point_constraint(r,&kind);
  ga_int(pc->Order());ga_real(pc->G0Criterion());ga_real(pc->G1Criterion());ga_real(pc->G2Criterion());
  gp_Pnt p;gp_Vec v1,v2,v3,v4,v5;pc->D0(p);ga_pnt(p);pc->D1(p,v1,v2);ga_pnt(p);ga_gp_vec(v1);ga_gp_vec(v2);
  pc->D2(p,v1,v2,v3,v4,v5);ga_pnt(p);ga_gp_vec(v1);ga_gp_vec(v2);ga_gp_vec(v3);ga_gp_vec(v4);ga_gp_vec(v5);
  ga_flag(pc->HasPnt2dOnSurf());ga_pnt2d(pc->Pnt2dOnSurf());
  if(kind==1)GA_TRY(ga_gp_lprop(pc->LPropSurf()));
  pc->SetOrder(r.integer());pc->SetG0Criterion(r.real());pc->SetG1Criterion(r.real());pc->SetG2Criterion(r.real());
  pc->SetPnt2dOnSurf(r.pnt2d());
  ga_int(pc->Order());ga_real(pc->G0Criterion());ga_real(pc->G1Criterion());ga_real(pc->G2Criterion());
  ga_flag(pc->HasPnt2dOnSurf());ga_pnt2d(pc->Pnt2dOnSurf()));
 ga_end();
}
// The 2d curve of a constraint at its ends and middle, or its nullity.
void ga_gp_curve2d(const occ::handle<Geom2d_Curve>& c,double f,double l){
 ga_flag(c.IsNull());if(c.IsNull())return;
 GA_TRY(ga_pnt2d(c->Value(f)));GA_TRY(ga_pnt2d(c->Value(0.5*(f+l))));GA_TRY(ga_pnt2d(c->Value(l)));
}
// `constraint u... projected setters`: every member of GeomPlate_CurveConstraint at the
// parameters u (3 values), then SetProjectedCurve with the projection of the curve on a
// surface (when `projected` is 1: surface tolU tolV), then the setters.
void ga_gp_curve_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<GeomPlate_CurveConstraint> cc=ga_gp_curve_constraint(r);double us[3]={r.real(),r.real(),r.real()};
  ga_int(cc->Order());ga_int(cc->NbPoints());GA_TRY(ga_real(cc->FirstParameter()));GA_TRY(ga_real(cc->LastParameter()));
  GA_TRY(ga_real(cc->Length()));
  for(double u:us){gp_Pnt p;gp_Vec v1,v2,v3,v4,v5;
   GA_TRY(cc->D0(u,p);ga_pnt(p));GA_TRY(cc->D1(u,p,v1,v2);ga_pnt(p);ga_gp_vec(v1);ga_gp_vec(v2));
   GA_TRY(cc->D2(u,p,v1,v2,v3,v4,v5);ga_pnt(p);ga_gp_vec(v3);ga_gp_vec(v4);ga_gp_vec(v5));
   GA_TRY(ga_real(cc->G0Criterion(u)));GA_TRY(ga_real(cc->G1Criterion(u)));GA_TRY(ga_real(cc->G2Criterion(u)));
   GA_TRY(ga_gp_lprop(cc->LPropSurf(u)));}
  ga_flag(cc->Curve3d().IsNull());ga_flag(cc->ProjectedCurve().IsNull());
  if(r.flag()){occ::handle<Geom_Surface> s=ga_gp_surface(r);double tu=r.real(),tv=r.real();
   occ::handle<GeomAdaptor_Surface> hs=new GeomAdaptor_Surface(s);
   occ::handle<ProjLib_HCompProjectedCurve> proj=new ProjLib_HCompProjectedCurve(hs,cc->Curve3d(),tu,tv);
   cc->SetProjectedCurve(proj,tu,tv);ga_flag(cc->ProjectedCurve().IsNull());
   GA_TRY(ga_real(cc->FirstParameter()));GA_TRY(ga_real(cc->LastParameter()));
   GA_TRY(occ::handle<Geom2d_Curve> c2=cc->Curve2dOnSurf();ga_gp_curve2d(c2,proj->FirstParameter(),proj->LastParameter()));}
  cc->SetOrder(r.integer());cc->SetNbPoints(r.integer());ga_int(cc->Order());ga_int(cc->NbPoints());
  occ::handle<Geom2d_Curve> line=new Geom2d_Line(gp_Pnt2d(0.,0.),gp_Dir2d(1.,0.));cc->SetCurve2dOnSurf(line);
  GA_TRY(ga_gp_curve2d(cc->Curve2dOnSurf(),0.,1.)));
 ga_end();
}
// `default`: the empty GeomPlate_CurveConstraint (its criteria without law and its curve
// members dereference null handles natively, so only the laws once set are read).
void ga_gp_curve_empty_case(std::istream&){
 occ::handle<GeomPlate_CurveConstraint> cc=new GeomPlate_CurveConstraint();
 ga_int(cc->Order());ga_int(cc->NbPoints());ga_flag(cc->Curve3d().IsNull());ga_flag(cc->ProjectedCurve().IsNull());
 GA_TRY(ga_flag(cc->Curve2dOnSurf().IsNull()));
 GA_TRY(cc->SetG0Criterion(ga_gp_law(0.,1.,3.,4.)));GA_TRY(ga_real(cc->G0Criterion(0.25)));
 GA_TRY(cc->SetG1Criterion(ga_gp_law(0.,1.,1.,2.)));GA_TRY(ga_real(cc->G1Criterion(0.5)));
 GA_TRY(cc->SetG2Criterion(ga_gp_law(0.,1.,5.,7.)));GA_TRY(ga_real(cc->G2Criterion(0.75)));
 ga_end();
}
// A Plate_Plate of `n` pinpoints `(u v x y z iu iv)` solved at order `order`.
void ga_gp_plate(GaReader& r,Plate_Plate& plate){
 int n=r.integer();for(int i=0;i<n;i++){double u=r.real(),v0=r.real();gp_XY uv(u,v0);gp_XYZ v=r.xyz();int iu=r.integer(),iv=r.integer();
  plate.Load(Plate_PinpointConstraint(uv,v,iu,iv));}
 int order=r.integer();plate.SolveTI(order,1.);
}
void ga_gp_eval(const occ::handle<Geom_Surface>& s,double u,double v){
 GA_TRY(ga_pnt(s->Value(u,v)));
 GA_TRY(gp_Pnt p;gp_Vec d1u,d1v;s->D1(u,v,p,d1u,d1v);ga_pnt(p);ga_gp_vec(d1u);ga_gp_vec(d1v));
 GA_TRY(gp_Pnt p;gp_Vec d1u,d1v,d2u,d2v,d2uv;s->D2(u,v,p,d1u,d1v,d2u,d2v,d2uv);ga_gp_vec(d2u);ga_gp_vec(d2v);ga_gp_vec(d2uv));
}
void ga_gp_bounds(const occ::handle<Geom_Surface>& s){double a,b,c,d;s->Bounds(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);}
// `surface plate bounds(4) params`: a GeomPlate_Surface of the initial surface and the
// plate: every member, on the plate, its copy and a plate surface on it.
void ga_gp_surface_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> init=ga_gp_surface(r);Plate_Plate plate;ga_gp_plate(r,plate);ga_flag(plate.IsDone());
  occ::handle<GeomPlate_Surface> gs=new GeomPlate_Surface(init,plate);double b[4]={r.real(),r.real(),r.real(),r.real()};
  GA_TRY(gs->SetBounds(b[0],b[1],b[2],b[3]));ga_gp_bounds(gs);
  GA_TRY(double a,bb,c,d;gs->RealBounds(a,bb,c,d);ga_real(a);ga_real(bb);ga_real(c);ga_real(d));
  NCollection_Sequence<gp_XY> seq;gs->Constraints(seq);ga_int(seq.Length());for(const gp_XY& xy:seq)ga_xy(xy);
  int n=r.integer();for(int i=0;i<n;i++){double u=r.real(),v=r.real();ga_gp_eval(gs,u,v);}
  GA_TRY(gp_Pnt p;gp_Vec a1,a2,a3,a4,a5,a6,a7,a8,a9;gs->D3(0.,0.,p,a1,a2,a3,a4,a5,a6,a7,a8,a9));
  GA_TRY(ga_gp_vec(gs->DN(0.,0.,1,0)));GA_TRY(ga_flag(gs->IsCNu(1)));GA_TRY(ga_flag(gs->IsCNv(1)));
  ga_flag(gs->IsUClosed());ga_flag(gs->IsVClosed());ga_flag(gs->IsUPeriodic());ga_flag(gs->IsVPeriodic());
  ga_real(gs->UPeriod());ga_real(gs->VPeriod());ga_int(int(gs->Continuity()));
  ga_real(gs->UReversedParameter(0.25));ga_real(gs->VReversedParameter(-0.5));
  ga_flag(gs->UIso(0.5).IsNull());ga_flag(gs->VIso(0.5).IsNull());
  double tu=0.3,tv=0.7;gs->TransformParameters(tu,tv,gp_Trsf());ga_real(tu);ga_real(tv);
  gs->UReverse();gs->VReverse();gs->Transform(gp_Trsf());ga_gp_bounds(gs);
  ga_flag(gs->CallSurfinit()==init);
  occ::handle<GeomPlate_Surface> copy=occ::down_cast<GeomPlate_Surface>(gs->Copy());ga_gp_bounds(copy);
  occ::handle<GeomPlate_Surface> nested=new GeomPlate_Surface(gs,plate);ga_gp_bounds(nested);GA_TRY(ga_gp_eval(nested,0.5,0.5)));
 ga_end();
}
// The plane of an average plane: its location and axes, or the exception.
// The builder lives on the heap, whose memory the default manager clears, so the box of a
// cloud without plane (uninitialized natively) reads zeros on both sides.
void ga_gp_average_out(const GeomPlate_BuildAveragePlane& bap){
 ga_flag(bap.IsPlane());ga_flag(bap.IsLine());
 GA_TRY(occ::handle<Geom_Plane> pl=bap.Plane();ga_flag(pl.IsNull());if(!pl.IsNull()){const gp_Ax3& a=pl->Position();
  ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());ga_dir(a.YDirection());}
  double u1,u2,v1,v2;bap.MinMaxBox(u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2));
 GA_TRY(occ::handle<Geom_Line> ln=bap.Line();ga_flag(ln.IsNull());if(!ln.IsNull()){ga_pnt(ln->Position().Location());ga_dir(ln->Position().Direction());});
}
// `n points nbBound tol popt nopt`: GeomPlate_BuildAveragePlane of a point cloud.
void ga_gp_average_case(std::istream& input){
 GaReader r{input};int n=r.integer();occ::handle<NCollection_HArray1<gp_Pnt>> pts=new NCollection_HArray1<gp_Pnt>(1,n);
 for(int i=1;i<=n;i++)pts->SetValue(i,r.pnt());int nb=r.integer();double tol=r.real();int popt=r.integer(),nopt=r.integer();
 GA_TRY(std::unique_ptr<GeomPlate_BuildAveragePlane> bap(new GeomPlate_BuildAveragePlane(pts,nb,tol,popt,nopt));ga_gp_average_out(*bap));
 ga_end();
}
// `nn normals np points`: GeomPlate_BuildAveragePlane of the best vector of normals.
void ga_gp_normals_case(std::istream& input){
 GaReader r{input};int nn=r.integer();NCollection_Sequence<gp_Vec> normals;for(int i=0;i<nn;i++)normals.Append(gp_Vec(r.xyz()));
 int n=r.integer();occ::handle<NCollection_HArray1<gp_Pnt>> pts=new NCollection_HArray1<gp_Pnt>(1,n);for(int i=1;i<=n;i++)pts->SetValue(i,r.pnt());
 GA_TRY(std::unique_ptr<GeomPlate_BuildAveragePlane> bap(new GeomPlate_BuildAveragePlane(normals,pts));ga_gp_average_out(*bap));
 ga_end();
}
// `linTol angTol steps (n normals)...`: successive HalfSpace calls on one set of normals.
void ga_gp_halfspace_case(std::istream& input){
 GaReader r{input};double lin=r.real(),ang=r.real();int steps=r.integer();NCollection_Sequence<gp_Vec> normals;
 NCollection_Sequence<GeomPlate_Aij> bset;bset.Append(GeomPlate_Aij());bset.Clear();
 for(int s=0;s<steps;s++){int n=r.integer();NCollection_Sequence<gp_Vec> fresh;for(int i=0;i<n;i++)fresh.Append(gp_Vec(r.xyz()));
  GA_TRY(ga_flag(GeomPlate_BuildAveragePlane::HalfSpace(fresh,normals,bset,lin,ang)));
  ga_int(normals.Length());for(const gp_Vec& v:normals)ga_gp_vec(v);ga_int(bset.Length());}
 ga_end();
}
