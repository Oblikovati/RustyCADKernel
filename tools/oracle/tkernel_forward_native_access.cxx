// OCCT delegates accessor/postfix directly, including when a host reports exhaustion.
#include <NCollection_ForwardRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>

struct Host {
  int position = 7;
  std::string* trace;
  bool failRead = false;
  bool failNext = false;
  bool More() const { *trace += "M"; return false; }
  int Value() const {
    *trace += "V";
    if (failRead) throw std::runtime_error("read");
    return position;
  }
  void Next() {
    *trace += "N";
    ++position;
    if (failNext) throw std::runtime_error("next");
  }
};
static void access() {
  std::string trace;
  Host host{7, &trace};
  NCollection_ForwardRangeIterator<Host> iter(&host);
  const int value = *iter;
  const auto old = iter++;
  std::cout << "access " << value << ' ' << *old << ' ' << host.position << ' ' << trace << '\n';
}
static void failures() {
  std::string trace;
  Host host{7, &trace, true};
  NCollection_ForwardRangeIterator<Host> iter(&host);
  try { iter++; } catch (const std::runtime_error&) {}
  std::cout << "read_failure " << host.position << ' ' << trace << '\n';
  trace.clear(); host.failRead = false; host.failNext = true;
  try { iter++; } catch (const std::runtime_error&) {}
  std::cout << "next_failure " << host.position << ' ' << trace << '\n';
}
static void proxies() {
  NCollection_ForwardRangeDetail::ArrowProxy<int> arrow{42};
  std::string trace;
  Host host{7, &trace};
  NCollection_ForwardRangeIterator<Host> iter(&host);
  auto postfix = iter++;
  postfix.myValue = 19;
  std::cout << "proxy " << (arrow.operator->() == &arrow.myValue) << ' '
            << *arrow.operator->() << ' ' << *postfix << '\n';
}
int main() { access(); failures(); proxies(); }
