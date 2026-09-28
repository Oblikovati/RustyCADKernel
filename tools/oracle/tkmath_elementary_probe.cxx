// ElCLib, ElSLib and gp conic/quadric/axis oracles at pinned OCCT 3d097a0.
// Every value is printed as raw IEEE bits and every raised exception as its type name.
#include <ElCLib.hxx>
#include <ElSLib.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
#include <NCollection_Array1.hxx>
#define EL_TRY(statement) try{statement;}catch(const Standard_Failure& el_error){el_fail(el_error);}
void el_fail(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
void el_real(double value){std::cout<<bits(value)<<' ';}
void el_flag(bool value){std::cout<<value<<' ';}
void el_pnt(const gp_Pnt& p){xyz(p.XYZ());}
void el_vec(const gp_Vec& v){xyz(v.XYZ());}
void el_dir(const gp_Dir& d){xyz(d.XYZ());}
void el_ax1(const gp_Ax1& a){el_pnt(a.Location());el_dir(a.Direction());}
void el_ax2(const gp_Ax2& a){el_pnt(a.Location());el_dir(a.Direction());el_dir(a.XDirection());el_dir(a.YDirection());}
void el_ax3(const gp_Ax3& a){el_pnt(a.Location());el_dir(a.Direction());el_dir(a.XDirection());el_dir(a.YDirection());}
void el_pnt2(const gp_Pnt2d& p){xy(p.XY());}
void el_vec2(const gp_Vec2d& v){xy(v.XY());}
void el_dir2(const gp_Dir2d& d){xy(d.XY());}
void el_ax2d(const gp_Ax2d& a){el_pnt2(a.Location());el_dir2(a.Direction());}
void el_ax22d(const gp_Ax22d& a){el_pnt2(a.Location());el_dir2(a.XDirection());el_dir2(a.YDirection());}
void el_lin(const gp_Lin& l){el_ax1(l.Position());}
void el_circ(const gp_Circ& c){el_ax2(c.Position());el_real(c.Radius());}
void el_reals(const double* values,int count){for(int i=0;i<count;++i)el_real(values[i]);}
gp_Pnt el_read_pnt(std::istream& in){double x,y,z;in>>x>>y>>z;return gp_Pnt(x,y,z);}
gp_XYZ el_read_xyz(std::istream& in){double x,y,z;in>>x>>y>>z;return gp_XYZ(x,y,z);}
gp_Pnt2d el_read_pnt2(std::istream& in){double x,y;in>>x>>y;return gp_Pnt2d(x,y);}
gp_XY el_read_xy(std::istream& in){double x,y;in>>x>>y;return gp_XY(x,y);}
#include "tkmath_elclib_probe.cxx"
#include "tkmath_elslib_probe.cxx"
#include "tkmath_gp_conic_probe.cxx"
#include "tkmath_gp_quadric_probe.cxx"
