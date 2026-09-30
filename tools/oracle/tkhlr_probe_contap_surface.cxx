// Contap_SurfProps at pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Contap_SurfProps.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <NCollection_Array2.hxx>
occ::handle<Geom_Surface> hl_contap_warped_surface(double height){
 NCollection_Array2<gp_Pnt> poles(1,3,1,3);
 for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)poles(i,j)=gp_Pnt(i-1,j-1,(i-1)*(j-1)*height);
 return new Geom_BezierSurface(poles);
}
occ::handle<Geom_Surface> hl_contap_make_surface(int kind,const gp_Ax3& f,double radius,double semi){
 switch(kind){case 0:return new Geom_Plane(f);case 1:return new Geom_SphericalSurface(f,radius);
 case 2:return new Geom_CylindricalSurface(f,radius);case 3:return new Geom_ConicalSurface(f,semi,radius);
 case 4:return new Geom_ToroidalSurface(f,radius,0.5);default:return hl_contap_warped_surface(radius);}
}
occ::handle<Adaptor3d_Surface> hl_contap_surface(GaReader& r){
 int kind=r.integer();gp_Ax3 f=hl_contour_frame(r);double radius=r.real(),semi=r.real();
 auto surface=hl_contap_make_surface(kind,f,radius,semi);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 return new GeomAdaptor_Surface(surface,u1,u2,v1,v2);
}
void hl_contap_surface_properties(const occ::handle<Adaptor3d_Surface>& s,double u,double v){
 gp_Pnt p;gp_Vec n,du,dv;
 GA_TRY(Contap_SurfProps::Normale(s,u,v,p,n);ga_pnt(p);ga_xyz(n.XYZ()));
 GA_TRY(Contap_SurfProps::DerivAndNorm(s,u,v,p,du,dv,n);ga_pnt(p);ga_xyz(du.XYZ());ga_xyz(dv.XYZ());ga_xyz(n.XYZ()));
 GA_TRY(Contap_SurfProps::NormAndDn(s,u,v,p,n,du,dv);ga_pnt(p);ga_xyz(n.XYZ());ga_xyz(du.XYZ());ga_xyz(dv.XYZ()));
}
void hl_contap_surface_case(std::istream& in){
 GaReader r{in};GA_TRY(auto s=hl_contap_surface(r);double u=r.real(),v=r.real();hl_contap_surface_properties(s,u,v));ga_end();
}
