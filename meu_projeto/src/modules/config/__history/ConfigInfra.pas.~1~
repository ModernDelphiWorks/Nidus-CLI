unit ConfigInfra;

interface

uses
  SysUtils,
  Generics.Collections,
  System.Evolution.Std,
  System.Evolution.Threading,
  ConfigInterfaces;

type
  TConfigInfra = class
  private
  public
    constructor Create();
    destructor Destroy; override;
    // Json
    function FromJson<T: class, constructor>(const AJson: String): T;
    function ToJson<T: class, constructor>(const AObject: T): String;
  end;

implementation

{ TConfigInfra }

constructor TConfigInfra.Create();
begin

end;

destructor TConfigInfra.Destroy;
begin

  inherited;
end;

function TConfigInfra.FromJson<T>(const AJson: String): T;
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

function TConfigInfra.ToJson<T>(const AObject: T): String;
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
