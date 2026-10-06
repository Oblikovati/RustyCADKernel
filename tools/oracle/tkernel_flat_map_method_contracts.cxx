// Pinned FlatMap slot pointers and iterator assignment/increment behavior.
#include <NCollection_FlatMap.hxx>
#include <iostream>
struct Key {
  int identity;
  const char* label;
};
struct IdentityPolicy {
  size_t operator()(const Key& key) const { return key.identity; }
  bool operator()(const Key& a,const Key& b) const { return a.identity==b.identity; }
};
using Map=NCollection_FlatMap<Key,IdentityPolicy>;
static void pointers() {
  Map map;
  Key lookup{7,"lookup"};
  std::cout << "missing_pointer " << (map.Seek(lookup)==nullptr) << ' ' << (map.ChangeSeek(lookup)==nullptr) << '\n';
  map.Add(Key{7,"stored"});
  const Key* pointer=map.Seek(lookup);
  std::cout << "slot_identity " << (pointer==map.ChangeSeek(lookup)) << ' ' << (pointer==&map.Contained(lookup)->get()) << '\n';
  map.ChangeSeek(lookup)->label="changed";
  std::cout << "key_mutation " << (pointer==map.Seek(lookup)) << ' ' << map.Seek(lookup)->label << '\n';
  map.Clear(false);
  std::cout << "retained_empty " << (map.Capacity()>0) << ' ' << (map.ChangeSeek(lookup)==nullptr) << '\n';
}
static void cursorContracts() {
  Map map;map.Add(Key{7,"stored"});
  Map::Iterator source(map),cursor;
  const bool returned=&(cursor=source)==&cursor;
  std::cout << "cursor_assignment " << returned << ' ' << cursor.IsEqual(source) << ' ' << (cursor.operator->()==map.Seek(Key{7,"lookup"})) << '\n';
  const bool oldEqual=cursor++==source;
  std::cout << "cursor_postfix " << oldEqual << ' ' << cursor.More() << ' ' << (cursor==map.end()) << '\n';
  cursor=source;
  const bool self=&(cursor=cursor)==&cursor;
  const bool prefix=&(++cursor)==&cursor;
  std::cout << "cursor_prefix " << self << ' ' << prefix << ' ' << (cursor==Map::Iterator()) << '\n';
  auto old=cursor++;
  std::cout << "exhausted_postfix " << (old==cursor) << '\n';
  bool failure=false;
  try {cursor.Key();} catch(const Standard_OutOfRange&) {failure=true;}
  std::cout << "exhausted_access " << failure << '\n';
}
static void rebindAndSelf() {
  Map first,second;first.Add(Key{7,"first"});second.Add(Key{8,"second"});
  Map::Iterator cursor(first),other(second);
  cursor.Next();
  const bool distinct=!cursor.IsEqual(other);
  cursor=other;
  std::cout << "rebind " << distinct << ' ' << cursor.Key().identity << ' ' << !cursor.IsEqual(Map::Iterator(first)) << '\n';
  const Key* pointer=first.Seek(Key{7,"lookup"});
  const bool identity=&(first=first)==&first;
  std::cout << "self_assignment " << identity << ' ' << (first.Seek(Key{7,"lookup"})==pointer) << ' ' << first.Size() << '\n';
}
int main(){pointers();cursorContracts();rebindAndSelf();}
