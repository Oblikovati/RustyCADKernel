#include <BOPTools_PairSelector.hxx>
#include <iostream>
void run(int same,int shared,int repeat,int offset){
 BVH_BoxSet<double,3,int> first,second;for(int i=0;i<12;i++){first.Add(100-i,BVH_Box<double,3>(BVH_Vec3d(i,0,0),BVH_Vec3d(i+1.25,1,1)));second.Add(i%3,BVH_Box<double,3>(BVH_Vec3d(i+offset*.5,0,0),BVH_Vec3d(i+offset*.5+.75,1,1)));}
 BOPTools_PairSelector<3> selector;selector.SetBVHSets(&first,shared?&first:&second);selector.SetSame(same);int a=selector.Select(),b=-1;if(repeat==2)selector.Clear();if(repeat)b=selector.Select();
 double metric=42.;bool rejected=selector.RejectNode(BVH_Vec3d(0,0,0),BVH_Vec3d(1,1,1),BVH_Vec3d(2,0,0),BVH_Vec3d(3,1,1),metric);
 std::cout<<same<<' '<<shared<<' '<<repeat<<' '<<offset<<' '<<a<<' '<<b<<' '<<rejected<<' '<<metric<<' '<<selector.Pairs().Size();for(auto p:selector.Pairs())std::cout<<' '<<p.ID1<<' '<<p.ID2;selector.Sort();for(auto p:selector.Pairs())std::cout<<' '<<p.ID1<<' '<<p.ID2;std::cout<<'\n';
}
int main(){for(int a=0;a<2;a++)for(int s=0;s<2;s++)for(int r=0;r<3;r++)for(int o=0;o<3;o++)run(a,s,r,o);}
