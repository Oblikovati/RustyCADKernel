#include <NCollection_CellFilter.hxx>
struct ThrowingCellInspector {
 static constexpr int Dimension=2;using Target=int;using Point=gp_XY;int fail,parity;bool keep=false;
 static double Coord(int i,const Point& p){return p.Coord(i+1);}
 static bool IsEqual(int a,int b){return a==b;}
 NCollection_CellFilter_Action Inspect(int target){ga_int(target);if(keep)return CellFilter_Keep;if(target==fail)throw Standard_OutOfRange("callback failure");return (target+parity)%2==0?CellFilter_Purge:CellFilter_Keep;}
};
void mesh_cell_failure(std::istream& input){int fail,parity,range;input>>fail>>parity>>range;NCollection_CellFilter<ThrowingCellInspector> filter(1.);for(int i=1;i<=4;++i)filter.Add(i,gp_XY(0,0),gp_XY(1,1));ThrowingCellInspector inspector{fail,parity};try{if(range)filter.Inspect(gp_XY(0,0),gp_XY(1,1),inspector);else filter.Inspect(gp_XY(0,0),inspector);std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}inspector.keep=true;filter.Inspect(gp_XY(0,0),gp_XY(1,1),inspector);ga_end();}
