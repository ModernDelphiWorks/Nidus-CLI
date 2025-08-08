unit ConfigController;

interface

uses
  System.Evolution.ResultPair,
  ConfigService,
  ConfigInterfaces;

type
  TConfigController = class(TInterfacedObject, IConfig)
  private
    FService: TConfigService;
  public
    constructor Create(const AReq: TConfigService);
    destructor Destroy; override;
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

uses
  nest4d.horse;

{ TConfigController }

constructor TConfigController.Create(const AReq: TConfigService);
begin
  FService := AReq;
end;

destructor TConfigController.Destroy;
begin
  FService.Free;
  inherited;
end;

function TConfigController.Delete: TResultPair<String, Exception>;
begin
  Result := FService.Delete;
end;

function TConfigController.Find: TResultPair<String, Exception>;
begin
  Result := FService.Find;
end;

function TConfigController.Insert(const AJson: String): TResultPair<String, Exception>;
begin
  Result := FService.Insert(AJson);
end;

function TConfigController.Update(const AJson: String): TResultPair<String, Exception>;
begin
  Result := FService.Update(AJson);
end;

end.
