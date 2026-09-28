// TopLoc_Location composition, powers, equality and hashing oracle at pinned OCCT 3d097a0.
#include <TopLoc_Datum3D.hxx>
#include <TopLoc_Location.hxx>
#include <Standard_HashUtils.hxx>
#include <vector>
namespace toploc_probe {
using poly_probe::failure;using poly_probe::integer;using poly_probe::real;
void trsf(const gp_Trsf& t){for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)std::cout<<bits(t.Value(r,c))<<' ';std::cout<<int(t.Form())<<' ';}
occ::handle<TopLoc_Datum3D> datum(std::istream& in){
 double p[3],d[3];for(double& v:p)v=real(in);for(double& v:d)v=real(in);double angle=real(in),scale=real(in),t[3];for(double& v:t)v=real(in);
 gp_Trsf r;r.SetRotation(gp_Ax1(gp_Pnt(p[0],p[1],p[2]),gp_Dir(d[0],d[1],d[2])),angle);
 gp_Trsf s;s.SetScale(gp_Pnt(0,0,0),scale);gp_Trsf m;m.SetTranslation(gp_Vec(t[0],t[1],t[2]));
 return new TopLoc_Datum3D(m*s*r);
}
// The HashCode recurrence over (datum address, power) pairs, head first.
size_t hash_items(const std::vector<std::pair<size_t,int>>& items){
 if(items.empty())return 0;size_t h=opencascade::MurmurHash::optimalSeed<size_t>();
 for(const auto& item:items){size_t c[3]={item.first,opencascade::hash(item.second),h};h=opencascade::hashBytes(c,sizeof(c));}
 return h;
}
void hashes(const TopLoc_Location& l,const std::vector<occ::handle<TopLoc_Datum3D>>& datums){
 std::vector<std::pair<size_t,int>> real,synthetic;
 for(TopLoc_Location it=l;!it.IsIdentity();it=it.NextLocation()){
  size_t id=0;while(id<datums.size()&&datums[id]!=it.FirstDatum())++id;
  real.emplace_back(std::hash<occ::handle<TopLoc_Datum3D>>{}(it.FirstDatum()),it.FirstPower());synthetic.emplace_back(id*8+4096,it.FirstPower());}
 std::cout<<(l.HashCode()==hash_items(real))<<' '<<hash_items(synthetic)<<' '<<real.size()<<' ';
}
void guarded(const char* tag,const std::function<TopLoc_Location()>& make){
 std::cout<<tag<<' ';try{TopLoc_Location l=make();std::cout<<l.IsIdentity()<<' ';trsf(l.Transformation());}catch(const Standard_Failure& f){failure(f);}
}
void location_case(std::istream& in){
 int nd=integer(in);std::vector<occ::handle<TopLoc_Datum3D>> datums;
 try{for(int i=0;i<nd;++i)datums.push_back(datum(in));}catch(const Standard_Failure& f){failure(f);std::cout<<'\n';return;}
 int m=integer(in);TopLoc_Location chain;TopLoc_Location first;
 try{for(int i=0;i<m;++i){int id=integer(in),p=integer(in);TopLoc_Location item=TopLoc_Location(datums[id]).Powered(p);if(i==0)first=item;chain=chain*item;}
 }catch(const Standard_Failure& f){failure(f);}
 int pw=integer(in);
 std::cout<<chain.IsIdentity()<<' ';trsf(chain.Transformation());hashes(chain,datums);
 try{std::cout<<chain.FirstPower()<<' ';}catch(const Standard_Failure& f){failure(f);}
 trsf(chain.NextLocation().Transformation());
 guarded("inv",[&]{return chain.Inverted();});guarded("id",[&]{return chain*chain.Inverted();});
 guarded("pow",[&]{return chain.Powered(pw);});guarded("npow",[&]{return chain.Powered(-pw);});
 guarded("div",[&]{return chain.Divided(first);});guarded("pre",[&]{return chain.Predivided(first);});
 guarded("self",[&]{return chain.Predivided(chain);});
 TopLoc_Location copy(chain.Transformation());std::cout<<(chain==chain*TopLoc_Location())<<(chain!=copy)<<(first.Inverted()*first==TopLoc_Location())<<' ';
 std::ostringstream json;chain.DumpJson(json);std::cout<<json.str()<<'\n';
}
} // namespace toploc_probe
