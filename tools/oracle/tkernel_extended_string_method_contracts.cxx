#include <TCollection_ExtendedString.hxx>
#include <iostream>
#include <string_view>
using Text = TCollection_ExtendedString;
static void shared_empty() {
  Text first, second;
  std::cout << "shared " << (first.ToExtString() == second.ToExtString()) << ' ' << (first.ToExtString() == Text::EmptyString().ToExtString()) << '\n';
}
static void empty_owned() {
  Text text(u"abc"); auto pointer = text.ToExtString(); text.Trunc(0); Text copied(text);
  std::cout << "empty_owned " << (pointer == text.ToExtString()) << ' ' << (copied.ToExtString() == Text::EmptyString().ToExtString());
  text.Clear(); std::cout << ' ' << (text.ToExtString() == Text::EmptyString().ToExtString()) << '\n';
}
static void empty_copy() {
  Text text(u"abc"); auto pointer = text.ToExtString(); text.Copy(nullptr, 20);
  bool null = pointer == text.ToExtString(); text.Copy(u"a", -1);
  std::cout << "empty_copy " << null << ' ' << (pointer == text.ToExtString()) << ' ' << text.Length() << '\n';
}
static void move_empty() {
  Text source(u"abc"); auto pointer = source.ToExtString(); source.Trunc(0); Text moved(std::move(source));
  Text destination(u"old"); bool returned = &(destination = std::move(moved)) == &destination;
  std::cout << "move_owned " << returned << ' ' << (destination.ToExtString() == pointer) << ' ' << (source.ToExtString() == Text::EmptyString().ToExtString()) << ' ' << (moved.ToExtString() == source.ToExtString()) << '\n';
}
static void move_static() {
  Text source, destination(u"old"); auto pointer = destination.ToExtString(); destination.Move(std::move(source));
  std::cout << "move_static " << (destination.ToExtString() == pointer) << ' ' << destination.Length() << ' ' << (source.ToExtString() == Text::EmptyString().ToExtString()) << '\n';
}
static void remove_all() {
  const char16_t units[] = {65, 0, 66, 65}; Text text(units, 4); auto pointer = text.ToExtString();
  text.RemoveAll(65);
  std::cout << "remove_all " << (text.ToExtString() == pointer) << ' ' << text.Length() << ' ' << unsigned(text.Value(1)) << ' ' << unsigned(text.Value(2)) << '\n';
}
static void view() {
  const char16_t units[] = {65, 0, 66}; Text text;
  bool returned = &(text = std::u16string_view(units, 3)) == &text;
  auto pointer = text.ToExtString(); text = std::u16string_view();
  std::cout << "view " << returned << ' ' << (pointer == text.ToExtString()) << ' ' << text.Length() << '\n';
}
int main() { shared_empty(); empty_owned(); empty_copy(); move_empty(); move_static(); remove_all(); view(); }
