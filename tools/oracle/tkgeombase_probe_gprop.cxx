// TKGeomBase oracle: the GProp package (tkgeombase_probe.cxx). Each request builds one
// framework and prints every global property it answers (gb_gp_props); the Rust twin is
// crates/tkgeombase/examples/gb_probe/gprop.rs.
#include <GProp.hxx>
#include <GProp_CelGProps.hxx>
#include <GProp_GProps.hxx>
#include <GProp_PEquation.hxx>
#include <GProp_PGProps.hxx>
#include <GProp_PrincipalProps.hxx>
#include <GProp_SelGProps.hxx>
#include <GProp_VelGProps.hxx>
#include <gp_Mat.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
void gb_gp_mat(const gp_Mat& m){for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)gb_real(m.Value(i,j));}
// Every query of a GProp_GProps: mass, centre, matrix of inertia, static moments, the
// moment and radius of gyration about the Z axis and an oblique axis, then the principal
// properties with both symmetry tests (default and 1e-6 relative tolerance).
void gb_gp_props(const GProp_GProps& g){
 gb_real(g.Mass());gb_pnt(g.CentreOfMass());gb_gp_mat(g.MatrixOfInertia());
 double x,y,z;g.StaticMoments(x,y,z);gb_real(x);gb_real(y);gb_real(z);
 const gp_Ax1 axes[2]={gp_Ax1(gp_Pnt(0,0,0),gp_Dir(0,0,1)),gp_Ax1(gp_Pnt(1,2,3),gp_Dir(1,1,1))};
 for(const gp_Ax1& a:axes){gb_real(g.MomentOfInertia(a));gb_real(g.RadiusOfGyration(a));}
 try{
  GProp_PrincipalProps p=g.PrincipalProperties();
  p.Moments(x,y,z);gb_real(x);gb_real(y);gb_real(z);p.RadiusOfGyration(x,y,z);gb_real(x);gb_real(y);gb_real(z);
  gb_vec(p.FirstAxisOfInertia());gb_vec(p.SecondAxisOfInertia());gb_vec(p.ThirdAxisOfInertia());
  gb_flag(p.HasSymmetryAxis());gb_flag(p.HasSymmetryAxis(1e-6));gb_flag(p.HasSymmetryPoint());gb_flag(p.HasSymmetryPoint(1e-6));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
// n points as (x, y, z, density) records.
void gb_gp_points(GbReader& r,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<double>& d){
 for(int i=1;i<=p.Length();++i){p(i)=r.pnt();d(i)=r.real();}
}
// gprop_points: mode 0 unit AddPoint, 1 weighted AddPoint, 2 array constructors (unit and
// weighted), 3 2D arrays of `rows` rows (column-major walk), with the barycentres.
void gb_gprop_points_case(std::istream& in){
 GbReader r{in};const int mode=r.integer(),n=r.integer(),rows=r.integer();
 NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> d(1,n);gb_gp_points(r,p,d);
 try{
  if(mode<=1){GProp_PGProps g;for(int i=1;i<=n;++i){if(mode==0)g.AddPoint(p(i));else g.AddPoint(p(i),d(i));}gb_gp_props(g);}
  if(mode==2){
   gb_gp_props(GProp_PGProps(p));GB_TRY(gb_gp_props(GProp_PGProps(p,d)));
   GB_TRY(gb_pnt(GProp_PGProps::Barycentre(p)));double m;gp_Pnt b;GB_TRY(GProp_PGProps::Barycentre(p,d,m,b);gb_real(m);gb_pnt(b));
  }
  if(mode==3){
   const int cols=n/rows;NCollection_Array2<gp_Pnt> p2(1,rows,1,cols);NCollection_Array2<double> d2(1,rows,1,cols);
   for(int i=1;i<=rows;++i)for(int j=1;j<=cols;++j){p2(i,j)=p((i-1)*cols+j);d2(i,j)=d((i-1)*cols+j);}
   gb_gp_props(GProp_PGProps(p2));GB_TRY(gb_gp_props(GProp_PGProps(p2,d2)));
   gb_pnt(GProp_PGProps::Barycentre(p2));double m;gp_Pnt b;GB_TRY(GProp_PGProps::Barycentre(p2,d2,m,b);gb_real(m);gb_pnt(b));
  }
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// gprop_pequation: GProp_PEquation of n points with a tolerance.
void gb_gprop_pequation_case(std::istream& in){
 GbReader r{in};const int n=r.integer();const double tol=r.real();
 NCollection_Array1<gp_Pnt> p(1,n);for(int i=1;i<=n;++i)p(i)=r.pnt();
 GProp_PEquation e(p,tol);gb_int(int(e.GetType()));
 gb_flag(e.IsPoint());gb_flag(e.IsLinear());gb_flag(e.IsPlanar());gb_flag(e.IsSpace());
 gb_pnt(e.Barycentre());for(int i=1;i<=3;++i){gb_vec(e.PrincipalAxis(i));gb_real(e.Extent(i));}
 GB_TRY(gb_pnt(e.Point()));GB_TRY(gb_lin(e.Line()));GB_TRY(gb_pln(e.Plane()));
 gp_Pnt b;gp_Vec v1,v2,v3;GB_TRY(e.Box(b,v1,v2,v3);gb_pnt(b);gb_vec(v1);gb_vec(v2);gb_vec(v3));
 gb_end();
}
// One elementary solid or surface patch: `perform` builds the framework empty, sets the
// location and calls Perform; otherwise the matching constructor is used.
template<class Props,class Shape>
void gb_gp_patch(const Shape& shape,const double a[4],const gp_Pnt& loc,bool perform){
 if(!perform){gb_gp_props(Props(shape,a[0],a[1],a[2],a[3],loc));return;}
 Props g;g.SetLocation(loc);g.Perform(shape,a[0],a[1],a[2],a[3]);gb_gp_props(g);
}
template<class Props>
void gb_gp_patches(GbReader& r,int shape,const gp_Pnt& loc,bool perform){
 gp_Ax3 ax=r.ax3();double r1=r.real(),r2=0;if(shape==1||shape==3)r2=r.real();
 double a[4];for(double& x:a)x=r.real();
 if(shape==0)gb_gp_patch<Props>(gp_Cylinder(ax,r1),a,loc,perform);
 if(shape==1)gb_gp_patch<Props>(gp_Cone(ax,r1,r2),a,loc,perform);
 if(shape==2)gb_gp_patch<Props>(gp_Sphere(ax,r1),a,loc,perform);
 if(shape==3)gb_gp_patch<Props>(gp_Torus(ax,r1,r2),a,loc,perform);
}
// gprop_elementary: kind 0 whole circle, 1 circle arc, 2 line (CelGProps); 3..6 surface
// cylinder, cone (semi-angle, radius), sphere, torus (SelGProps); 7..10 the same solids
// (VelGProps). `perform` selects SetLocation + Perform instead of the constructor.
void gb_gprop_elementary_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();const bool perform=r.flag();gp_Pnt loc=r.pnt();
 try{
  if(kind<=1){gp_Ax2 ax=r.ax2();double rad=r.real();gp_Circ c(ax,rad);double u1=0,u2=2*M_PI;if(kind==1){u1=r.real();u2=r.real();}
   if(perform){GProp_CelGProps g;g.SetLocation(loc);g.Perform(c,u1,u2);gb_gp_props(g);}
   else if(kind==0)gb_gp_props(GProp_CelGProps(c,loc));else gb_gp_props(GProp_CelGProps(c,u1,u2,loc));}
  if(kind==2){gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());double u1=r.real(),u2=r.real();
   if(perform){GProp_CelGProps g;g.SetLocation(loc);g.Perform(l,u1,u2);gb_gp_props(g);}else gb_gp_props(GProp_CelGProps(l,u1,u2,loc));}
  if(kind>=3&&kind<=6)gb_gp_patches<GProp_SelGProps>(r,kind-3,loc,perform);
  if(kind>=7)gb_gp_patches<GProp_VelGProps>(r,kind-7,loc,perform);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// gprop_compose: a GProp_GProps at `loc` adding point sets (each with its own points,
// then the density of Add); an empty set checks the null-mass branch.
void gb_gprop_compose_case(std::istream& in){
 GbReader r{in};GProp_GProps g(r.pnt());const int items=r.integer();
 for(int k=0;k<items;++k){const int n=r.integer();GProp_PGProps p;for(int i=0;i<n;++i)p.AddPoint(r.pnt());
  const double density=r.real();GB_TRY(g.Add(p,density));}
 gb_gp_props(g);gb_end();
}
// gprop_hoperator: GProp::HOperator(G, Q, Mass).
void gb_gprop_hoperator_case(std::istream& in){
 GbReader r{in};gp_Pnt g=r.pnt(),q=r.pnt();gp_Mat m;GProp::HOperator(g,q,r.real(),m);gb_gp_mat(m);gb_end();
}
