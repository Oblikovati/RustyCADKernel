#include <ShapeFix_Face.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <iostream>
TopoDS_Wire rectangle(double x,double y,double size,bool close=true){
 BRepBuilderAPI_MakePolygon maker;maker.Add(gp_Pnt(x,y,0));maker.Add(gp_Pnt(x+size,y,0));maker.Add(gp_Pnt(x+size,y+size,0));maker.Add(gp_Pnt(x,y+size,0));if(close)maker.Close();return maker.Wire();
}
int count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind){int n=0;if(shape.IsNull())return n;for(TopExp_Explorer exp(shape,kind);exp.More();exp.Next())++n;return n;}
void run(const char* name,int mode){
 BRep_Builder builder;TopoDS_Face face;builder.MakeFace(face,new Geom_Plane(gp_Pln()),1e-7);
 auto first=rectangle(0,0,4,mode!=2),second=rectangle(10,0,4);if(mode==3)builder.MakeWire(first);
 builder.Add(face,first);if(mode!=8)builder.Add(face,second);
 NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> groups;
 NCollection_List<TopoDS_Shape> holes;
 if(mode==5||mode==6){auto hole=rectangle(1,1,1);if(mode==6)hole.Reverse();builder.Add(face,hole);holes.Append(hole);}
 groups.Bind(first,holes);if(mode!=1&&mode!=8)groups.Bind(second,NCollection_List<TopoDS_Shape>());
 if(mode==4)face.Reverse();
 if(mode==7){auto ignored=rectangle(20,0,1);ignored.Orientation(TopAbs_INTERNAL);builder.Add(face,ignored);}
 ShapeFix_Face tool(face);Handle(ShapeBuild_ReShape) context=new ShapeBuild_ReShape;if(mode==9)tool.SetContext(context);
 bool changed=tool.FixSplitFace(groups);const auto result=tool.Result();
 std::cout<<name<<' '<<changed<<' '<<result.IsNull()<<' '<<count(result,TopAbs_FACE)<<' '<<count(result,TopAbs_WIRE)<<' '<<tool.Face().IsSame(face)<<' '<<tool.Face().Orientation()<<' '<<tool.Status(ShapeExtend_DONE)<<' '<<context->IsRecorded(face);
 if(!result.IsNull()){for(TopExp_Explorer exp(result,TopAbs_FACE);exp.More();exp.Next()){std::cout<<' '<<exp.Current().Orientation();for(TopoDS_Iterator it(exp.Current(),false);it.More();it.Next())std::cout<<' '<<it.Value().Orientation();}}
 std::cout<<'\n';
}
int main(){run("two_outer",0);run("unmapped",1);run("open",2);run("empty",3);run("reversed_face",4);run("forward_hole",5);run("reversed_hole",6);run("internal_ignored",7);run("single",8);run("context",9);}
