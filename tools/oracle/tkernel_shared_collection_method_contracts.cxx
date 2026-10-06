// Native shared collection constructors, implicit special members and reference-count contracts.
#include <NCollection_HArray1.hxx>
#include <NCollection_HArray2.hxx>
#include <NCollection_HSequence.hxx>
#include <NCollection_Shared.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <string>
#include <utility>
#include <vector>
void array1Copy() {
  NCollection_HArray1<int> source(-2,0,7); source.IncrementRefCounter();
  NCollection_HArray1<int> copied(source),target(8,10,1); target.IncrementRefCounter();
  const int* before=target.Array1().Data(); target=source;
  std::cout<<"array1_copy "<<copied.GetRefCount()<<' '<<copied.Lower()<<' '<<copied.Upper()<<' '
    <<(copied.Array1().Data()!=source.Array1().Data())<<' '<<(target.Array1().Data()==before)<<' '
    <<target.Value(-2)<<' '<<target.GetRefCount()<<' '<<source.GetRefCount()<<'\n';
  source.DecrementRefCounter(); target.DecrementRefCounter();
}
void array1Move() {
  NCollection_HArray1<int> source(-2,0,7); const int* before=source.Array1().Data();
  NCollection_HArray1<int> moved(std::move(source)),target;
  const bool stable=moved.Array1().Data()==before;
  const int lower=source.Lower(),upper=source.Upper();
  const bool returned=&(target=std::move(moved))==&target;
  std::cout<<"array1_move "<<stable<<' '<<lower<<' '<<upper<<' '<<returned<<' '
    <<(target.Array1().Data()==before)<<' '<<moved.IsEmpty()<<'\n';
}
void array2CopyMove() {
  NCollection_HArray2<int> source(-1,0,4,5,7),copied(source),target(1,1,1,4,0);
  target.IncrementRefCounter(); target=source;
  const int* before=target.Array2().Data();
  NCollection_HArray2<int> moved(std::move(target));
  const bool stable=moved.Array2().Data()==before,empty=target.IsEmpty();
  const bool returned=&(target=std::move(moved))==&target;
  std::cout<<"array2 "<<copied.LowerRow()<<' '<<copied.LowerCol()<<' '
    <<(copied.Array2().Data()!=source.Array2().Data())<<' '<<target.GetRefCount()<<' '
    <<target.RowLength()<<' '<<target.ColLength()<<' '<<stable<<' '<<empty<<' '
    <<returned<<' '<<(target.Array2().Data()==before)<<'\n';
  target.DecrementRefCounter();
}
void buffer() {
  int values[]={10,20,30}; NCollection_HArray1<int> viewed(values[0],-2,0,true);
  const bool original=viewed.Array1().Data()==values,borrowed=!viewed.IsDeletable();
  viewed.ChangeArray1().SetValue(-1,99); NCollection_HArray1<int> copied(viewed);
  std::cout<<"buffer "<<original<<' '<<borrowed<<' '<<(!viewed.IsDeletable())<<' '
    <<copied.IsDeletable()<<' '<<(copied.Array1().Data()!=values)<<' '<<values[1]<<'\n';
  NCollection_Array1<int> base(values[0],-3,-2); NCollection_HArray1<int> fromBase(base);
  base.SetValue(-3,42);
  std::cout<<"base_copy "<<fromBase.Lower()<<' '<<fromBase.Upper()<<' '<<fromBase.Value(-3)<<'\n';
}
void sequence() {
  occ::handle<NCollection_BaseAllocator> sourceArena=new NCollection_IncAllocator(4096);
  occ::handle<NCollection_BaseAllocator> targetArena=new NCollection_IncAllocator(4096);
  NCollection_Sequence<int> sequence(sourceArena),other(targetArena); sequence.Append(7);
  NCollection_HSequence<int> source(sequence),target(other); source.IncrementRefCounter();
  NCollection_HSequence<int> copied(source); target.IncrementRefCounter(); target=source;
  std::cout<<"sequence_copy "<<copied.GetRefCount()<<' '<<(copied.Allocator()==sourceArena)<<' '
    <<(target.Allocator()==targetArena)<<' '<<target.First()<<' '<<target.GetRefCount()<<'\n';
  source.DecrementRefCounter(); target.DecrementRefCounter();
  const int* before=&source.First(); NCollection_HSequence<int> moved(std::move(source));
  const bool empty=source.IsEmpty(),stable=&moved.First()==before;
  const bool returned=&(source=std::move(moved))==&source;
  std::cout<<"sequence_move "<<empty<<' '<<stable<<' '<<returned<<' '<<moved.IsEmpty()<<' '
    <<(&source.First()==before)<<'\n';
}
struct Constructed {
  std::vector<std::string> items;
  Constructed()=default;
  Constructed(std::string& prefix,const std::string& suffix) { prefix+=suffix;items.push_back(prefix); }
};
void shared() {
  std::string prefix="a"; const std::string suffix="b";
  NCollection_Shared<Constructed> source(prefix,suffix); source.IncrementRefCounter();
  NCollection_Shared<Constructed> copied(source),target;target.IncrementRefCounter(); target=source;
  std::cout<<"shared "<<prefix<<' '<<copied.GetRefCount()<<' '<<copied.items.front()<<' '
    <<target.items.front()<<' '<<target.GetRefCount()<<'\n';
  source.DecrementRefCounter(); target.DecrementRefCounter();
}
int main() { array1Copy();array1Move();array2CopyMove();buffer();sequence();shared(); }
