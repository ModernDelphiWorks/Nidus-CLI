unit {{mod}}Interface;

interface

uses
  System.SysUtils,
  ModernSyntax.ResultPair;


type
  I{{mod}} = interface
    ['{????????-????-????-????-????????????}']
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

end.