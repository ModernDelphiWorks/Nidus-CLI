unit ConfigInterfaces;

interface

uses
  System.Evolution.ResultPair,
  SysUtils;

type
  IConfig = interface
    ['{????????-????-????-????-????????????}']
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

end.