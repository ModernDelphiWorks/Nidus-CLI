unit ConfigInterface;

interface

uses
  System.Evolution.ResultPair,
  SysUtils;

type
  IConfig = interface
    ['{880B91CF-49A4-4669-A741-A100C5280869}']
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

end.