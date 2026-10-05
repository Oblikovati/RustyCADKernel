#include <NCollection_FlatMap.hxx>
#include <NCollection_FlatDataMap.hxx>
#include <cctype>
#include <iostream>
#include <string>

struct FlatCopyElement
{
  int number;
  char role;
  std::string* trace;
  FlatCopyElement(int value, char kind, std::string& events)
      : number(value), role(kind), trace(&events) {}
  FlatCopyElement(const FlatCopyElement& source)
      : number(source.number), role(source.role), trace(source.trace)
  {
    trace->push_back(role);
  }
  FlatCopyElement& operator=(const FlatCopyElement&) = default;
  ~FlatCopyElement() { trace->push_back(static_cast<char>(std::tolower(role))); }
};

struct FlatCopyPolicy
{
  size_t salt;
  std::string* trace;
  FlatCopyPolicy(size_t value, std::string& events) : salt(value), trace(&events) {}
  FlatCopyPolicy(const FlatCopyPolicy& source) : salt(source.salt), trace(source.trace)
  {
    trace->push_back('C');
  }
  FlatCopyPolicy& operator=(const FlatCopyPolicy& source)
  {
    source.trace->push_back('A');
    salt = source.salt;
    trace = source.trace;
    return *this;
  }
  size_t operator()(const FlatCopyElement& key) const
  {
    trace->push_back('H');
    return size_t(key.number) + salt;
  }
  bool operator()(const FlatCopyElement& left, const FlatCopyElement& right) const
  {
    return left.number == right.number;
  }
};

template <typename Map> static void assignment_and_self(Map& target, Map& source, std::string& trace)
{
  trace.clear();
  const bool identity = &(target = source) == &target;
  std::cout << "assign " << trace << ' ' << identity << ' '
            << (target.Capacity() == source.Capacity()) << '\n';
  trace.clear();
  const bool selfIdentity = &(target = target) == &target;
  std::cout << "self " << trace.empty() << ' ' << selfIdentity << '\n';
  source.Clear(false);
  trace.clear();
  target = source;
  std::cout << "empty " << trace << ' ' << target.Capacity() << ' '
            << (source.Capacity() > 0) << ' ' << target.GetHasher().salt << '\n';
  trace.clear();
  Map emptyCopy(source);
  std::cout << "empty-copy " << trace << ' ' << emptyCopy.Capacity() << '\n';
}

static void set_copy_contracts()
{
  std::string trace;
  using Map = NCollection_FlatMap<FlatCopyElement, FlatCopyPolicy>;
  Map source(FlatCopyPolicy(7, trace), 40), target(FlatCopyPolicy(2, trace), 40);
  source.Add(FlatCopyElement(4, 'K', trace));
  target.Add(FlatCopyElement(9, 'K', trace));
  trace.clear();
  Map copied(source);
  std::cout << "copy " << trace << ' ' << (copied.Capacity() == source.Capacity()) << '\n';
  assignment_and_self(target, source, trace);
}

static void dictionary_copy_contracts()
{
  std::string trace;
  using Map = NCollection_FlatDataMap<FlatCopyElement, FlatCopyElement, FlatCopyPolicy>;
  Map source(FlatCopyPolicy(7, trace), 40), target(FlatCopyPolicy(2, trace), 40);
  source.Bind(FlatCopyElement(4, 'K', trace), FlatCopyElement(40, 'V', trace));
  target.Bind(FlatCopyElement(9, 'K', trace), FlatCopyElement(90, 'V', trace));
  trace.clear();
  Map copied(source);
  std::cout << "copy " << trace << ' ' << (copied.Capacity() == source.Capacity()) << '\n';
  assignment_and_self(target, source, trace);
}

int main()
{
  set_copy_contracts();
  dictionary_copy_contracts();
}
