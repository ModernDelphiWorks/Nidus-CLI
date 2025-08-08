unit XmlInterface;

interface

uses
  System.Evolution.ResultPair,
  SysUtils;

type
  IXml = interface
    ['{516A26E0-8C08-4CFD-A312-1EBB57CB374A}']
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

end.