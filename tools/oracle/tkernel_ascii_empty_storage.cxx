#include <TCollection_AsciiString.hxx>
#include <iostream>

void constructor_storage(const char* shared)
{
  const TCollection_AsciiString empty;
  const TCollection_AsciiString constructors[] = {
    TCollection_AsciiString(), TCollection_AsciiString(""),
    TCollection_AsciiString('\0'), TCollection_AsciiString(0, 'x'),
    TCollection_AsciiString(empty)};
  bool constructors_match = true;
  for (const auto& text : constructors)
    constructors_match &= text.ToCString() == shared;
  std::cout << "constructors:" << constructors_match << '\n';
}

void reset_storage(const char* shared)
{
  TCollection_AsciiString text("owned");
  text.Clear();
  const bool cleared = text.ToCString() == shared;
  text.Copy("owned");
  text.Copy("");
  const bool copied = text.ToCString() == shared;
  text.Copy("owned");
  const char* previous = text.ToCString();
  TCollection_AsciiString moved(std::move(text));
  std::cout << "reset:" << cleared << ':' << copied << ':'
            << (text.ToCString() == shared) << ':' << (moved.ToCString() == previous) << '\n';
}

void truncated_storage(const char* shared)
{
  TCollection_AsciiString text("owned");
  const char* previous = text.ToCString();
  text.Trunc(0);
  const TCollection_AsciiString clone(text);
  std::cout << "truncate:" << (text.ToCString() == previous) << ':'
            << (clone.ToCString() == shared) << ':' << (text == "") << '\n';
}

void mutated_storage(const char* shared)
{
  TCollection_AsciiString text;
  text.Capitalize();
  text.RightAdjust();
  text.Trunc(0);
  const bool unchanged = text.ToCString() == shared;
  text.Insert(1, 'a');
  const bool inserted = text == "a";
  text.Clear();
  text.LeftJustify(2, '.');
  const bool left = text == "..";
  text.Clear();
  text.RightJustify(2, '!');
  std::cout << "mutation:" << unchanged << ':' << inserted << ':' << left << ':' << (text == "!!") << '\n';
}

void removed_storage(const char* shared)
{
  TCollection_AsciiString text("aAa");
  const char* previous = text.ToCString();
  text.RemoveAll('a', false);
  TCollection_AsciiString untouched;
  untouched.RemoveAll('\0');
  std::cout << "remove:" << (text.ToCString() == previous) << ':'
            << (text.Length() == 0) << ':' << (untouched.ToCString() == shared) << '\n';
}

void removed_identical_storage()
{
  TCollection_AsciiString text("aaa");
  const char* previous = text.ToCString();
  text.RemoveAll('a');
  std::cout << "remove_sensitive:" << (text.ToCString() == previous) << ':'
            << (text.Length() == 0) << '\n';
}

int main()
{
  const char* shared = TCollection_AsciiString::EmptyString().ToCString();
  constructor_storage(shared);
  reset_storage(shared);
  truncated_storage(shared);
  mutated_storage(shared);
  removed_storage(shared);
  removed_identical_storage();
}
