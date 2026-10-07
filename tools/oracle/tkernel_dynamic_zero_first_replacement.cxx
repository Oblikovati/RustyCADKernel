// OCCT 3d097a0: retained slot zero remains live under a wrapped zero stride.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <limits>
#include <sstream>
#include <stdexcept>
struct Payload {
  int value=0; std::ostringstream* events=nullptr;
  Payload()=default;
  Payload(int v,std::ostringstream& log):value(v),events(&log) {
    log<<"C"<<v<<",";if(v==-1)throw std::runtime_error("constructor");
  }
  Payload(const Payload& other):value(other.value),events(other.events) {
    *events<<"K"<<value<<",";if(value==-2)throw std::runtime_error("copy");
  }
  Payload(Payload&& other):value(other.value),events(other.events) {
    *events<<"M"<<value<<",";other.value=-9;if(value==-3)throw std::runtime_error("move");
  }
  Payload& operator=(Payload&& other) {
    *events<<"A"<<value<<":"<<other.value<<",";value=other.value;
    if(value==99)throw std::runtime_error("assignment");return *this;
  }
  ~Payload(){if(events)*events<<"D"<<value<<",";}
};
template<class T> static NCollection_DynamicArray<T> retained() {
  NCollection_DynamicArray<T> array(2);array.Appended();array.Appended();array.Appended();
  array.Clear(false);array.SetIncrement(std::numeric_limits<size_t>::max());return array;
}
static void row(const char* name,NCollection_DynamicArray<Payload>& array,std::ostringstream& log,bool caught) {
  std::cout<<name<<" caught="<<caught<<" size="<<array.Size()<<" value="<<(array.IsEmpty()?0:array.First().value)
           <<" events="<<log.str()<<'\n';log.str("");log.clear();
}
static void emplaceRows() {
  auto array=retained<Payload>();std::ostringstream log;bool caught=false;
  try{array.EmplaceValue(size_t(0),-1,log);}catch(const std::runtime_error&){caught=true;}
  row("new_ctor_throw",array,log,caught);array.EmplaceValue(size_t(0),10,log);row("new",array,log,false);
  array.EmplaceValue(0,20,log);row("replace",array,log,false);
  try{array.EmplaceValue(0,-1,log);}catch(const std::runtime_error&){}
  row("existing_ctor_throw",array,log,true);
  try{array.EmplaceValue(size_t(0),99,log);}catch(const std::runtime_error&){}
  row("assignment_throw",array,log,true);array.EmplaceValue(0,21,log);array.EraseLast();
  row("erase_recovery",array,log,false);array.SetIncrement(size_t(2));array.EmplaceValue(0,22,log);
  row("reset",array,log,false);array.Clear(true);row("reset_clear",array,log,false);
}
static void borrowedRows() {
  auto array=retained<Payload>();std::ostringstream log;Payload donor(30,log);log.str("");
  array.SetValue(size_t(0),donor);row("borrow_new",array,log,false);
  donor.value=31;array.SetValue(0,donor);row("borrow_replace",array,log,false);
  donor.value=-2;try{array.SetValue(size_t(0),donor);}catch(const std::runtime_error&){}
  row("borrow_throw",array,log,true);array.EraseLast();
}
static void movedRows() {
  auto array=retained<Payload>();std::ostringstream log;Payload donor(40,log);log.str("");
  array.SetValue(size_t(0),std::move(donor));row("move_new",array,log,false);
  std::cout<<"move_donor="<<donor.value<<'\n';donor.value=41;
  array.SetValue(0,std::move(donor));row("move_replace",array,log,false);
  donor.value=-3;try{array.SetValue(size_t(0),std::move(donor));}catch(const std::runtime_error&){}
  row("move_throw",array,log,true);std::cout<<"throw_donor="<<donor.value<<'\n';array.EraseLast();
}
static void internalRows() {
  auto array=retained<Payload>();std::ostringstream log;array.EmplaceValue(0,50,log);log.str("");
  array.SetValue(size_t(0),array.First());row("self_copy",array,log,false);
  array.SetValue(0,std::move(array.ChangeFirst()));row("self_move",array,log,false);
  array.ChangeFirst().value=-3;
  try{array.SetValue(size_t(0),std::move(array.ChangeFirst()));}catch(const std::runtime_error&){}
  row("self_move_throw",array,log,true);array.EraseLast();
}
static void cachedRows() {
  auto array=retained<int>();array.EmplaceValue(size_t(0),7);array[size_t(0)]=8;
  const auto& sharedArray=array;NCollection_DynamicArray<int>::const_iterator shared(sharedArray,size_t(0));
  std::cout<<"const_capture value="<<shared.Value()<<" same="<<(&shared.Value()==&array.First())<<'\n';
  NCollection_DynamicArray<int>::iterator cursor(array,size_t(0));auto* original=&array.First();
  std::cout<<"capture value="<<cursor.Value()<<" same="<<(&cursor.Value()==original)<<" more="<<cursor.More()<<'\n';
  ++cursor;std::cout<<"advance more="<<cursor.More()<<'\n';--cursor;cursor.ChangeValue()=9;
  std::cout<<"retreat value="<<array.First()<<'\n';auto target=std::move(array);
  array.SetIncrement(size_t(2));array.Append(101);array.Append(102);
  std::cout<<"move value="<<cursor.Value()<<" target="<<target.First()<<'\n';cursor+=0;
  std::cout<<"reload value="<<cursor.Value()<<" same="<<(&cursor.Value()==&array.First())<<'\n';
  ++cursor;std::cout<<"snapshot more="<<cursor.More()<<" source_size="<<array.Size()<<'\n';target.EraseLast();
}
int main(){emplaceRows();borrowedRows();movedRows();internalRows();cachedRows();}
