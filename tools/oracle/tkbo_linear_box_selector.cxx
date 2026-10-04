#include <BOPTools_BoxTree.hxx>
#include <iostream>
template<int D> void run(int query,int repeat){
 using Vec=typename BVH::VectorType<double,D>::Type;auto point=[](double x){Vec result;for(int i=0;i<D;i++)result[i]=i?0.:x;return result;};
 BOPTools_BoxSet<double,D,int> boxes;for(int i=0;i<12;i++)boxes.Add(100-i,BVH_Box<double,D>(point(i),point(i+.75)));
 BOPTools_BoxSelector<D> selector;selector.SetBVHSet(&boxes);
 double bounds[][2]={{-1.,20.},{2.25,5.25},{99.,100.},{4.,4.}};
 BVH_Box<double,D> box;if(query<4)box=BVH_Box<double,D>(point(bounds[query][0]),point(bounds[query][1]));selector.SetBox(box);
 int first=selector.Select(),second=-1;if(repeat==1)second=selector.Select();if(repeat==2){selector.Clear();second=selector.Select();}
 bool inside=true;bool rejected=selector.RejectNode(point(3.),point(4.),inside);
 std::cout<<D<<' '<<query<<' '<<repeat<<' '<<first<<' '<<second<<' '<<rejected<<' '<<inside<<' '<<selector.AcceptMetric(inside)<<' '<<selector.Indices().Extent();for(auto id:selector.Indices())std::cout<<' '<<id;std::cout<<'\n';
}
int main(){for(int q=0;q<5;q++)for(int r=0;r<3;r++){run<2>(q,r);run<3>(q,r);}}
