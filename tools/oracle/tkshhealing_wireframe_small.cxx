#include <ShapeFix_Wireframe.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <iostream>
TopoDS_Wire polygon(){return BRepBuilderAPI_MakePolygon(gp_Pnt(0,0,0),gp_Pnt(0.01,0,0),gp_Pnt(1,1,0),gp_Pnt(0,1,0),true).Wire();}
TopoDS_Shape fixture(int mode,TopoDS_Edge& first){
 BRep_Builder builder;auto wire=polygon();first=TopoDS::Edge(TopExp_Explorer(wire,TopAbs_EDGE).Current());
 if(mode==1)return first;
 if(mode==2||mode==3){builder.MakeWire(wire);builder.Add(wire,first);builder.Add(wire,mode==2?first:first.Reversed());}
 if(mode==4){builder.MakeWire(wire);return wire;}
 if(mode<5)return wire;
 TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Pln()),1e-7);builder.Add(face,wire);
 if(mode==6)face.Reverse();
 if(mode!=7)return face;
 auto other=TopoDS::Face(face.EmptyCopied());builder.Add(other,wire);TopoDS_Compound compound;builder.MakeCompound(compound);builder.Add(compound,face);builder.Add(compound,other);return compound;
}
void run(const char* name,int mode,bool preload=false){
 TopoDS_Edge first;auto shape=fixture(mode,first);ShapeFix_Wireframe tool(shape);tool.SetPrecision(0.1);
 NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> small,multiple;
 NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> edgeFaces,faceSmall;
 if(preload)small.Add(first);
 bool changed=tool.CheckSmallEdges(small,edgeFaces,faceSmall,multiple);int faceLinks=0,smallLinks=0,reverseLinks=0;
 std::cout<<name<<' '<<changed<<' '<<small.Extent()<<' '<<edgeFaces.Extent()<<' '<<faceSmall.Extent()<<' '<<multiple.Extent();
 for(decltype(edgeFaces)::Iterator it(edgeFaces);it.More();it.Next())for(NCollection_List<TopoDS_Shape>::Iterator faces(it.Value());faces.More();faces.Next()){++faceLinks;if(faces.Value().Orientation()==TopAbs_REVERSED)++reverseLinks;}
 for(decltype(faceSmall)::Iterator it(faceSmall);it.More();it.Next())smallLinks+=it.Value().Extent();
 std::cout<<' '<<faceLinks<<' '<<smallLinks<<' '<<reverseLinks<<' '<<tool.StatusSmallEdges(ShapeExtend_OK)<<' '<<tool.Shape().IsSame(shape)<<'\n';
}
void state(){
 ShapeFix_Wireframe tool;std::cout<<"default "<<tool.Shape().IsNull()<<' '<<tool.ModeDropSmallEdges()<<' '<<tool.LimitAngle()<<' '<<tool.StatusWireGaps(ShapeExtend_OK)<<' '<<tool.StatusSmallEdges(ShapeExtend_OK)<<'\n';
 tool.ModeDropSmallEdges()=true;tool.SetLimitAngle(0.25);tool.Load(polygon());tool.ClearStatuses();std::cout<<"loaded "<<tool.Shape().IsNull()<<' '<<tool.ModeDropSmallEdges()<<' '<<tool.LimitAngle()<<' '<<tool.StatusWireGaps(ShapeExtend_OK)<<' '<<tool.StatusSmallEdges(ShapeExtend_OK)<<'\n';
}
int main(){state();run("free",0);run("edge",1);run("duplicate",2);run("seam",3);run("empty",4);run("face",5);run("reversed_face",6);run("shared_faces",7);run("preloaded_edge",1,true);run("preloaded_empty",4,true);}
