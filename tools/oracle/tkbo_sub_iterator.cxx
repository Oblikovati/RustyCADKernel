#include <BOPDS_DS.hxx>
#include <BOPDS_SubIterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <iostream>
class Probe: public BOPDS_SubIterator {public:using BOPDS_SubIterator::Intersect;};
NCollection_List<int> subset(BOPDS_DS& ds,int mode)
{
 NCollection_List<int> result;
 for(int i=ds.NbSourceShapes()-1;i>=0;--i) {
  const auto type=ds.ShapeInfo(i).ShapeType();
  if((mode==0 && (type==TopAbs_VERTEX||type==TopAbs_EDGE||type==TopAbs_FACE)) || (mode==1&&type==TopAbs_VERTEX) || (mode==2&&type==TopAbs_EDGE) || (mode==3&&type==TopAbs_FACE)) {
   result.Append(i);if(mode==2)result.Append(i);
  }
 }
 return result;
}
void run(BOPDS_DS& ds,int a,int b)
{
 auto first=subset(ds,a),second=subset(ds,b);Probe it;
 it.SetDS(&ds);it.SetSubSet1(first);it.SetSubSet2(second);it.Prepare();
 for(int pass=0;pass<2;++pass) {
  if(pass)it.Intersect();it.Initialize();
  std::cout<<a<<' '<<b<<' '<<pass<<' '<<it.ExpectedLength();
  while(it.More()){int x,y;it.Value(x,y);std::cout<<' '<<x<<' '<<y;it.Next();}
  std::cout<<'\n';
 }
}
int main()
{
 BOPDS_DS ds;NCollection_List<TopoDS_Shape> args;
 args.Append(BRepPrimAPI_MakeBox(1.,1.,1.).Shape());
 args.Append(BRepPrimAPI_MakeBox(gp_Pnt(.5,0,0),1.,1.,1.).Shape());
 ds.SetArguments(args);ds.Init(1e-7);
 for(int a=0;a<5;++a)for(int b=0;b<5;++b)run(ds,a,b);
}
