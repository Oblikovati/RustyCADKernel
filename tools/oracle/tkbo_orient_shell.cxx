#include <BOPTools_AlgoTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
#include <vector>
void run(int mode,int mask,int orientation){
 BRep_Builder builder;TopoDS_Shell shell;builder.MakeShell(shell);std::vector<TopoDS_Shape> faces;
 if(mode==2){TopoDS_Face face;builder.MakeFace(face);faces.push_back(face);}
 else if(mode!=4){TopoDS_Shape source;if(mode==1)source=BRepPrimAPI_MakeCylinder(2.,3.).Shape();else if(mode==3)source=BRepPrimAPI_MakeSphere(2.).Shape();else source=BRepPrimAPI_MakeBox(1.,2.,3.).Shape();for(TopExp_Explorer ex(source,TopAbs_FACE);ex.More();ex.Next())faces.push_back(ex.Current());}
 for(int i=0;i<int(faces.size());i++){auto face=faces[i];if(mask&(1<<(i%3)))face.Reverse();builder.Add(shell,face);if(mode==5)builder.Add(shell,face.Reversed());}
 shell.Orientation(TopAbs_Orientation(orientation));TopoDS_Shape output=shell;BOPTools_AlgoTools::OrientFacesOnShell(output);
 std::cout<<mode<<' '<<mask<<' '<<orientation<<' '<<output.IsSame(shell)<<' '<<int(output.Orientation());int count=0;for(TopoDS_Iterator it(output,false);it.More();it.Next())count++;std::cout<<' '<<count;
 for(TopoDS_Iterator it(output,false);it.More();it.Next()){int index=-1;for(int i=0;i<int(faces.size());i++)if(it.Value().IsSame(faces[i]))index=i;std::cout<<' '<<index<<' '<<int(it.Value().Orientation());}std::cout<<'\n';
}
int main(){for(int m=0;m<6;m++)for(int mask=0;mask<8;mask++)for(int o=0;o<4;o++)run(m,mask,o);}
