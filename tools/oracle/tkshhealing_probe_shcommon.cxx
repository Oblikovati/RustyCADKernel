// Readers and printers shared by the ShapeCustom, ShapeUpgrade and ShapeProcess probe
// operations; the Rust twin is crates/tkshhealing/examples/sh_probe/sh_common.rs.
//
// A geometry prelude `n (record)...` adds to the tables of a TKTopAlgo shape program,
// before its ops run (tp_shapes_into), geometry the ops cannot describe:
//  0 elementary: type P N X yrev params (type 0 plane; 1 cylinder R; 2 sphere R;
//    3 cone angle R; 4 torus R r) on gp_Ax3(P, N, X), Y-reversed (indirect) when yrev;
//  1 trimmed s u1 u2 v1 v2: Geom_RectangularTrimmedSurface of surface s;
//  2 offset s d: Geom_OffsetSurface of surface s;
//  3 location (trsf): a location of the transformation record.
// A transformation record `kind ...`: 0 translation V; 1 rotation P D angle; 2 scale P s;
// 3 mirror about point P; 4 mirror about axis P D; 5 mirror about plane P N; 6 product
// (trsf) (trsf) (the first multiplied by the second).
// The digests: a surface (dynamic type, then bounds, periodicity and points at three
// parameter pairs and D1 at a fourth); a 3d or 2d curve (dynamic type, range, points at
// its finite ends, middle and at 0.3, D1 at 0.3); a location (identity flag, then the 12
// values of its transformation).
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <gp_Ax3.hxx>
gp_Trsf shc_trsf(GaReader& r){
 int kind=r.integer();gp_Trsf T;
 if(kind==0){T.SetTranslation(gp_Vec(r.xyz()));}
 else if(kind==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();T.SetRotation(gp_Ax1(p,d),r.real());}
 else if(kind==2){gp_Pnt p=r.pnt();T.SetScale(p,r.real());}
 else if(kind==3){T.SetMirror(r.pnt());}
 else if(kind==4){gp_Pnt p=r.pnt();T.SetMirror(gp_Ax1(p,r.dir()));}
 else if(kind==5){gp_Pnt p=r.pnt();T.SetMirror(gp_Ax2(p,r.dir()));}
 else{gp_Trsf a=shc_trsf(r);T=a.Multiplied(shc_trsf(r));}
 return T;}
occ::handle<Geom_Surface> shc_elementary(GaReader& r){
 int type=r.integer();gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();gp_Ax3 frame(p,n,x);
 if(r.flag())frame.YReverse();
 if(type==0)return new Geom_Plane(frame);
 if(type==1)return new Geom_CylindricalSurface(frame,r.real());
 if(type==2)return new Geom_SphericalSurface(frame,r.real());
 if(type==3){double angle=r.real();return new Geom_ConicalSurface(frame,angle,r.real());}
 double major=r.real();return new Geom_ToroidalSurface(frame,major,r.real());}
void shc_prelude(GaReader& r,TpShapes& t){
 int n=r.integer();
 for(int k=0;k<n;++k){int kind=r.integer();
  if(kind==0)t.surfaces.push_back(shc_elementary(r));
  else if(kind==1){int s=r.integer();double u1=r.real(),u2=r.real(),v1=r.real();
   t.surfaces.push_back(new Geom_RectangularTrimmedSurface(t.surfaces.at(s),u1,u2,v1,r.real()));}
  else if(kind==2){int s=r.integer();t.surfaces.push_back(new Geom_OffsetSurface(t.surfaces.at(s),r.real()));}
  else t.locations.push_back(TopLoc_Location(shc_trsf(r)));}}
// The prelude, then the shape program on the same tables.
TpShapes shc_shapes(GaReader& r){TpShapes t;shc_prelude(r,t);tp_shapes_into(r,t);return t;}
void shc_surface(const occ::handle<Geom_Surface>& s){
 tp_type(s);if(s.IsNull())return;
 double u1=0.,u2=0.,v1=0.,v2=0.;s->Bounds(u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2);
 ga_flag(s->IsUPeriodic());ga_flag(s->IsVPeriodic());
 const double uv[3][2]={{0.25,0.5},{1.,-0.5},{-0.7,0.3}};
 for(const auto& p:uv)GA_TRY(ga_pnt(s->Value(p[0],p[1])))
 GA_TRY(gp_Pnt P;gp_Vec du,dv;s->D1(0.3,0.2,P,du,dv);ga_pnt(P);ga_xyz(du.XYZ());ga_xyz(dv.XYZ()))}
void shc_xyz(const gp_Pnt& p){ga_pnt(p);}
void shc_xyz(const gp_Vec& v){ga_xyz(v.XYZ());}
void shc_xyz(const gp_Pnt2d& p){ga_pnt2d(p);}
void shc_xyz(const gp_Vec2d& v){ga_xy(v.XY());}
template<class C> void shc_curve_points(const C& c){
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);
 if(!Precision::IsInfinite(f))GA_TRY(shc_xyz(c->Value(f)))
 if(!Precision::IsInfinite(l))GA_TRY(shc_xyz(c->Value(l)))
 if(!Precision::IsInfinite(f)&&!Precision::IsInfinite(l))GA_TRY(shc_xyz(c->Value(0.5*(f+l))))
 GA_TRY(shc_xyz(c->Value(0.3)))
 GA_TRY(shc_xyz(c->DN(0.3,1)))}
void shc_curve(const occ::handle<Geom_Curve>& c){tp_type(c);if(!c.IsNull())shc_curve_points(c);}
void shc_curve2d(const occ::handle<Geom2d_Curve>& c){tp_type(c);if(!c.IsNull())shc_curve_points(c);}
void shc_location(const TopLoc_Location& l){
 ga_flag(l.IsIdentity());const gp_Trsf& T=l.Transformation();
 for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(T.Value(i,j));}
