// Native borrowed-key forwarding and FlatDataMap iterator contracts.
#include <NCollection_FlatDataMap.hxx>
#include <iostream>
#include <string>
#include <stdexcept>
static std::string events;
struct Key {
  int number;
  bool fail=false;
  explicit Key(int value):number(value){}
  Key(const Key& source):number(source.number),fail(source.fail){events+='K';if(fail)throw std::runtime_error("injected copy failure");}
  Key(Key&&)=default;
  Key& operator=(Key&&)=default;
};
struct Policy {
  size_t operator()(const Key& key) const {events+='H';return key.number;}
  bool operator()(const Key& a,const Key& b) const {events+='E';return a.number==b.number;}
};
struct Value {
  int number;
  explicit Value(int n):number(n){events+='C';}
};
static void forwarding() {
  NCollection_FlatDataMap<Key,Value,Policy> map;
  Key key(1);
  const bool inserted=map.Emplace(key,42);
  std::cout << "emplace_miss " << inserted << ' ' << events << '\n';events.clear();
  const bool replaced=map.Emplace(key,43);
  std::cout << "emplace_hit " << replaced << ' ' << events << '\n';events.clear();
  const bool kept=map.TryEmplace(key,99);
  std::cout << "try_emplace_hit " << kept << ' ' << events << '\n';events.clear();
  const int old=map.TryEmplaced(key,99).number;
  std::cout << "try_emplaced_hit " << old << ' ' << events << '\n';events.clear();
}
struct IntPolicy {
  size_t operator()(int value)const{return value;}
  bool operator()(int a,int b)const{return a==b;}
};
static void displacement() {
  NCollection_FlatDataMap<int,int,IntPolicy> map;
  map.Bind(0,10);map.Bind(1,20);
  const int eight=8,sixteen=16,twentyfour=24;
  std::cout << "displacement " << map.Emplaced(eight,80) << ' ' << map.TryEmplaced(sixteen,160) << ' ' << map.TryEmplace(twentyfour,240) << ' ' << map.Emplaced(eight,81) << ' ' << map.Find(1) << '\n';
}
static void failure() {
  NCollection_FlatDataMap<Key,Value,Policy> map;
  Key key(1);key.fail=true;
  try{map.Emplace(key,42);}catch(const std::runtime_error&){}
  std::cout << "copy_failure " << map.IsEmpty() << ' ' << map.Capacity() << ' ' << events << '\n';events.clear();
}
static void pointers() {
  NCollection_FlatDataMap<int,std::string> map;
  std::cout << "missing_pointer " << (map.Seek(1)==nullptr) << ' ' << (map.ChangeSeek(1)==nullptr) << '\n';
  map.Bind(1,"first");const auto* pointer=map.Seek(1);
  const bool same=pointer==map.ChangeSeek(1);
  map.ChangeFind(1)="replacement";
  std::cout << "stored_pointer " << same << ' ' << (map.Seek(1)==pointer) << '\n';
  map.Clear(false);std::cout << "cleared_pointer " << (map.Seek(1)==nullptr) << '\n';
}
static void readonlyIterator() {
  using Map=NCollection_FlatDataMap<int,int>;
  Map map;map.Bind(1,42);
  Map::const_iterator source=map.cbegin(),cursor;
  const bool assigned=&(cursor=source)==&cursor;
  const bool self=&(cursor=cursor)==&cursor;
  std::cout << "readonly_assignment " << assigned << ' ' << self << ' ' << (cursor==source) << ' ' << cursor.Key() << ' ' << (cursor.operator->()==map.Seek(1)) << '\n';
  const bool old=cursor++==source;
  const bool exhausted=!cursor.More();
  cursor=source;const bool prefix=&(++cursor)==&cursor;
  std::cout << "readonly_increment " << old << ' ' << exhausted << ' ' << prefix << ' ' << (cursor==Map::const_iterator()) << '\n';
}
static void mutableIterator() {
  using Map=NCollection_FlatDataMap<int,int>;
  Map map;map.Bind(1,42);map.Bind(2,43);
  Map::Iterator cursor(map);
  const int first=cursor.Value();
  const bool pointers=&cursor.Value()==&cursor.ChangeValue();
  const bool self=&(cursor=cursor)==&cursor;
  const bool old=*(cursor++)==first;
  const bool more=cursor.More();
  const bool prefix=&(++cursor)==&cursor;
  std::cout << "mutable_increment " << pointers << ' ' << self << ' ' << old << ' ' << more << ' ' << prefix << ' ' << cursor.More() << '\n';
  const int* pointer=map.Seek(1);const bool returned=&(map=map)==&map;
  std::cout << "self_assignment " << returned << ' ' << (pointer==map.Seek(1)) << ' ' << map.Find(1) << '\n';
}
int main(){forwarding();displacement();failure();pointers();readonlyIterator();mutableIterator();}
