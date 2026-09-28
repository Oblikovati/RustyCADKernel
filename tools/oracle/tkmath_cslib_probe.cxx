// CSLib normals, normal derivatives, NormalPolyDef and Class2d at pinned OCCT 3d097a0.
#include <CSLib.hxx>
#include <CSLib_Class2d.hxx>
#include <CSLib_NormalPolyDef.hxx>
#include <NCollection_Array2.hxx>
#include <NCollection_DynamicArray.hxx>
#include <NCollection_Sequence.hxx>
#include <limits>
#include <utility>
#include <vector>
gp_Vec cslib_vec(std::istream& in){double x,y,z;in>>x>>y>>z;return gp_Vec(x,y,z);}
void cslib_failure(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';}
void cslib_normal_case(std::istream& in){
 gp_Vec d1u=cslib_vec(in),d1v=cslib_vec(in),d2u=cslib_vec(in),d2v=cslib_vec(in),d2uv=cslib_vec(in);double tol;in>>tol;
 try{CSLib_DerivativeStatus s;gp_Dir n;CSLib::Normal(d1u,d1v,tol,s,n);std::cout<<int(s)<<' ';xyz(n.XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 try{CSLib_NormalStatus s;bool done=false;gp_Dir n;CSLib::Normal(d1u,d1v,d2u,d2v,d2uv,tol,done,s,n);std::cout<<done<<' '<<int(s)<<' ';xyz(n.XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 try{CSLib_NormalStatus s;gp_Dir n;CSLib::Normal(d1u,d1v,tol,s,n);std::cout<<int(s)<<' ';xyz(n.XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 std::cout<<'\n';
}
NCollection_Array2<gp_Vec> cslib_table(std::istream& in,int size){
 NCollection_Array2<gp_Vec> table(0,size,0,size);for(int i=0;i<=size;++i)for(int j=0;j<=size;++j)table(i,j)=cslib_vec(in);return table;
}
void cslib_singular_case(std::istream& in){
 int order,size;double tol,u,v,umin,umax,vmin,vmax;in>>order>>size>>tol>>u>>v>>umin>>umax>>vmin>>vmax;
 NCollection_Array2<gp_Vec> der=cslib_table(in,size);
 try{CSLib_NormalStatus s;gp_Dir n;int ou=0,ov=0;CSLib::Normal(order,der,tol,u,v,umin,umax,vmin,vmax,s,n,ou,ov);
  std::cout<<int(s)<<' ';xyz(n.XYZ());std::cout<<ou<<' '<<ov<<' ';}catch(const Standard_Failure& f){cslib_failure(f);}
 std::cout<<'\n';
}
void cslib_dn_case(std::istream& in){
 int size,nu,nv,iu,iv;in>>size>>nu>>nv>>iu>>iv;
 NCollection_Array2<gp_Vec> first=cslib_table(in,size);NCollection_Array2<gp_Vec> second=cslib_table(in,size);
 try{xyz(CSLib::DNNUV(nu,nv,first).XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 try{xyz(CSLib::DNNUV(nu,nv,first,second).XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 try{xyz(CSLib::DNNormal(nu,nv,first,iu,iv).XYZ());}catch(const Standard_Failure& f){cslib_failure(f);}
 std::cout<<'\n';
}
void cslib_poly_case(std::istream& in){
 int k;double x;in>>k>>x;NCollection_Array1<double> li(0,std::max(k,0));for(int i=0;i<=std::max(k,0);++i)in>>li(i);
 try{CSLib_NormalPolyDef p(k,li);double f=91,d=92,vf=93,vd=94;bool a=p.Value(x,f);bool b=p.Derivative(x,d);bool c=p.Values(x,vf,vd);
  std::cout<<a<<' '<<b<<' '<<c<<' '<<bits(f)<<' '<<bits(d)<<' '<<bits(vf)<<' '<<bits(vd)<<' ';}catch(const Standard_Failure& f){cslib_failure(f);}
 std::cout<<'\n';
}
CSLib_Class2d cslib_classifier(std::istream& in,int container,double tu,double tv,const double* b){
 int n;in>>n;NCollection_Array1<gp_Pnt2d> points(1,std::max(n,1));NCollection_Sequence<gp_Pnt2d> sequence;NCollection_DynamicArray<gp_Pnt2d> dynamic;
 for(int i=1;i<=n;++i){double x,y;in>>x>>y;points(i)=gp_Pnt2d(x,y);sequence.Append(points(i));dynamic.Append(points(i));}
 if(container==1)return CSLib_Class2d(sequence,tu,tv,b[0],b[1],b[2],b[3]);
 if(container==2)return CSLib_Class2d(dynamic,tu,tv,b[0],b[1],b[2],b[3]);
 return CSLib_Class2d(points,tu,tv,b[0],b[1],b[2],b[3]);
}
void cslib_queries(const CSLib_Class2d& c,const std::vector<gp_Pnt2d>& q){for(const gp_Pnt2d& p:q)std::cout<<int(c.SiDans(p))<<' ';}
void cslib_class2d_case(std::istream& in){
 int special,container;double tu,tv,on,b[4];in>>special>>container>>tu>>tv>>on>>b[0]>>b[1]>>b[2]>>b[3];
 if(special&1)tu=std::numeric_limits<double>::infinity();
 if(special&2)tv=std::numeric_limits<double>::quiet_NaN();
 if(special&4)on=std::numeric_limits<double>::infinity();
 CSLib_Class2d c=cslib_classifier(in,container,tu,tv,b);
 int m;in>>m;std::vector<gp_Pnt2d> q;for(int i=0;i<m;++i){double x,y;in>>x>>y;q.emplace_back(x,y);}
 cslib_queries(c,q);for(const gp_Pnt2d& p:q)std::cout<<int(c.SiDans_OnMode(p,on))<<' ';
 const gp_Pnt2d center(b[0]/2+b[2]/2,b[1]/2+b[3]/2);for(int i=0;i<64;++i)std::cout<<int(c.SiDans(center));std::cout<<' ';
 cslib_queries(c,q);CSLib_Class2d copy(c);cslib_queries(copy,q);
 CSLib_Class2d moved(std::move(c));std::cout<<int(c.SiDans(center))<<' ';cslib_queries(moved,q);
 CSLib_Class2d assigned;assigned=std::move(moved);std::cout<<int(moved.SiDans(center))<<' ';cslib_queries(assigned,q);
 std::cout<<'\n';
}
