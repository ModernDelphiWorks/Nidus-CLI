unit Nfemodule;

interface

uses
  Generics.Collections,
  nest4d.module,
  nest4d,
  Nfeservice,
  Nferepository,
  Nfecontroller,
  Nfeinfra;

type
  TNfeModule = class(TModule)
  public
    constructor Create; override;
    function Binds: TBinds; override;
    function Imports: TImports; override;
  end;

implementation

{ TNfeModule }

function TNfeModule.Binds: TBinds;
begin
  Result := [Bind<TNfeInfra>.Factory,
             Bind<TNfeRepository>.Factory,
             Bind<TNfeService>.Factory,
             Bind<TNfeController>.Singleton];
end;

constructor TNfeModule.Create;
begin

  inherited;
end;

function TNfeModule.Imports: TImports;
begin
  Result := [];
end;

end.
