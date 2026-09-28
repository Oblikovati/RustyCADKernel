// gp_Dir::D / gp_Dir2d::D constructors and the gp statics at pinned OCCT 3d097a0. Input: main
// direction, X direction (0..5 for X, Y, Z, NX, NY, NZ) and an origin; the 2D names use 0..3.
#include <gp.hxx>
void gpstd_ax2(const gp_Ax2& a){xyz(a.Location().XYZ());xyz(a.Direction().XYZ());xyz(a.XDirection().XYZ());xyz(a.YDirection().XYZ());}
void gpstd_ax3(const gp_Ax3& a){xyz(a.Location().XYZ());xyz(a.Direction().XYZ());xyz(a.XDirection().XYZ());xyz(a.YDirection().XYZ());std::cout<<a.Direct()<<' ';}
void gpstd_ax1(const gp_Ax1& a){xyz(a.Location().XYZ());xyz(a.Direction().XYZ());}
void gpstd_ax2d(const gp_Ax2d& a){xy(a.Location().XY());xy(a.Direction().XY());}
void gp_standard_case(std::istream& in){
 const gp_Dir::D names[6]={gp_Dir::D::X,gp_Dir::D::Y,gp_Dir::D::Z,gp_Dir::D::NX,gp_Dir::D::NY,gp_Dir::D::NZ};
 int n,x;double px,py,pz;in>>n>>x>>px>>py>>pz;gp_Pnt p(px,py,pz);gp_Dir::D main=names[n],second=names[x];
 xyz(gp_Dir(main).XYZ());gpstd_ax1(gp_Ax1(p,main));gpstd_ax1(gp_Ax1(main));
 gpstd_ax2(gp_Ax2(p,main,second));gpstd_ax2(gp_Ax2(p,main));gpstd_ax2(gp_Ax2(main));
 gpstd_ax3(gp_Ax3(p,main,second));gpstd_ax3(gp_Ax3(p,main));gpstd_ax3(gp_Ax3(main));
 const gp_Dir2d::D names2[4]={gp_Dir2d::D::X,gp_Dir2d::D::Y,gp_Dir2d::D::NX,gp_Dir2d::D::NY};
 gp_Dir2d::D name2=names2[x%4];xy(gp_Dir2d(name2).XY());gpstd_ax2d(gp_Ax2d(gp_Pnt2d(px,py),name2));gpstd_ax2d(gp_Ax2d(name2));
 std::cout<<bits(gp::Resolution())<<' ';xyz(gp::Origin().XYZ());xyz(gp::DX().XYZ());xyz(gp::DY().XYZ());xyz(gp::DZ().XYZ());
 gpstd_ax1(gp::OX());gpstd_ax1(gp::OY());gpstd_ax1(gp::OZ());gpstd_ax2(gp::XOY());gpstd_ax2(gp::ZOX());gpstd_ax2(gp::YOZ());
 xy(gp::Origin2d().XY());xy(gp::DX2d().XY());xy(gp::DY2d().XY());gpstd_ax2d(gp::OX2d());gpstd_ax2d(gp::OY2d());
 std::cout<<'\n';
}
