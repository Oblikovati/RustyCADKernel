// Bnd_OBB, Bnd_BoundSortBox, Bnd_Tools and the remaining Bnd_Box/Box2d/B2/B3 members at pinned OCCT 3d097a0.
#include <Bnd_OBB.hxx>
#include <Bnd_BoundSortBox.hxx>
#include <Bnd_Tools.hxx>
#include <Bnd_Box2d.hxx>
#include <Bnd_B2.hxx>
#include <Bnd_B3.hxx>
#include <gp_Pln.hxx>
#include <NCollection_HArray1.hxx>
namespace bnd_probe {
struct Reader {
  std::istream& in;
  double real() {double v;in>>v;return v;}
  int integer() {return int(real());}
  gp_XYZ triple() {double x=real(),y=real(),z=real();return gp_XYZ(x,y,z);}
};
void failure(const Standard_Failure& f) {std::cout<<f.ExceptionType()<<' ';}
void obb_values(const Bnd_OBB& o) {
  std::cout<<o.IsVoid()<<' '<<o.IsAABox()<<' ';xyz(o.Center());xyz(o.XDirection());xyz(o.YDirection());xyz(o.ZDirection());
  auto [hx,hy,hz]=o.GetHalfSizes();std::cout<<bits(hx)<<' '<<bits(hy)<<' '<<bits(hz)<<' '<<bits(o.SquareExtent())<<' ';
}
void obb_vertices(const Bnd_OBB& o) {
  gp_Pnt v[8];bool ok=o.GetVertex(v);std::cout<<ok<<' ';if(ok)for(const gp_Pnt& p:v)xyz(p.XYZ());
}
void obb_position(const Bnd_OBB& o) {
  try{gp_Ax3 a=o.Position();xyz(a.Location().XYZ());xyz(a.Direction().XYZ());xyz(a.XDirection().XYZ());xyz(a.YDirection().XYZ());}
  catch(const Standard_Failure& f){failure(f);}
}
void obb_dump(const Bnd_OBB& o,int depth) {std::ostringstream s;o.DumpJson(s,depth);std::cout<<s.str()<<" | ";}
// obb_build optimal with_tolerances point_lower tolerance_lower n x y z [tol]...
void obb_build_case(std::istream& in) {
  Reader r{in};bool optimal=r.integer()!=0,tolerant=r.integer()!=0;int lower=r.integer(),tolLower=r.integer(),n=r.integer();
  NCollection_Array1<gp_Pnt> points(lower,lower+n-1);NCollection_Array1<double> tolerances(tolLower,tolLower+n-1);
  for(int i=0;i<n;++i){points(lower+i)=gp_Pnt(r.triple());if(tolerant)tolerances(tolLower+i)=r.real();}
  Bnd_OBB o(gp_Pnt(7,8,9),gp_Dir(1,0,0),gp_Dir(0,1,0),gp_Dir(0,0,1),1,2,3);
  try{o.ReBuild(points,tolerant?&tolerances:nullptr,optimal);std::cout<<"ok ";}catch(const Standard_Failure& f){failure(f);}
  obb_values(o);obb_vertices(o);obb_position(o);
  for(int i=0;i<n;++i)std::cout<<o.IsOut(points(lower+i));
  std::cout<<' ';obb_dump(o,-1);obb_dump(o,0);std::cout<<'\n';
}
Bnd_OBB obb_read(Reader& r) {
  gp_XYZ c=r.triple(),x=r.triple(),y=r.triple(),z=r.triple(),h=r.triple();bool aa=r.integer()!=0;
  Bnd_OBB o(gp_Pnt(c),gp_Dir(x),gp_Dir(y),gp_Dir(z),h.X(),h.Y(),h.Z());o.SetAABox(aa);return o;
}
Bnd_Box box_read(Reader& r) {
  double v[6];for(double& value:v)value=r.real();double gap=r.real();int flags=r.integer();
  Bnd_Box b;if(!(flags&64))b.Update(v[0],v[1],v[2],v[3],v[4],v[5]);b.SetGap(gap);
  if(flags&1)b.OpenXmin();if(flags&2)b.OpenXmax();if(flags&4)b.OpenYmin();if(flags&8)b.OpenYmax();if(flags&16)b.OpenZmin();if(flags&32)b.OpenZmax();
  return b;
}
void obb_add(Bnd_OBB o,const Bnd_OBB& other) {try{o.Add(other);std::cout<<"ok ";}catch(const Standard_Failure& f){failure(f);}obb_values(o);}
void obb_add(Bnd_OBB o,const gp_Pnt& p) {try{o.Add(p);std::cout<<"ok ";}catch(const Standard_Failure& f){failure(f);}obb_values(o);}
// obb_pair A(16) B(16) point(3) enlarge box(6+gap+flags)
void obb_pair_case(std::istream& in) {
  Reader r{in};Bnd_OBB a=obb_read(r),b=obb_read(r);gp_Pnt p(r.triple());double gap=r.real();Bnd_Box box=box_read(r);
  obb_values(a);obb_values(b);obb_vertices(a);obb_position(a);
  std::cout<<a.IsOut(b)<<b.IsOut(a)<<a.Intersects(b)<<a.IsCompletelyInside(b)<<b.IsCompletelyInside(a)<<a.IsOut(p)<<a.Contains(p)<<b.IsOut(p)<<' ';
  obb_add(a,b);obb_add(b,a);obb_add(a,p);obb_add(Bnd_OBB(),a);obb_add(a,Bnd_OBB());obb_add(Bnd_OBB(),p);
  Bnd_OBB e=a;e.Enlarge(gap);obb_values(e);Bnd_OBB fromBox(box);obb_values(fromBox);std::cout<<fromBox.IsOut(a)<<a.IsOut(fromBox)<<' ';
  Bnd_OBB cleared=a;cleared.SetVoid();obb_values(cleared);obb_position(cleared);std::cout<<cleared.IsOut(p)<<cleared.IsOut(a)<<' ';
  obb_dump(a,-1);obb_dump(a,1);std::cout<<'\n';
}
void sort_result(const NCollection_List<int>& list) {std::cout<<list.Size()<<':';for(int index:list)std::cout<<index<<',';std::cout<<' ';}
// sortbox mode n box... enclosing nq query... np plane...
void sortbox_case(std::istream& in) {
  Reader r{in};int mode=r.integer(),n=r.integer();
  occ::handle<NCollection_HArray1<Bnd_Box>> boxes=new NCollection_HArray1<Bnd_Box>(1,n);for(int i=1;i<=n;++i)boxes->SetValue(i,box_read(r));
  Bnd_Box enclosing=box_read(r);Bnd_BoundSortBox sorter;
  try{
    if(mode==0)sorter.Initialize(boxes);
    if(mode==1)sorter.Initialize(enclosing,boxes);
    if(mode==2){sorter.Initialize(enclosing,n);for(int i=1;i<=n;++i)sorter.Add(boxes->Value(i),i);}
    if(mode==3){sorter.Initialize(boxes);sorter.Initialize(boxes);}
    std::cout<<"ok ";
  }catch(const Standard_Failure& f){failure(f);}
  if(mode==2){
    try{sorter.Add(boxes->Value(1),1);std::cout<<"ok ";}catch(const Standard_Failure& f){failure(f);}
    try{sorter.Add(boxes->Value(1),n+1);std::cout<<"ok ";}catch(const Standard_Failure& f){failure(f);}
  }
  int nq=r.integer();for(int i=0;i<nq;++i){Bnd_Box q=box_read(r);try{sort_result(sorter.Compare(q));}catch(const Standard_Failure& f){failure(f);}}
  int np=r.integer();for(int i=0;i<np;++i){gp_XYZ o=r.triple(),d=r.triple();try{sort_result(sorter.Compare(gp_Pln(gp_Pnt(o),gp_Dir(d))));}catch(const Standard_Failure& f){failure(f);}}
  std::cout<<'\n';
}
void box3_extra(const Bnd_Box& b) {
  auto l=b.Get();std::cout<<bits(l.Xmin)<<' '<<bits(l.Xmax)<<' '<<bits(l.Ymin)<<' '<<bits(l.Ymax)<<' '<<bits(l.Zmin)<<' '<<bits(l.Zmax)<<' ';
  std::cout<<bits(b.GetXMin())<<' '<<bits(b.GetXMax())<<' '<<bits(b.GetYMin())<<' '<<bits(b.GetYMax())<<' '<<bits(b.GetZMin())<<' '<<bits(b.GetZMax())<<' ';
  std::ostringstream s;std::streambuf* saved=std::cout.rdbuf(s.rdbuf());b.Dump();std::cout.rdbuf(saved);
  std::string text=s.str();for(char& c:text)if(c=='\n')c='|';std::cout<<text<<' ';
  try{BVH_Box<double,3> v=Bnd_Tools::Bnd2BVH(b);const BVH_Vec3d& lo=v.CornerMin();const BVH_Vec3d& hi=v.CornerMax();
    std::cout<<bits(lo.x())<<' '<<bits(lo.y())<<' '<<bits(lo.z())<<' '<<bits(hi.x())<<' '<<bits(hi.y())<<' '<<bits(hi.z())<<' ';}
  catch(const Standard_Failure& f){failure(f);}
}
// box_extra box(6+gap+flags) point(3)
void box_extra_case(std::istream& in) {
  Reader r{in};Bnd_Box b=box_read(r);gp_XYZ p=r.triple();box3_extra(b);b.Update(p.X(),p.Y(),p.Z());box3_extra(b);std::cout<<'\n';
}
Bnd_Box2d box2_read(Reader& r) {
  double v[4];for(double& value:v)value=r.real();double gap=r.real();int flags=r.integer();
  Bnd_Box2d b;if(!(flags&16))b.Update(v[0],v[1],v[2],v[3]);b.SetGap(gap);
  if(flags&1)b.OpenXmin();if(flags&2)b.OpenXmax();if(flags&4)b.OpenYmin();if(flags&8)b.OpenYmax();
  return b;
}
void box2_extra(const Bnd_Box2d& b) {
  auto l=b.Get();std::cout<<b.IsVoid()<<' '<<bits(l.Xmin)<<' '<<bits(l.Xmax)<<' '<<bits(l.Ymin)<<' '<<bits(l.Ymax)<<' ';
  std::cout<<bits(b.GetXMin())<<' '<<bits(b.GetXMax())<<' '<<bits(b.GetYMin())<<' '<<bits(b.GetYMax())<<' ';
  std::ostringstream s;std::streambuf* saved=std::cout.rdbuf(s.rdbuf());b.Dump();std::cout.rdbuf(saved);
  std::string text=s.str();for(char& c:text)if(c=='\n')c='|';std::cout<<text<<' ';
  try{BVH_Box<double,2> v=Bnd_Tools::Bnd2BVH(b);std::cout<<bits(v.CornerMin().x())<<' '<<bits(v.CornerMin().y())<<' '<<bits(v.CornerMax().x())<<' '<<bits(v.CornerMax().y())<<' ';}
  catch(const Standard_Failure& f){failure(f);}
}
gp_Trsf2d trsf2_read(Reader& r) {
  int kind=r.integer();double a=r.real(),b=r.real(),c=r.real();gp_Trsf2d t;
  if(kind==1)t.SetTranslation(gp_Vec2d(a,b));
  if(kind==2)t.SetRotation(gp_Pnt2d(a,b),c);
  if(kind==3)t.SetScale(gp_Pnt2d(a,b),c);
  if(kind==4)t.SetMirror(gp_Pnt2d(a,b));
  if(kind==5){gp_Trsf2d s;s.SetScale(gp_Pnt2d(b,a),c);t.SetRotation(gp_Pnt2d(a,b),c);t=t*s;}
  return t;
}
// box2_extra box(4+gap+flags) other(4+gap+flags) point(2) dir(2) trsf(4) trsf(4)
void box2_extra_case(std::istream& in) {
  Reader r{in};Bnd_Box2d b=box2_read(r),o=box2_read(r);double px=r.real(),py=r.real(),dx=r.real(),dy=r.real();gp_Trsf2d t1=trsf2_read(r),t2=trsf2_read(r);
  box2_extra(b);std::cout<<b.IsOut(o,t1)<<b.IsOut(t1,o,t2)<<' ';
  Bnd_Box2d ray=b;ray.Update(px,py);box2_extra(ray);ray.Set(gp_Pnt2d(px,py),gp_Dir2d(dx,dy));box2_extra(ray);
  ray=b;ray.Add(gp_Pnt2d(px,py),gp_Dir2d(dx,dy));box2_extra(ray);std::cout<<'\n';
}
gp_Trsf trsf3_read(Reader& r) {
  int kind=r.integer();gp_XYZ a=r.triple(),d=r.triple();double c=r.real();gp_Trsf t;
  if(kind==1)t.SetTranslation(gp_Vec(a));
  if(kind==2)t.SetRotation(gp_Ax1(gp_Pnt(a),gp_Dir(d)),c);
  if(kind==3)t.SetScale(gp_Pnt(a),c);
  if(kind==4)t.SetMirror(gp_Pnt(a));
  if(kind==5){gp_Trsf s;s.SetScale(gp_Pnt(d),c);t.SetRotation(gp_Ax1(gp_Pnt(a),gp_Dir(d)),c);t=t*s;}
  if(kind==6)t.SetMirror(gp_Ax2(gp_Pnt(a),gp_Dir(d)));
  return t;
}
template<class T> void compact_relations(const double* v,const gp_Trsf& t3,const gp_Trsf2d& t2) {
  Bnd_B3<T> a(gp_XYZ(v[0],v[1],v[2]),gp_XYZ(v[3],v[4],v[5])),b(gp_XYZ(v[6],v[7],v[8]),gp_XYZ(v[9],v[10],v[11]));
  std::cout<<a.IsOut(b,t3)<<b.IsOut(a,t3)<<a.IsIn(b,t3)<<b.IsIn(a,t3)<<' ';
  Bnd_B2<T> c(gp_XY(v[0],v[1]),gp_XY(v[3],v[4])),d(gp_XY(v[6],v[7]),gp_XY(v[9],v[10]));
  std::cout<<c.IsOut(d,t2)<<d.IsOut(c,t2)<<c.IsIn(d,t2)<<d.IsIn(c,t2)<<' ';
}
// compact_trsf a(6) b(6) trsf3(8) trsf2(4)
void compact_trsf_case(std::istream& in) {
  Reader r{in};double v[12];for(double& value:v)value=r.real();gp_Trsf t3=trsf3_read(r);gp_Trsf2d t2=trsf2_read(r);
  compact_relations<double>(v,t3,t2);compact_relations<float>(v,t3,t2);std::cout<<'\n';
}
} // namespace bnd_probe
