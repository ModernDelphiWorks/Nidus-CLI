unit {{mod}}Repository;

interface

uses
  System.SysUtils,
  {{mod}}Infra,
  {{mod}}Interface;

type
  T{{mod}}Repository = class
  private
    FInfra: T{{mod}}Infra;
  public
    constructor Create(const AInfra: T{{mod}}Infra);
    destructor Destroy; override;
    function Find: String;
    function Insert(const AJson: String): String;
    function Update(const AJson: String): String;
    function Delete: String;
  end;

implementation

{ T{{mod}}Repository }

constructor T{{mod}}Repository.Create(const AInfra: T{{mod}}Infra);
begin
  FInfra := AInfra;
end;

destructor T{{mod}}Repository.Destroy;
begin
  FInfra.Free;
  inherited;
end;

function T{{mod}}Repository.Delete: String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function T{{mod}}Repository.Find: String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function T{{mod}}Repository.Insert(const AJson: String): String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

function T{{mod}}Repository.Update(const AJson: String): String;
begin
  try
    Result := 'sucesso!';
  except
    raise Exception.Create('falha!');
  end;
end;

end.
