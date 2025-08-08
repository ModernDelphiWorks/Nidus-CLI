unit ConfigService;

interface

uses
  SysUtils,
  System.Evolution.ResultPair,
  ConfigRepository,
  ConfigInterface;

type
  TConfigService = class(TInterfacedObject, IConfig)
  private
    FRepository: TConfigRepository;
  public
    constructor Create(const ARepository: TConfigRepository);
    destructor Destroy; override;
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

{ TConfigService }

constructor TConfigService.Create(const ARepository: TConfigRepository);
begin
  FRepository := ARepository;
end;

destructor TConfigService.Destroy;
begin
  FRepository.Free;
  inherited;
end;

function TConfigService.Delete: TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Delete);
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function TConfigService.Find: TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Find);
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function TConfigService.Insert(const AJson: String): TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Insert(AJson));
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function TConfigService.Update(const AJson: String): TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Update(AJson));
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

end.
