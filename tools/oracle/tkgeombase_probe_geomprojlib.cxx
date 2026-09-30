// GeomProjLib (tools/tkgeombase_corpus_geomprojlib.py): the six Curve2d overloads,
// Project and ProjectOnPlane on the GeomTools builders (gt_curve, gt_surface, gt_ax3);
// the results print through gb_gcv_curve and gb_gcv_curve2d.

#include <GeomProjLib.hxx>
// `gpl_curve2d variant first last u1 u2 v1 v2 tol curve surface`: variant 0
// (C, First, Last, S, U1, U2, V1, V2, Tol), 1 (C, First, Last, S, Tol),
// 2 (C, First, Last, S), 3 (C, S), 4 (C, S, U1, U2, V1, V2), 5 (C, S, U1, U2, V1, V2,
// Tol); then the 2D curve and, for the overloads that take it, the tolerance reached.
void gb_gpl_curve2d_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const double f=r.real(),l=r.real();
 const double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();double tol=r.real();
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);occ::handle<Geom_Surface> s=gt_surface(r);occ::handle<Geom2d_Curve> p;
  switch(variant){
   case 0:p=GeomProjLib::Curve2d(c,f,l,s,u1,u2,v1,v2,tol);break;
   case 1:p=GeomProjLib::Curve2d(c,f,l,s,tol);break;
   case 2:p=GeomProjLib::Curve2d(c,f,l,s);break;
   case 3:p=GeomProjLib::Curve2d(c,s);break;
   case 4:p=GeomProjLib::Curve2d(c,s,u1,u2,v1,v2);break;
   default:p=GeomProjLib::Curve2d(c,s,u1,u2,v1,v2,tol);break;
  }
  gb_gcv_curve2d(p);if(variant==0||variant==1||variant==5)gb_real(tol));
 gb_end();
}
// `gpl_project curve surface`: the 3D curve of the normal projection.
void gb_gpl_project_case(std::istream& in){
 GbReader r{in};
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);occ::handle<Geom_Surface> s=gt_surface(r);gb_gcv_curve(GeomProjLib::Project(c,s)));
 gb_end();
}
// `gpl_on_plane keep ax3 dir curve`: the projection on Geom_Plane(ax3) along dir.
void gb_gpl_on_plane_case(std::istream& in){
 GbReader r{in};const bool keep=r.flag();const gp_Ax3 pl=gt_ax3(r);const gp_Dir d=r.dir();
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);occ::handle<Geom_Plane> plane=new Geom_Plane(pl);
  gb_gcv_curve(GeomProjLib::ProjectOnPlane(c,plane,d,keep)));
 gb_end();
}
