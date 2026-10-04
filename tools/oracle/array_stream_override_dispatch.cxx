#include <Standard_ArrayStreamBuffer.hxx>

#include <iostream>

class DerivedStreamBuffer final : public Standard_ArrayStreamBuffer
{
public:
  explicit DerivedStreamBuffer(const char* theBytes)
      : Standard_ArrayStreamBuffer(theBytes, 3)
  {
  }

  void Init(const char* theBegin, size_t theSize) override
  {
    ++myInitCalls;
    Standard_ArrayStreamBuffer::Init(theBegin, theSize);
  }

  int_type underflow() override
  {
    ++myUnderflowCalls;
    if (myCurrent == myEnd)
    {
      return traits_type::eof();
    }
    ++myCurrent;
    return 300;
  }

  int_type uflow() override
  {
    ++myUflowCalls;
    return Standard_ArrayStreamBuffer::uflow();
  }

  int_type pbackfail(int_type theCharacter) override
  {
    ++myPbackfailCalls;
    const int_type aPutback = Standard_ArrayStreamBuffer::pbackfail(theCharacter);
    return aPutback == traits_type::eof() ? aPutback : 511;
  }

  std::streamsize showmanyc() override
  {
    ++myShowmanycCalls;
    myCurrent = myEnd;
    return Standard_ArrayStreamBuffer::showmanyc();
  }

  pos_type seekoff(off_type theOffset,
                   std::ios_base::seekdir theDirection,
                   std::ios_base::openmode theMode) override
  {
    ++mySeekoffCalls;
    return Standard_ArrayStreamBuffer::seekoff(theOffset, theDirection, theMode);
  }

  pos_type seekpos(pos_type thePosition, std::ios_base::openmode theMode) override
  {
    ++mySeekposCalls;
    return Standard_ArrayStreamBuffer::seekpos(thePosition, theMode);
  }

  std::streamsize xsgetn(char* theOutput, std::streamsize theCount) override
  {
    ++myXsgetnCalls;
    return Standard_ArrayStreamBuffer::xsgetn(theOutput, theCount);
  }

  void PrintCalls() const
  {
    std::cout << myInitCalls << ' ' << myUnderflowCalls << ' ' << myUflowCalls << ' '
              << myPbackfailCalls << ' ' << myShowmanycCalls << ' ' << mySeekoffCalls << ' '
              << mySeekposCalls << ' ' << myXsgetnCalls << '\n';
  }

private:
  int myInitCalls      = 0;
  int myUnderflowCalls = 0;
  int myUflowCalls     = 0;
  int myPbackfailCalls = 0;
  int myShowmanycCalls = 0;
  int mySeekoffCalls   = 0;
  int mySeekposCalls   = 0;
  int myXsgetnCalls    = 0;
};

int main()
{
  char aStorage[] = "ABC";
  DerivedStreamBuffer aDerived(aStorage);
  Standard_ArrayStreamBuffer* aBuffer = &aDerived;
  char aReplacement[] = "ABC";
  aBuffer->Init(aReplacement, 3);
  std::cout << aBuffer->sgetc() << ' ' << aBuffer->sbumpc() << ' ' << aBuffer->sputbackc('B')
            << ' ' << aBuffer->in_avail() << ' '
            << aBuffer->pubseekoff(2, std::ios_base::beg, std::ios_base::in) << ' '
            << aBuffer->pubseekpos(1, std::ios_base::in) << ' ';
  char aReadback[2] = {};
  std::cout << aBuffer->sgetn(aReadback, 2) << ' ' << aReadback[0] << aReadback[1] << '\n';
  aDerived.PrintCalls();
}
