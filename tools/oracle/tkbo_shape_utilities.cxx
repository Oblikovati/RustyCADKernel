#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <TopoDS_CompSolid.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Vertex.hxx>
TopoDS_Shape bare(int type){BRep_Builder b;switch(type){case 0:{TopoDS_Compound s;b.MakeCompound(s);return s;}case 1:{TopoDS_CompSolid s;b.MakeCompSolid(s);return s;}case 2:{TopoDS_Solid s;b.MakeSolid(s);return s;}case 3:{TopoDS_Shell s;b.MakeShell(s);return s;}case 4:{TopoDS_Face s;b.MakeFace(s);return s;}case 5:{TopoDS_Wire s;b.MakeWire(s);return s;}case 6:{TopoDS_Edge s;b.MakeEdge(s);return s;}default:{TopoDS_Vertex s;b.MakeVertex(s);return s;}}}
void dimensions(int mode,int ori){
 auto shape=bare(mode<8?mode:0);BRep_Builder b;
 if(mode>=8){auto nested=bare(0);auto vertex=bare(7);b.Add(nested,vertex);vertex.Orientation(TopAbs_REVERSED);b.Add(nested,vertex);b.Add(shape,nested);b.Add(shape,mode==8?bare(7):bare(6));}
 shape.Orientation(TopAbs_Orientation(ori));int min,max;BOPTools_AlgoTools::Dimensions(shape,min,max);
 NCollection_List<TopoDS_Shape> list,unique;NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> fence;
 BOPTools_AlgoTools::TreatCompound(shape,list);BOPTools_AlgoTools::TreatCompound(shape,unique,&fence);int before=unique.Size();BOPTools_AlgoTools::TreatCompound(shape,unique,&fence);
 std::cout<<mode<<' '<<ori<<' '<<min<<' '<<max<<' '<<BOPTools_AlgoTools::Dimension(shape)<<' '<<list.Size()<<' '<<before<<' '<<unique.Size();
 for(auto const& child:list)std::cout<<' '<<int(child.ShapeType())<<' '<<int(child.Orientation());std::cout<<'\n';
}
void shell_case(int mode,int ori){auto face=pcurve_fixture(0).face;BRep_Builder b;TopoDS_Shell shell;b.MakeShell(shell);
 if(mode==3||mode==4)face.Orientation(TopAbs_INTERNAL);
 if(mode!=0)b.Add(shell,face);
 if(mode==2||mode==4||mode==6){face.Orientation(TopAbs_FORWARD);b.Add(shell,face);}
 if(mode==6)b.Add(shell,face);
 if(mode==5)for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next())b.Degenerated(TopoDS::Edge(ex.Current()),true);
 if(mode==7)b.Add(shell,pcurve_fixture(0).face);
 shell.Orientation(TopAbs_Orientation(ori));std::cout<<mode<<' '<<ori<<' '<<BOPTools_AlgoTools::IsOpenShell(shell)<<'\n';}
int main(int argc,char**){if(argc>1){for(int mode=0;mode<8;mode++)for(int ori=0;ori<4;ori++)shell_case(mode,ori);return 0;}for(int mode=0;mode<10;mode++)for(int ori=0;ori<4;ori++)dimensions(mode,ori);}
