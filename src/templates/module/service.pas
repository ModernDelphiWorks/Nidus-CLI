unit {{mod}}Service;

interface

uses
  System.SysUtils,
  System.Evolution.ResultPair,
  {{mod}}Repository,
  {{mod}}Interface;

type
  T{{mod}}Service = class(TInterfacedObject, I{{mod}})
  private
    FRepository: T{{mod}}Repository;
  public
    constructor Create(const ARepository: T{{mod}}Repository);
    destructor Destroy; override;
    function Find: TResultPair<String, Exception>;
    function Insert(const AJson: String): TResultPair<String, Exception>;
    function Update(const AJson: String): TResultPair<String, Exception>;
    function Delete: TResultPair<String, Exception>;
  end;

implementation

{ T{{mod}}Service }

constructor T{{mod}}Service.Create(const ARepository: T{{mod}}Repository);
begin
  FRepository := ARepository;
end;

destructor T{{mod}}Service.Destroy;
begin
  FRepository.Free;
  inherited;
end;

function T{{mod}}Service.Delete: TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Delete);
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function T{{mod}}Service.Find: TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Find);
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function T{{mod}}Service.Insert(const AJson: String): TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Insert(AJson));
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

function T{{mod}}Service.Update(const AJson: String): TResultPair<String, Exception>;
begin
  try
    Result.Success(FRepository.Update(AJson));
  except
    on E: Exception do
      Result.Failure(E);
  end;
end;

end.
