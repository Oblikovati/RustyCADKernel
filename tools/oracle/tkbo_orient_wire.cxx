#include <BOPTools_AlgoTools.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Iterator.hxx>
#include <iostream>
#include <vector>
void run(int mode,int mask,int orientation){
 BRep_Builder builder;std::vector<TopoDS_Vertex> vertices(4);for(int i=0;i<4;i++)builder.MakeVertex(vertices[i],gp_Pnt(i,0,0),1e-7);
 int links[4][3][2]={{{0,1},{1,2},{2,3}},{{0,1},{1,2},{2,0}},{{0,1},{1,2},{1,3}},{{0,0},{1,2},{2,3}}};std::vector<TopoDS_Edge> edges;
 TopoDS_Wire wire;builder.MakeWire(wire);
 int count=mode==5?0:mode==4?1:3;
 for(int i=0;i<count;i++){TopoDS_Edge edge;builder.MakeEdge(edge);if(mode!=4){auto first=vertices[links[mode][i][0]],last=vertices[links[mode][i][1]];first.Orientation(TopAbs_FORWARD);last.Orientation(TopAbs_REVERSED);builder.Add(edge,first);builder.Add(edge,last);}edge.Orientation(mask&(1<<i)?TopAbs_REVERSED:TopAbs_FORWARD);edges.push_back(edge);builder.Add(wire,edge);}
 wire.Orientation(TopAbs_Orientation(orientation));TopoDS_Shape output=wire;BOPTools_AlgoTools::OrientEdgesOnWire(output);
 std::cout<<mode<<' '<<mask<<' '<<orientation<<' '<<output.IsSame(wire)<<' '<<int(output.Orientation());int extent=0;for(TopoDS_Iterator it(output,false);it.More();it.Next())extent++;std::cout<<' '<<extent;
 for(TopoDS_Iterator it(output,false);it.More();it.Next()){int index=-1;for(int i=0;i<int(edges.size());i++)if(it.Value().IsSame(edges[i]))index=i;std::cout<<' '<<index<<' '<<int(it.Value().Orientation());}std::cout<<'\n';
}
int main(){for(int m=0;m<6;m++)for(int mask=0;mask<8;mask++)for(int o=0;o<4;o++)run(m,mask,o);}
