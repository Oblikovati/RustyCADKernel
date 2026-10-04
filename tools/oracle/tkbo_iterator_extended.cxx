#include <BOPDS_DS.hxx>
#include <BOPDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_Map.hxx>
#include <iostream>
void output(BOPDS_Iterator& it){for(auto pair:{std::pair{TopAbs_VERTEX,TopAbs_VERTEX},std::pair{TopAbs_VERTEX,TopAbs_EDGE},std::pair{TopAbs_EDGE,TopAbs_EDGE},std::pair{TopAbs_VERTEX,TopAbs_FACE},std::pair{TopAbs_FACE,TopAbs_FACE}}){it.Initialize(pair.first,pair.second);std::cout<<it.ExpectedLength();while(it.More()){int a,b;it.Value(a,b);std::cout<<' '<<a<<' '<<b;it.Next();}std::cout<<'\n';}}
void run(int mode,int repeat){
 NCollection_List<TopoDS_Shape> args;args.Append(BRepPrimAPI_MakeBox(1.,1.,1.).Shape());args.Append(BRepPrimAPI_MakeBox(gp_Pnt(.5,0,0),1.,1.,1.).Shape());BOPDS_DS ds;ds.SetArguments(args);ds.Init(1e-7);
 BOPDS_Iterator it;it.SetDS(&ds);it.Prepare();NCollection_Map<int> selected;int first=-1,last=-1;
 for(int i=0;i<ds.NbSourceShapes();i++){auto type=ds.ShapeInfo(i).ShapeType();if(type==TopAbs_VERTEX){if(first<0)first=i;last=i;if(mode>=2 && mode<=3)selected.Add(i);}if(mode==3 && type==TopAbs_EDGE)selected.Add(i);}
 if(mode==1 || mode==4){selected.Add(first);ds.ChangeShapeInfo(first).ChangeBox().Enlarge(.75);}
 if(mode==4)ds.AddShapeSD(first,last);
 for(int i=0;i<repeat;i++)it.IntersectExt(selected);
 std::cout<<"case "<<mode<<' '<<repeat<<'\n';output(it);it.Prepare();output(it);
}
int main(){for(int mode=0;mode<5;mode++)for(int repeat=1;repeat<3;repeat++)run(mode,repeat);}
