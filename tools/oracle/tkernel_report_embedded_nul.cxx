// Message_Report::dumpMessages streams GetMessageKey() as a C string at OCCT 3d097a0.
#include <Message_AlertExtended.hxx>
#include <Message_Attribute.hxx>
#include <Message_CompositeAlerts.hxx>
#include <Message_Report.hxx>
#include <iostream>

occ::handle<Message_AlertExtended> namedAlert(const char* bytes, int length)
{
  TCollection_AsciiString name(length, '\0');
  for (int index = 0; index < length; ++index)
    if (bytes[index] != '\0') name.SetValue(index + 1, bytes[index]);
  occ::handle<Message_AlertExtended> alert = new Message_AlertExtended();
  alert->SetAttribute(new Message_Attribute(name));
  return alert;
}

int main()
{
  Message_Report report;
  auto parent = namedAlert("parent\0hidden", 13);
  auto child = namedAlert("child\0hidden", 12);
  parent->CompositeAlerts(true)->AddAlert(Message_Info, child);
  report.AddAlert(Message_Info, parent);
  report.AddAlert(Message_Info, namedAlert("\0hidden", 7));
  report.AddAlert(Message_Warning, namedAlert("warning\0hidden", 14));
  report.Dump(std::cout);
}
