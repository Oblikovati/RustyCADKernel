// NCollection hash maps probe operations (twin of oracle_probe/ncollection/hash_maps.rs).
// Each request is a script of single-letter operations on one map of int keys; every
// operation prints its result (or `throw <ExceptionType>`), then the map's bucket count,
// extent and full Iterator traversal, which exposes OCCT's exact bucket order.
#include <NCollection_DataMap.hxx>
#include <NCollection_DefaultHasher.hxx>
#include <NCollection_DoubleMap.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_IndexedMap.hxx>
#include <NCollection_Map.hxx>
#include <NCollection_MapAlgo.hxx>
#include <NCollection_Primes.hxx>
#include <Standard_Failure.hxx>
#include <cstddef>
#include <istream>
#include <ostream>
#include <string>

template <class Action>
static void hm_guard(std::ostream& out, Action theAction)
{
  try
  {
    theAction();
  }
  catch (const Standard_Failure& theFailure)
  {
    out << "throw " << theFailure.ExceptionType();
  }
  out << ' ';
}

static int hm_int(std::istream& in)
{
  int aValue = 0;
  in >> aValue;
  return aValue;
}

static size_t hm_size(std::istream& in)
{
  size_t aValue = 0;
  in >> aValue;
  return aValue;
}

static void hm_datamap_op(NCollection_DataMap<int, int>& map, char code, std::istream& in, std::ostream& out)
{
  if (code == 'b') { const int k = hm_int(in), v = hm_int(in); out << map.Bind(k, v); }
  else if (code == 't') { const int k = hm_int(in), v = hm_int(in); out << map.TryBind(k, v); }
  else if (code == 's') { const int k = hm_int(in), v = hm_int(in); out << *map.Bound(k, v); }
  else if (code == 'T') { const int k = hm_int(in), v = hm_int(in); out << map.TryBound(k, v); }
  else if (code == 'e') { const int k = hm_int(in), v = hm_int(in); out << map.Emplace(k, v); }
  else if (code == 'E') { const int k = hm_int(in), v = hm_int(in); out << map.Emplaced(k, v); }
  else if (code == 'y') { const int k = hm_int(in), v = hm_int(in); out << map.TryEmplace(k, v); }
  else if (code == 'Y') { const int k = hm_int(in), v = hm_int(in); out << map.TryEmplaced(k, v); }
  else if (code == 'u') { out << map.UnBind(hm_int(in)); }
  else if (code == 'i') { out << map.IsBound(hm_int(in)); }
  else if (code == 'r') { map.ReSize(hm_size(in)); out << '-'; }
  else if (code == 'c') { map.Clear(hm_int(in) != 0); out << '-'; }
  else if (code == 'f') { const int k = hm_int(in); hm_guard(out, [&] { out << map.Find(k); }); }
  else if (code == 'F') { int v = -1; const bool f = map.Find(hm_int(in), v); out << f << ':' << v; }
  else if (code == 'q') { const int* v = map.Seek(hm_int(in)); if (v) out << *v; else out << "null"; }
  else if (code == 'x') { auto p = map.Contained(hm_int(in)); if (p) out << p->first.get() << ':' << p->second.get(); else out << "none"; }
  else if (code == 'A') { NCollection_DataMap<int, int> aCopy(map); map.Assign(aCopy); out << '-'; }
}

static void hm_datamap(std::istream& in, std::ostream& out)
{
  NCollection_DataMap<int, int> map(hm_size(in));
  char code;
  while (in >> code) { hm_datamap_op(map, code, in, out); out << ' '; }
  out << "| n=" << map.NbBuckets() << " e=" << map.Extent() << " s=" << map.Size() << " :";
  for (NCollection_DataMap<int, int>::Iterator it(map); it.More(); it.Next())
    out << ' ' << it.Key() << '=' << it.Value();
}

static void hm_map_op(NCollection_Map<int>& map, char code, std::istream& in, std::ostream& out)
{
  if (code == 'a') { out << map.Add(hm_int(in)); }
  else if (code == 'A') { out << map.Added(hm_int(in)); }
  else if (code == 'e') { out << map.Emplace(hm_int(in)); }
  else if (code == 'E') { out << map.Emplaced(hm_int(in)); }
  else if (code == 'y') { out << map.TryEmplace(hm_int(in)); }
  else if (code == 'Y') { out << map.TryEmplaced(hm_int(in)); }
  else if (code == 'm') { out << map.Remove(hm_int(in)); }
  else if (code == 'i') { out << map.Contains(hm_int(in)); }
  else if (code == 'x') { auto k = map.Contained(hm_int(in)); if (k) out << k->get(); else out << "none"; }
  else if (code == 'r') { map.ReSize(hm_size(in)); out << '-'; }
  else if (code == 'c') { map.Clear(hm_int(in) != 0); out << '-'; }
  else if (code == 'C') { NCollection_Map<int> aCopy(map); map.Assign(aCopy); out << '-'; }
}

static void hm_map_print(const NCollection_Map<int>& map, std::ostream& out)
{
  out << "| n=" << map.NbBuckets() << " e=" << map.Extent() << " :";
  for (NCollection_Map<int>::Iterator it(map); it.More(); it.Next())
    out << ' ' << it.Key();
}

static void hm_map(std::istream& in, std::ostream& out)
{
  NCollection_Map<int> map(hm_size(in));
  char code;
  while (in >> code) { hm_map_op(map, code, in, out); out << ' '; }
  hm_map_print(map, out);
}

static void hm_read_keys(NCollection_Map<int>& map, std::istream& in)
{
  const int aCount = hm_int(in);
  for (int i = 0; i < aCount; ++i)
    map.Add(hm_int(in));
}

static void hm_algo(std::istream& in, std::ostream& out)
{
  std::string name;
  in >> name;
  NCollection_Map<int> result, left, right;
  hm_read_keys(result, in);
  hm_read_keys(left, in);
  hm_read_keys(right, in);
  namespace algo = NCollection_MapAlgo;
  if (name == "union") algo::Union(result, left, right);
  else if (name == "intersection") algo::Intersection(result, left, right);
  else if (name == "subtraction") algo::Subtraction(result, left, right);
  else if (name == "difference") algo::Difference(result, left, right);
  else if (name == "unite") out << algo::Unite(left, right);
  else if (name == "intersect") out << algo::Intersect(left, right);
  else if (name == "subtract") out << algo::Subtract(left, right);
  else if (name == "differ") out << algo::Differ(left, right);
  else if (name == "has_intersection") out << algo::HasIntersection(left, right);
  else if (name == "contains") out << algo::Contains(left, right);
  else if (name == "is_equal") out << algo::IsEqual(left, right);
  out << ' ';
  hm_map_print(result, out);
  out << ' ';
  hm_map_print(left, out);
}

static void hm_indexed_op(NCollection_IndexedMap<int>& map, char code, std::istream& in, std::ostream& out)
{
  if (code == 'a') { out << map.Add(hm_int(in)); }
  else if (code == 'A') { out << map.Added(hm_int(in)); }
  else if (code == 'e') { out << map.Emplace(hm_int(in)); }
  else if (code == 'y') { out << map.TryEmplace(hm_int(in)); }
  else if (code == 's') { const size_t i = hm_size(in); const int k = hm_int(in); hm_guard(out, [&] { map.Substitute(i, k); out << '-'; }); }
  else if (code == 'w') { const size_t i = hm_size(in), j = hm_size(in); hm_guard(out, [&] { map.Swap(i, j); out << '-'; }); }
  else if (code == 'l') { hm_guard(out, [&] { map.RemoveLast(); out << '-'; }); }
  else if (code == 'R') { const size_t i = hm_size(in); hm_guard(out, [&] { map.RemoveFromIndex(i); out << '-'; }); }
  else if (code == 'm') { out << map.RemoveKey(hm_int(in)); }
  else if (code == 'f') { const size_t i = hm_size(in); hm_guard(out, [&] { out << map.FindKey(i); }); }
  else if (code == 'n') { out << map.FindIndex(hm_int(in)); }
  else if (code == 'x') { auto k = map.Contained(hm_int(in)); if (k) out << k->get(); else out << "none"; }
  else if (code == 'r') { map.ReSize(hm_size(in)); out << '-'; }
  else if (code == 'c') { map.Clear(hm_int(in) != 0); out << '-'; }
  else if (code == 'C') { NCollection_IndexedMap<int> aCopy(map); map.Assign(aCopy); out << '-'; }
}

static void hm_indexed(std::istream& in, std::ostream& out)
{
  NCollection_IndexedMap<int> map(hm_size(in));
  char code;
  while (in >> code) { hm_indexed_op(map, code, in, out); out << ' '; }
  out << "| n=" << map.NbBuckets() << " e=" << map.Extent() << " :";
  for (NCollection_IndexedMap<int>::Iterator it(map); it.More(); it.Next())
    out << ' ' << it.Index() << '=' << it.Value() << '@' << map.FindIndex(it.Value());
}

static void hm_indexed_data_op(NCollection_IndexedDataMap<int, int>& map, char code, std::istream& in, std::ostream& out)
{
  if (code == 'a') { const int k = hm_int(in), v = hm_int(in); out << map.Add(k, v); }
  else if (code == 'b') { const int k = hm_int(in), v = hm_int(in); out << map.Bind(k, v); }
  else if (code == 't') { const int k = hm_int(in), v = hm_int(in); out << map.TryBind(k, v); }
  else if (code == 'B') { const int k = hm_int(in), v = hm_int(in); out << *map.Bound(k, v); }
  else if (code == 'T') { const int k = hm_int(in), v = hm_int(in); out << map.TryBound(k, v); }
  else if (code == 'e') { const int k = hm_int(in), v = hm_int(in); out << map.Emplace(k, v); }
  else if (code == 'y') { const int k = hm_int(in), v = hm_int(in); out << map.TryEmplace(k, v); }
  else if (code == 's') { const size_t i = hm_size(in); const int k = hm_int(in), v = hm_int(in); hm_guard(out, [&] { map.Substitute(i, k, v); out << '-'; }); }
  else if (code == 'w') { const size_t i = hm_size(in), j = hm_size(in); hm_guard(out, [&] { map.Swap(i, j); out << '-'; }); }
  else if (code == 'l') { hm_guard(out, [&] { map.RemoveLast(); out << '-'; }); }
  else if (code == 'R') { const size_t i = hm_size(in); hm_guard(out, [&] { map.RemoveFromIndex(i); out << '-'; }); }
  else if (code == 'm') { map.RemoveKey(hm_int(in)); out << '-'; }
  else if (code == 'f') { const size_t i = hm_size(in); hm_guard(out, [&] { out << map.FindFromIndex(i); }); }
  else if (code == 'F') { const int k = hm_int(in); hm_guard(out, [&] { out << map.FindFromKey(k); }); }
  else if (code == 'G') { int v = -1; const bool f = map.FindFromKey(hm_int(in), v); out << f << ':' << v; }
  else if (code == 'q') { const int* v = map.Seek(hm_int(in)); if (v) out << *v; else out << "null"; }
  else if (code == 'r') { map.ReSize(hm_size(in)); out << '-'; }
  else if (code == 'c') { map.Clear(hm_int(in) != 0); out << '-'; }
  else if (code == 'C') { NCollection_IndexedDataMap<int, int> aCopy(map); map.Assign(aCopy); out << '-'; }
}

static void hm_indexed_data(std::istream& in, std::ostream& out)
{
  NCollection_IndexedDataMap<int, int> map(hm_size(in));
  char code;
  while (in >> code) { hm_indexed_data_op(map, code, in, out); out << ' '; }
  out << "| n=" << map.NbBuckets() << " e=" << map.Extent() << " :";
  for (NCollection_IndexedDataMap<int, int>::Iterator it(map); it.More(); it.Next())
    out << ' ' << it.Index() << '=' << it.Key() << ':' << it.Value() << '@' << map.FindIndex(it.Key());
}

static void hm_double_op(NCollection_DoubleMap<int, int>& map, char code, std::istream& in, std::ostream& out)
{
  if (code == 'b') { const int a = hm_int(in), b = hm_int(in); hm_guard(out, [&] { map.Bind(a, b); out << '-'; }); }
  else if (code == 't') { const int a = hm_int(in), b = hm_int(in); out << map.TryBind(a, b); }
  else if (code == 'u') { out << map.UnBind1(hm_int(in)); }
  else if (code == 'U') { out << map.UnBind2(hm_int(in)); }
  else if (code == 'q') { const int a = hm_int(in), b = hm_int(in); out << map.AreBound(a, b); }
  else if (code == 'i') { out << map.IsBound1(hm_int(in)); }
  else if (code == 'I') { out << map.IsBound2(hm_int(in)); }
  else if (code == 'f') { const int a = hm_int(in); hm_guard(out, [&] { out << map.Find1(a); }); }
  else if (code == 'g') { const int b = hm_int(in); hm_guard(out, [&] { out << map.Find2(b); }); }
  else if (code == 'F') { int v = -1; const bool f = map.Find1(hm_int(in), v); out << f << ':' << v; }
  else if (code == 'G') { int v = -1; const bool f = map.Find2(hm_int(in), v); out << f << ':' << v; }
  else if (code == 'r') { map.ReSize(hm_size(in)); out << '-'; }
  else if (code == 'c') { map.Clear(hm_int(in) != 0); out << '-'; }
  else if (code == 'C') { NCollection_DoubleMap<int, int> aCopy(map); map.Assign(aCopy); out << '-'; }
}

static void hm_double(std::istream& in, std::ostream& out)
{
  NCollection_DoubleMap<int, int> map(hm_size(in));
  char code;
  while (in >> code) { hm_double_op(map, code, in, out); out << ' '; }
  out << "| n=" << map.NbBuckets() << " e=" << map.Extent() << " :";
  for (NCollection_DoubleMap<int, int>::Iterator it(map); it.More(); it.Next())
    out << ' ' << it.Key1() << ':' << it.Key2();
}

template <class T>
static void hm_hash_of(std::istream& in, std::ostream& out)
{
  long long aValue = 0;
  in >> aValue;
  const T aKey = static_cast<T>(aValue);
  NCollection_DefaultHasher<T> aHasher;
  out << aHasher(aKey) << ' ' << aHasher(aKey, aKey);
}

static void hm_hash(std::istream& in, std::ostream& out)
{
  std::string type;
  in >> type;
  if (type == "bool") hm_hash_of<bool>(in, out);
  else if (type == "char") hm_hash_of<char>(in, out);
  else if (type == "schar") hm_hash_of<signed char>(in, out);
  else if (type == "uchar") hm_hash_of<unsigned char>(in, out);
  else if (type == "wchar") hm_hash_of<wchar_t>(in, out);
  else if (type == "char16") hm_hash_of<char16_t>(in, out);
  else if (type == "char32") hm_hash_of<char32_t>(in, out);
  else if (type == "short") hm_hash_of<short>(in, out);
  else if (type == "ushort") hm_hash_of<unsigned short>(in, out);
  else if (type == "int") hm_hash_of<int>(in, out);
  else if (type == "uint") hm_hash_of<unsigned int>(in, out);
  else if (type == "long") hm_hash_of<long>(in, out);
  else if (type == "ulong") hm_hash_of<unsigned long>(in, out);
  else if (type == "llong") hm_hash_of<long long>(in, out);
  else if (type == "ullong") hm_hash_of<unsigned long long>(in, out);
  else if (type == "double")
  {
    double aValue = 0.0;
    in >> aValue;
    NCollection_DefaultHasher<double> aHasher;
    out << aHasher(aValue) << ' ' << aHasher(aValue, aValue);
  }
}

static bool ncollection_hash_maps_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_hash_maps_prime") out << NCollection_Primes::NextPrimeForMap(hm_size(in));
  else if (op == "ncoll_hash_maps_hash") hm_hash(in, out);
  else if (op == "ncoll_hash_maps_datamap") hm_datamap(in, out);
  else if (op == "ncoll_hash_maps_map") hm_map(in, out);
  else if (op == "ncoll_hash_maps_algo") hm_algo(in, out);
  else if (op == "ncoll_hash_maps_indexed") hm_indexed(in, out);
  else if (op == "ncoll_hash_maps_indexed_data") hm_indexed_data(in, out);
  else if (op == "ncoll_hash_maps_double") hm_double(in, out);
  else return false;
  return true;
}
