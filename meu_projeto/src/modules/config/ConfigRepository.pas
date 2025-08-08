unit ConfigRepository;

interface

uses
  SysUtils,
  ConfigInfra,
  ConfigInterface;

type
  TConfigRepository = class
  private
    FInfra: TConfigInfra;
  public
    constructor Create(const AInfra: TConfigInfra);
    destructor Destroy; override;
    function Find: String;
    function Insert(const AJson: String): String;
    function Update(const AJson: String): String;
    function Delete: String;
  end;

implementation

{ TConfigRepository }

constructor TConfigRepository.Create(const AInfra: TConfigInfra);
begin
  FInfra := AInfra;
end;

destructor TConfigRepository.Destroy;
begin
  FInfra.Free;
  inherited;
end;

function TConfigRepository.Delete: String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function TConfigRepository.Find: String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function TConfigRepository.Insert(const AJson: String): String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function TConfigRepository.Update(const AJson: String): String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

end.
