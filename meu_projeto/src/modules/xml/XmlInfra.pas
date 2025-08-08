unit XmlInfra;

interface

uses
  SysUtils,
  Generics.Collections,
  System.Evolution.Std,
  System.Evolution.Threading,
  XmlInterface;

type
  TXmlInfra = class
  private
  public
    constructor Create();
    destructor Destroy; override;
    // Json
    function FromJson<T: class, constructor>(const AJson: String): T;
    function ToJson<T: class, constructor>(const AObject: T): String;
  end;

implementation

{ TXmlInfra }

constructor TXmlInfra.Create();
begin

end;

destructor TXmlInfra.Destroy;
begin

  inherited;
end;

function TXmlInfra.FromJson<T>(const AJson: String): T;
var
  LFuture: TFuture;
begin
  LFuture := Async(function: TValue
                   begin
                     Result := TValue.Empty;
                   end).Await();
  if LFuture.IsOk then
    Result := LFuture.Ok<T>
  else
    raise Exception.Create(LFuture.Err);
end;

function TXmlInfra.ToJson<T>(const AObject: T): String;
var
  LFuture: TFuture;
begin
  LFuture := Async(function: TValue
                   begin
                     Result := TValue.Empty;
                   end).Await();
  if LFuture.IsOk then
    Result := LFuture.Ok<String>
  else
    raise Exception.Create(LFuture.Err);
end;

end.
