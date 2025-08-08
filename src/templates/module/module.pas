unit {{mod}}module;

interface

uses
  System.SysUtils,
  Generics.Collections,
  nest4d.module,
  nest4d,
  {{mod}}service,
  {{mod}}repository,
  {{mod}}controller,
  {{mod}}infra;

type
  T{{mod}}Module = class(TModule)
  public
    constructor Create; override;
    function Binds: TBinds; override;
    function Imports: TImports; override;
  end;

implementation

{ T{{mod}}Module }

function T{{mod}}Module.Binds: TBinds;
begin
  Result := [Bind<T{{mod}}Infra>.Factory,
             Bind<T{{mod}}Repository>.Factory,
             Bind<T{{mod}}Service>.Factory,
             Bind<T{{mod}}Controller>.Singleton];
end;

constructor T{{mod}}Module.Create;
begin

  inherited;
end;

function T{{mod}}Module.Imports: TImports;
begin
  Result := [];
end;

end.
