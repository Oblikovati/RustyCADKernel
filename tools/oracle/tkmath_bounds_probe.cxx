// Supplemental OCCT oracle: gp_Lin, gp_Pln and Bnd_Box observable state and queries.
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
#include <Bnd_Box.hxx>
void box_values(const Bnd_Box& box) {
  std::cout<<box.IsVoid()<<' '<<box.IsWhole()<<' '<<box.IsOpen()<<' '<<box.HasFinitePart()<<' ';
  std::cout<<bits(box.GetGap())<<' '<<bits(box.SquareExtent())<<' ';
  if(!box.IsVoid()){auto b=box.Get();std::cout<<bits(b.Xmin)<<' '<<bits(b.Ymin)<<' '<<bits(b.Zmin)<<' '<<bits(b.Xmax)<<' '<<bits(b.Ymax)<<' '<<bits(b.Zmax)<<' ';}
}
void plane_values(const gp_Pln& plane) {
  xyz(plane.Location().XYZ());xyz(plane.Position().Direction().XYZ());xyz(plane.Position().XDirection().XYZ());xyz(plane.Position().YDirection().XYZ());
  double a,b,c,d;plane.Coefficients(a,b,c,d);std::cout<<bits(a)<<' '<<bits(b)<<' '<<bits(c)<<' '<<bits(d)<<' ';
}
void bounds_case(std::istream& input) {
  double v[16];for(auto& value:v)input>>value;
  gp_Pnt first(v[0],v[1],v[2]),second(v[3],v[4],v[5]);gp_Dir direction(v[6],v[7],v[8]);
  Bnd_Box box;box.Add(first);box.Add(second);box.SetGap(v[9]);int flags=int(v[15]);
  if(flags&1)box.OpenXmin();if(flags&2)box.OpenXmax();if(flags&4)box.OpenYmin();if(flags&8)box.OpenYmax();if(flags&16)box.OpenZmin();if(flags&32)box.OpenZmax();
  if(flags==64)box.SetVoid();if(flags==65)box.SetWhole();
  gp_Pnt point(v[10],v[11],v[12]);gp_Lin line(point,direction);gp_Pln plane(point,direction);
  box_values(box);std::cout<<box.IsOut(point)<<' '<<box.IsOut(line)<<' '<<box.IsOut(plane)<<' '<<box.IsOut(first,point,direction)<<' ';
  std::cout<<box.IsXThin(v[13])<<' '<<box.IsYThin(v[13])<<' '<<box.IsZThin(v[13])<<' '<<box.IsThin(v[13])<<' ';
  Bnd_Box other;other.Add(point);other.SetGap(v[14]);std::cout<<box.IsOut(other)<<' '<<bits(box.Distance(other))<<' ';box.Add(other);box_values(box);
  gp_Trsf rotation;rotation.SetRotation(gp_Ax1(point,direction),v[13]);box_values(box.Transformed(rotation));box_values(box.FinitePart());std::cout<<'\n';
}
void line_plane_case(std::istream& input) {
  double v[12];for(auto& value:v)input>>value;
  gp_Pnt origin(v[0],v[1],v[2]),point(v[3],v[4],v[5]);gp_Dir direction(v[6],v[7],v[8]);
  gp_Lin line(origin,direction),other(point,gp_Dir(v[3],v[4],v[5]));
  std::cout<<bits(line.Distance(point))<<' '<<bits(line.SquareDistance(point))<<' '<<bits(line.Distance(other))<<' ';
  gp_Pln plane(origin,direction);plane_values(plane);std::cout<<bits(plane.Distance(point))<<' '<<bits(plane.SignedDistance(point))<<' ';
  plane_values(gp_Pln(v[6],v[7],v[8],v[9]));plane_values(plane.Scaled(point,v[10]));
  plane_values(plane.Mirrored(gp_Ax2(point,direction)));plane_values(plane.Rotated(gp_Ax1(point,direction),v[11]));std::cout<<'\n';
}
#include <Bnd_Box2d.hxx>
#include <gp_Lin2d.hxx>
void box2_values(const Bnd_Box2d& box) {
  std::cout<<box.IsVoid()<<' '<<box.IsWhole()<<' '<<bits(box.GetGap())<<' '<<bits(box.SquareExtent())<<' ';
  auto b=box.Get();std::cout<<bits(b.Xmin)<<' '<<bits(b.Ymin)<<' '<<bits(b.Xmax)<<' '<<bits(b.Ymax)<<' ';
}
void bounds2_case(std::istream& input) {
  double v[13];for(auto& value:v)input>>value;
  gp_Pnt2d first(v[0],v[1]),second(v[2],v[3]),point(v[4],v[5]);gp_Dir2d direction(v[6],v[7]);
  Bnd_Box2d box;box.Add(first);box.Add(second);box.SetGap(v[8]);int flags=int(v[12]);
  if(flags&1)box.OpenXmin();if(flags&2)box.OpenXmax();if(flags&4)box.OpenYmin();if(flags&8)box.OpenYmax();if(flags==16)box.SetVoid();if(flags==17)box.SetWhole();
  box_values(Bnd_Box());box2_values(box);std::cout<<box.IsOut(point)<<' '<<box.IsOut(gp_Lin2d(point,direction))<<' '<<box.IsOut(first,point)<<' ';
  Bnd_Box2d other;other.Add(point);other.SetGap(v[9]);std::cout<<box.IsOut(other)<<' '<<bits(box.Distance(other))<<' ';box.Add(other);box2_values(box);
  box.Add(point);box2_values(box);gp_Trsf2d rotation;rotation.SetRotation(point,v[10]);box2_values(box.Transformed(rotation));
  gp_Trsf2d translation;translation.SetTranslation(gp_Vec2d(v[10],v[11]));box2_values(box.Transformed(translation));std::cout<<'\n';
}
#include <Bnd_Sphere.hxx>
void sphere_case(std::istream& input) {
  double v[12];for(auto& value:v)input>>value;
  Bnd_Sphere sphere(gp_XYZ(v[0],v[1],v[2]),v[6],3,4),other(gp_XYZ(v[3],v[4],v[5]),v[7],8,9);sphere.SetValid(v[8]>0);other.SetValid(v[9]>0);
  gp_XYZ point(v[3],v[10],v[11]);double lo,hi; sphere.Distances(point,lo,hi);std::cout<<bits(lo)<<' '<<bits(hi)<<' ';sphere.SquareDistances(point,lo,hi);std::cout<<bits(lo)<<' '<<bits(hi)<<' ';
  double limit=v[10];std::cout<<sphere.IsOut(point,limit)<<' '<<bits(limit)<<' '<<sphere.IsOut(other)<<' ';sphere.Add(other);xyz(sphere.Center());std::cout<<bits(sphere.Radius())<<' '<<sphere.IsValid()<<' '<<sphere.U()<<' '<<sphere.V()<<' '<<bits(sphere.SquareExtent())<<'\n';
}
#include <Bnd_B2.hxx>
template<class T> void compact2_values(const Bnd_B2<T>& box) {
  std::cout<<box.IsVoid()<<' ';xy(box.CornerMin());xy(box.CornerMax());std::cout<<bits(box.SquareExtent())<<' ';
}
template<class T> void compact2_typed(const double* v) {
  Bnd_B2<T> box(gp_XY(v[0],v[1]),gp_XY(v[2],v[3])),other(gp_XY(v[4],v[5]),gp_XY(v[6],v[7]));
  compact2_values(box);gp_XY point(v[8],v[9]);std::cout<<box.IsOut(point)<<' '<<box.IsOut(other)<<' '<<box.IsIn(other)<<' '<<box.IsOut(point,v[10])<<' '<<box.IsOut(point,v[10],true)<<' ';
  std::cout<<box.IsOut(gp_Ax2d(gp_Pnt2d(point),gp_Dir2d(1,0)))<<' '<<box.IsOut(point,gp_XY(v[4],v[5]))<<' ';
  auto limited=box;std::cout<<limited.Limit(other)<<' ';compact2_values(limited);
  box.Add(point);box.Add(other);box.Enlarge(v[10]);compact2_values(box);
  gp_Trsf2d rotation;rotation.SetRotation(gp_Pnt2d(point),v[11]);compact2_values(box.Transformed(rotation));
}
void compact2_case(std::istream& input) {
  double v[12];for(auto& value:v)input>>value;compact2_typed<double>(v);compact2_typed<float>(v);std::cout<<'\n';
}
#include <Bnd_B3.hxx>
template<class T> void compact3_values(const Bnd_B3<T>& box) {
  std::cout<<box.IsVoid()<<' ';xyz(box.CornerMin());xyz(box.CornerMax());std::cout<<bits(box.SquareExtent())<<' ';
}
template<class T> void compact3_typed(const double* v) {
  Bnd_B3<T> box(gp_XYZ(v[0],v[1],v[2]),gp_XYZ(v[3],v[4],v[5])),other(gp_XYZ(v[6],v[7],v[8]),gp_XYZ(v[9],v[10],v[11]));
  compact3_values(box);gp_XYZ point(v[12],v[13],v[14]);gp_Dir direction(v[6],v[7],v[8]);gp_Ax1 axis(gp_Pnt(point),direction);
  std::cout<<box.IsOut(point)<<' '<<box.IsOut(other)<<' '<<box.IsIn(other)<<' '<<box.IsOut(point,v[15])<<' '<<box.IsOut(point,v[15],true)<<' ';
  std::cout<<box.IsOut(axis,false,v[17])<<' '<<box.IsOut(axis,true,v[17])<<' '<<box.IsOut(gp_Ax3(gp_Pnt(point),direction))<<' ';
  auto limited=box;std::cout<<limited.Limit(other)<<' ';compact3_values(limited);box.Add(point);box.Add(other);box.Enlarge(v[15]);compact3_values(box);
  gp_Trsf rotation;rotation.SetRotation(axis,v[16]);compact3_values(box.Transformed(rotation));
}
void compact3_case(std::istream& input) {
  double v[18];for(auto& value:v)input>>value;compact3_typed<double>(v);compact3_typed<float>(v);std::cout<<'\n';
}
