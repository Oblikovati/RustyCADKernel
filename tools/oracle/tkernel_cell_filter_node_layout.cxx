#include <NCollection_CellFilter.hxx>
#include <cstdint>
#include <iostream>
struct alignas(64) Aligned { unsigned char value; };
template<class T> struct Inspector {
 using Target=T;using Point=double;static constexpr int Dimension=1;
 static double Coord(int,const Point&p){return p;}
 static bool IsEqual(const T&,const T&){return false;}
 NCollection_CellFilter_Action Inspect(const T&){return CellFilter_Keep;}
};
template<class T> void row(const char*label){
 using Filter=NCollection_CellFilter<Inspector<T>>;using Node=typename Filter::ListNode;
 using Cell=typename Filter::Cell;using MapNode=typename Filter::CellMap::MapNode;
 std::cout<<label<<" objectnode "<<sizeof(Node)<<' '<<alignof(Node)<<' '<<offsetof(Node,Object)<<' '<<offsetof(Node,Next)
 <<" cell "<<sizeof(Cell)<<' '<<alignof(Cell)<<' '<<offsetof(Cell,index)<<' '<<offsetof(Cell,Objects)
 <<" mapnode "<<sizeof(MapNode)<<' '<<alignof(MapNode)<<" index "<<sizeof(typename Filter::CellIndex)<<'\n';
}
int main(){row<unsigned char>("u8");row<int>("i32");row<Aligned>("align64");}
