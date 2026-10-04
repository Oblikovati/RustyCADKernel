#pragma once
#include "tkoffset_analyse_fixture.hxx"
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
TopoDS_Shape rectangle_pair(bool split,bool side=false){
 BRep_Builder builder;std::vector<TopoDS_Vertex> vertices;
 for(auto point:{gp_Pnt(0,0,0),gp_Pnt(1,0,0),gp_Pnt(2,0,0),gp_Pnt(0,1,0),gp_Pnt(1,1,0),gp_Pnt(2,1,0),gp_Pnt(1,.5,0),gp_Pnt(0,0,-1),gp_Pnt(2,0,-1)})vertices.push_back(BRepBuilderAPI_MakeVertex(point).Vertex());
 auto edge=[&](int first,int last){return BRepBuilderAPI_MakeEdge(vertices[first],vertices[last]).Edge();};
 std::vector<TopoDS_Edge> common;if(split){common.push_back(edge(1,6));common.push_back(edge(6,4));}else common.push_back(edge(1,4));
 auto bottomLeft=edge(0,1),bottomRight=edge(1,2);
 TopoDS_Wire left,right;builder.MakeWire(left);builder.MakeWire(right);
 builder.Add(left,bottomLeft);for(const auto& current:common)builder.Add(left,current);builder.Add(left,edge(4,3));builder.Add(left,edge(3,0));
 builder.Add(right,bottomRight);builder.Add(right,edge(2,5));builder.Add(right,edge(5,4));for(auto current=common.rbegin();current!=common.rend();++current)builder.Add(right,current->Reversed());
 TopoDS_Compound pair;builder.MakeCompound(pair);builder.Add(pair,BRepBuilderAPI_MakeFace(left,true).Face());builder.Add(pair,BRepBuilderAPI_MakeFace(right,true).Face());if(side){TopoDS_Wire boundary;builder.MakeWire(boundary);builder.Add(boundary,bottomRight.Reversed());builder.Add(boundary,bottomLeft.Reversed());builder.Add(boundary,edge(0,7));builder.Add(boundary,edge(7,8));builder.Add(boundary,edge(8,2));builder.Add(pair,BRepBuilderAPI_MakeFace(boundary,true).Face());}return pair;
}
