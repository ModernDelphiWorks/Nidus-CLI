unit {{mod}}Controller;

interface

uses
  System.SysUtils,
  System.Evolution.ResultPair,
  {{mod}}Service,
  {{mod}}Interface;

type
  T{{mod}}Controller = class(TInterfacedObject, I{{mod}})
  private
    FService: T{{mod}}Service;
  public
    constructor Create(const AReq: T{{mod}}Service);
    destructor Destroy; override;
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

uses
  nest4d.horse;

{ T{{mod}}Controller }

constructor T{{mod}}Controller.Create(const AReq: T{{mod}}Service);
begin
  FService := AReq;
end;

destructor T{{mod}}Controller.Destroy;
begin
  FService.Free;
  inherited;
end;

function T{{mod}}Controller.Delete: TResultPair<String, Exception>;
begin
  Result := FService.Delete;
end;

function T{{mod}}Controller.Find: TResultPair<String, Exception>;
begin
  Result := FService.Find;
end;

function T{{mod}}Controller.Insert(const AJson: String): TResultPair<String, Exception>;
begin
  Result := FService.Insert(AJson);
end;

function T{{mod}}Controller.Update(const AJson: String): TResultPair<String, Exception>;
begin
  Result := FService.Update(AJson);
end;

end.
