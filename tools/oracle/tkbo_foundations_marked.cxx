// Direct member calls for the complete IntTools_MarkedRangeSet interface at 3d097a0.
#include <IntTools_MarkedRangeSet.hxx>
#include <Standard_Failure.hxx>
void bo_marked_emit(const IntTools_MarkedRangeSet& set) {
 bo_int(set.Length());for(int index=1;index<=set.Length();++index){bo_range(set.Range(index));bo_int(set.Flag(index));}
}
bool bo_marked_insert(IntTools_MarkedRangeSet& set,const double* v) {
 if(int(v[0])==0)return set.InsertRange(v[1],v[2],int(v[3]));
 if(int(v[0])==1)return set.InsertRange(IntTools_Range(v[1],v[2]),int(v[3]));
 if(int(v[0])==2)return set.InsertRange(v[1],v[2],int(v[3]),int(v[4]));
 return set.InsertRange(IntTools_Range(v[1],v[2]),int(v[3]),int(v[4]));
}
void bo_marked_query(IntTools_MarkedRangeSet& set,double value) {
 bo_int(set.GetIndex(value));bo_int(set.GetIndex(value,true));bo_int(set.GetIndex(value,false));
 const auto& indices=set.GetIndices(value);bo_int(indices.Length());for(int index:indices)bo_int(index);
}
void bo_marked_case(const std::vector<double>& v) {
 const int count=int(v[0]);NCollection_Array1<double> bounds(-2,count-3);
 for(int i=0;i<count;++i)bounds.SetValue(i-2,v[i+1]);
 IntTools_MarkedRangeSet set(bounds,int(v[count+1]));bo_marked_emit(set);
 for(size_t i=count+2;i+4<v.size();i+=5) {
  try{bo_int(bo_marked_insert(set,&v[i]));}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  bo_marked_emit(set);
 }
 if(set.Length()>0){set.SetFlag(1,-7);bo_marked_emit(set);}
 for(double value:{-1.,0.,0.25,0.5,0.75,1.,1.5,2.,3.,4.})bo_marked_query(set,value);
 set.SetBoundaries(-2.,3.,9);bo_marked_emit(set);
 NCollection_Array1<double> reset(1,3);reset(1)=0.;reset(2)=0.;reset(3)=1.;set.SetRanges(reset,4);bo_marked_emit(set);
}
