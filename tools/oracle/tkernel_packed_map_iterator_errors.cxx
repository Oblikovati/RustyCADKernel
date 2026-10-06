// Message and constructor-choice oracle for the unchanged pinned legacy iterator.
#include <NCollection_PackedMap.hxx>
#include <Standard_Failure.hxx>
#include <cstdint>
#include <iostream>
#include <string>
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> using Cursor = typename Map<T>::Iterator;
template<class T> using Word = typename Map<T>::IndexType;
void failure_fields(const Standard_Failure& failure) {
  const std::string trace = failure.GetStackString();
  const bool shape = trace.rfind("\n==Backtrace==", 0) == 0 && trace.size() >= 14
                     && trace.substr(trace.size() - 14) == "\n=============";
  std::cout << "0|0|0|" << failure.ExceptionType() << '|' << failure.what()
            << '|' << !trace.empty() << '|' << shape;
}
template<class T> void access(const char* name, int frames, const char* state,
                              const Cursor<T>& cursor, bool value) {
  const auto cached = std::uint64_t(Word<T>(*cursor));
  std::cout << name << ':' << frames << ':' << state << ':' << (value ? "value" : "key")
            << ' ' << cursor.More() << '|' << cached << '|';
  try {
    const T& result = value ? cursor.Value() : cursor.Key();
    std::cout << "1|" << std::uint64_t(Word<T>(result)) << '|' << (&result == &*cursor) << "|||0|0";
  } catch(const Standard_Failure& failure) { failure_fields(failure); }
  std::cout << '|' << Standard_Failure::DefaultStackTraceLength()
            << '|' << (cached == std::uint64_t(Word<T>(*cursor))) << '\n';
}
template<class T> void snapshot(const char* name, int frames, const char* state, const Cursor<T>& cursor) {
  access<T>(name, frames, state, cursor, false);
  access<T>(name, frames, state, cursor, true);
}
template<class T> void width(const char* name, int frames) {
  Map<T> empty, populated; populated.Add(T(41)); populated.Add(T(42));
  Cursor<T> initial, cursor(empty);
  snapshot<T>(name, frames, "default", initial); snapshot<T>(name, frames, "bound_empty", cursor);
  cursor.Initialize(populated); snapshot<T>(name, frames, "active", cursor);
  while (cursor.More()) cursor.Next();
  snapshot<T>(name, frames, "exhausted", cursor);
  cursor.Reset(); snapshot<T>(name, frames, "reset_active", cursor);
  cursor.Initialize(empty); snapshot<T>(name, frames, "initialized_empty", cursor);
  cursor.Reset(); snapshot<T>(name, frames, "reset_empty", cursor);
  cursor.Initialize(populated); snapshot<T>(name, frames, "initialized_active", cursor);
}
int main() {
  const int original = Standard_Failure::DefaultStackTraceLength();
  for (int frames : {-3, 0, 3}) {
    Standard_Failure::SetDefaultStackTraceLength(frames);
    width<std::int32_t>("i32", frames); width<std::uint32_t>("u32", frames);
    width<std::int64_t>("i64", frames); width<std::uint64_t>("u64", frames);
    width<std::int8_t>("i8", frames); width<bool>("bool", frames);
  }
  Standard_Failure::SetDefaultStackTraceLength(original);
}
