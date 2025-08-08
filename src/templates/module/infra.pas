unit <mod>Infra;

interface

uses
  System.SysUtils,
  Generics.Collections,
  System.Evolution.Std,
  System.Evolution.Threading,
  <mod>Interface;

type
  T<mod>Infra = class
  private
  public
    constructor Create();
    destructor Destroy; override;
    // Json
    function FromJson<T: class, constructor>(const AJson: String): T;
    function ToJson<T: class, constructor>(const AObject: T): String;
  end;

implementation

{ T<mod>Infra }

constructor T<mod>Infra.Create();
begin

end;

destructor T<mod>Infra.Destroy;
begin

  inherited;
end;

function T<mod>Infra.FromJson<T>(const AJson: String): T;
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

function T<mod>Infra.ToJson<T>(const AObject: T): String;
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
