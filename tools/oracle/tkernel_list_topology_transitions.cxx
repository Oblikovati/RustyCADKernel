// Pinned native List state transitions, including both cursor links and allocator transfers.
#include <NCollection_List.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
struct List:NCollection_List<int>{
 using NCollection_List<int>::NCollection_List;
 int head() const {return pFirst()?static_cast<const ListNode*>(pFirst())->Value():0;}
 int tail() const {return pLast()?static_cast<const ListNode*>(pLast())->Value():0;}
};
int item(const NCollection_ListNode* node){return node?static_cast<const NCollection_TListNode<int>*>(node)->Value():0;}
List initial(int count,bool foreign=false){List list(foreign?new NCollection_IncAllocator:nullptr);for(int i=1;i<=count;++i)list.Append(i);return list;}
List::Iterator position(const List& list,int index){List::Iterator cursor(list);for(int i=0;i<index;++i)cursor.Next();return cursor;}
void print(int count,int pos,const char* group,int op,int donor,int foreign,bool error,const List& list,const List::Iterator& cursor){
 std::cout<<count<<' '<<pos<<' '<<group<<' '<<op<<' '<<donor<<' '<<foreign<<' '<<error<<" | "<<list.Size()<<' '<<list.head()<<' '<<list.tail()<<" :";
 for(auto it=list.cbegin();it.More();it.Next())std::cout<<' '<<it.Value();
 std::cout<<" | "<<item(cursor.myCurrent)<<' '<<item(cursor.myPrevious)<<'\n';
}
void positioned(int count,int pos){
 for(int op=0;op<4;++op){auto list=initial(count);auto cursor=position(list,pos);bool error=false;
  try{switch(op){case 0:list.Append(9,cursor);break;case 1:list.EmplaceBefore(cursor,9);break;case 2:list.EmplaceAfter(cursor,9);break;case 3:list.Remove(cursor);break;}}catch(const Standard_Failure&){error=true;}
  print(count,pos,"cursor",op,0,0,error,list,cursor);
 }
}
void transfers(int count,int pos){
 for(int donor=0;donor<=3;++donor)for(int foreign=0;foreign<2;++foreign)for(int op=0;op<4;++op){auto list=initial(count);auto source=initial(donor,foreign);auto cursor=position(list,pos);bool error=false;
  try{switch(op){case 0:list.Append(source);break;case 1:list.Prepend(source);break;case 2:list.InsertBefore(source,cursor);break;case 3:list.InsertAfter(source,cursor);break;}}catch(const Standard_Failure&){error=true;}
  print(count,pos,"splice",op,donor,foreign,error,list,cursor);
  print(count,pos,"source",op,donor,foreign,error,source,List::Iterator());
 }
}
void simple(int count){
 for(int op=0;op<9;++op){auto list=initial(count);bool error=false;auto cursor=position(list,count);
  try{switch(op){case 0:list.Append(9);break;case 1:list.Prepend(9);break;case 2:list.Reverse();break;case 3:list.Clear();break;case 4:list.RemoveFirst();break;case 5:list.Append(list);break;case 6:list.Prepend(list);break;case 7:list.InsertBefore(list,cursor);break;case 8:list.Assign(list);break;}}catch(const Standard_Failure&){error=true;}
  print(count,0,"simple",op,0,0,error,list,List::Iterator());
 }
}
void ownership(int count){
 auto list=initial(count);List copied(list);print(count,0,"copy",0,0,0,false,copied,List::Iterator());
 List moved(std::move(list));print(count,0,"move",0,0,0,false,moved,List::Iterator());print(count,0,"moved_from",0,0,0,false,list,List::Iterator());
 auto target=initial(2,true);target=std::move(moved);print(count,0,"move_assign",0,0,1,false,target,List::Iterator());print(count,0,"assigned_from",0,0,0,false,moved,List::Iterator());
}
int main(){for(int count=0;count<=4;++count){simple(count);ownership(count);for(int pos=0;pos<=count;++pos){positioned(count,pos);transfers(count,pos);}}}
