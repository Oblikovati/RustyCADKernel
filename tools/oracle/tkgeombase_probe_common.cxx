// Shared reader and printers of the TKGeomBase oracle (tkgeombase_probe.cxx). Every double
// is printed as raw IEEE bits, every raised exception as its type name, as in the TKG2d and
// TKG3d oracles; the Rust twin is crates/tkgeombase/examples/gb_probe/common.rs.
#define GB_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& gb_error){std::cout<<gb_error.ExceptionType()<<' ';}
uint64_t gb_bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void gb_real(double v){std::cout<<gb_bits(v)<<' ';}
void gb_int(int v){std::cout<<v<<' ';}
void gb_flag(bool v){std::cout<<(v?1:0)<<' ';}
void gb_end(){std::cout<<'\n';}
void gb_xyz(const gp_XYZ& v){gb_real(v.X());gb_real(v.Y());gb_real(v.Z());}
void gb_pnt(const gp_Pnt& p){gb_xyz(p.XYZ());}
void gb_vec(const gp_Vec& v){gb_xyz(v.XYZ());}
void gb_dir(const gp_Dir& d){gb_xyz(d.XYZ());}
void gb_ax1(const gp_Ax1& a){gb_pnt(a.Location());gb_dir(a.Direction());}
void gb_ax2(const gp_Ax2& a){gb_pnt(a.Location());gb_dir(a.Direction());gb_dir(a.XDirection());gb_dir(a.YDirection());}
void gb_ax3(const gp_Ax3& a){gb_pnt(a.Location());gb_dir(a.Direction());gb_dir(a.XDirection());gb_dir(a.YDirection());}
void gb_xy(const gp_XY& v){gb_real(v.X());gb_real(v.Y());}
void gb_pnt2d(const gp_Pnt2d& p){gb_xy(p.XY());}
void gb_vec2d(const gp_Vec2d& v){gb_xy(v.XY());}
void gb_dir2d(const gp_Dir2d& d){gb_xy(d.XY());}
void gb_ax2d(const gp_Ax2d& a){gb_pnt2d(a.Location());gb_dir2d(a.Direction());}
void gb_ax22d(const gp_Ax22d& a){gb_pnt2d(a.Location());gb_dir2d(a.XDirection());gb_dir2d(a.YDirection());}
// Sequential reader over one request; the corpus only feeds it valid directions and frames.
struct GbReader{
 std::istream& in;
 double real(){double v;in>>v;return v;}
 int integer(){return int(real());}
 bool flag(){return integer()!=0;}
 gp_XYZ xyz(){double x=real(),y=real();return gp_XYZ(x,y,real());}
 gp_Pnt pnt(){double x=real(),y=real();return gp_Pnt(x,y,real());}
 gp_Vec vec(){double x=real(),y=real();return gp_Vec(x,y,real());}
 gp_Dir dir(){double x=real(),y=real();return gp_Dir(x,y,real());}
 gp_Ax1 ax1(){gp_Pnt p=pnt();return gp_Ax1(p,dir());}
 gp_Ax2 ax2(){gp_Pnt p=pnt();gp_Dir n=dir();return gp_Ax2(p,n,dir());}
 gp_Ax3 ax3(){gp_Pnt p=pnt();gp_Dir n=dir();return gp_Ax3(p,n,dir());}
 gp_XY xy(){double x=real();return gp_XY(x,real());}
 gp_Pnt2d pnt2d(){double x=real();return gp_Pnt2d(x,real());}
 gp_Vec2d vec2d(){double x=real();return gp_Vec2d(x,real());}
 gp_Dir2d dir2d(){double x=real();return gp_Dir2d(x,real());}
 gp_Ax2d ax2d(){gp_Pnt2d p=pnt2d();return gp_Ax2d(p,dir2d());}
 gp_Ax22d ax22d(){gp_Pnt2d p=pnt2d();gp_Dir2d x=dir2d();return gp_Ax22d(p,x,dir2d());}
};
