#include <ChFi3d.hxx>
#include <iostream>
int main(){
 for(int a=0;a<4;a++)for(int b=0;b<4;b++)for(int c=0;c<4;c++){
  auto side=TopAbs_Orientation(a);ChFi3d::NextSide(side,TopAbs_Orientation(b),TopAbs_Orientation(c));std::cout<<"single "<<a<<' '<<b<<' '<<c<<' '<<side<<'\n';
  for(int d=0;d<4;d++){
   for(int choice=-8;choice<=8;choice++){auto x=TopAbs_Orientation(a),y=TopAbs_Orientation(b);int result=ChFi3d::NextSide(x,y,TopAbs_Orientation(c),TopAbs_Orientation(d),choice);std::cout<<"pair "<<a<<' '<<b<<' '<<c<<' '<<d<<' '<<choice<<' '<<result<<' '<<x<<' '<<y<<'\n';}
   for(int e=0;e<4;e++)std::cout<<"same "<<a<<' '<<b<<' '<<c<<' '<<d<<' '<<e<<' '<<ChFi3d::SameSide(TopAbs_Orientation(a),TopAbs_Orientation(b),TopAbs_Orientation(c),TopAbs_Orientation(d),TopAbs_Orientation(e))<<'\n';
  }
 }
}
