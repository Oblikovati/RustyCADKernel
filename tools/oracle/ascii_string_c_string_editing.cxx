#include <Standard_Failure.hxx>
#include <Standard_OutOfRange.hxx>
#include <TCollection_AsciiString.hxx>

#include <iomanip>
#include <iostream>

static void show(const TCollection_AsciiString& text)
{
  std::cout << text.Length();
  for (int index = 0; index < text.Length(); ++index)
  {
    std::cout << ' ' << std::hex << std::setw(2) << std::setfill('0')
              << static_cast<unsigned>(static_cast<unsigned char>(text.ToCString()[index])) << std::dec;
  }
  std::cout << '\n';
}

template <typename Operation>
static void show_failure(Operation&& operation)
{
  try
  {
    operation();
    std::cout << "OK\n";
  }
  catch (const Standard_OutOfRange&)
  {
    std::cout << "Standard_OutOfRange\n";
  }
  catch (const Standard_Failure&)
  {
    std::cout << "Standard_Failure\n";
  }
}

static void copy_cases()
{
  const char source[] = "a\0b";
  TCollection_AsciiString text("old");
  text.Copy(source);
  show(text);
  text.Copy(source, 3);
  show(text);
  text = source;
  show(text);
  const char* const sourcePointer = source;
  text.Copy(sourcePointer);
  show(text);
  text = sourcePointer;
  show(text);
  text.Copy(static_cast<const char*>(nullptr));
  show(text);
}

static void insert_cases()
{
  const char source[] = "b\0c";
  const char* const sourcePointer = source;
  TCollection_AsciiString direct("AZ");
  direct.Insert(2, source);
  show(direct);
  TCollection_AsciiString pointer("AZ");
  pointer.Insert(2, sourcePointer);
  show(pointer);
  TCollection_AsciiString span("AZ");
  span.Insert(2, source, 3);
  show(span);
  TCollection_AsciiString after("AZ");
  after.InsertAfter(1, source);
  show(after);
  TCollection_AsciiString afterPointer("AZ");
  afterPointer.InsertAfter(1, sourcePointer);
  show(afterPointer);
  TCollection_AsciiString afterSpan("AZ");
  afterSpan.InsertAfter(1, source, 3);
  show(afterSpan);
  TCollection_AsciiString before("AZ");
  before.InsertBefore(2, source);
  show(before);
  TCollection_AsciiString beforePointer("AZ");
  beforePointer.InsertBefore(2, sourcePointer);
  show(beforePointer);
  TCollection_AsciiString beforeSpan("AZ");
  beforeSpan.InsertBefore(2, source, 3);
  show(beforeSpan);
}

static void null_and_empty_cases()
{
  TCollection_AsciiString text("AZ");
  text.Insert(-1, static_cast<const char*>(nullptr));
  text.InsertAfter(-1, static_cast<const char*>(nullptr));
  text.InsertBefore(0, static_cast<const char*>(nullptr));
  show(text);
  show_failure([&text]() { text.Insert(-1, ""); });
  show_failure([&text]() { text.InsertAfter(-1, ""); });
  show_failure([&text]() { text.InsertBefore(0, ""); });
}

int main()
{
  copy_cases();
  insert_cases();
  null_and_empty_cases();
}
