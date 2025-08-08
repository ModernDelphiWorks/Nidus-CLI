unit NfeInterface;

interface

uses
  System.Evolution.ResultPair,
  SysUtils;

type
  INfe = interface
    ['{C8B31CA6-5590-4C99-8504-619AD005D1DD}']
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

end.