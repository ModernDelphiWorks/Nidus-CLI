unit Configmodule;

interface

uses
  Generics.Collections,
  nest4d.module,
  nest4d,
  Configservice,
  Configrepository,
  Configcontroller,
  Configinfra;

type
  TConfigModule = class(TModule)
  public
    constructor Create; override;
    function Binds: TBinds; override;
    function Imports: TImports; override;
  end;

implementation

{ TConfigModule }

function TConfigModule.Binds: TBinds;
begin
  Result := [Bind<TConfigInfra>.Factory,
             Bind<TConfigRepository>.Factory,
             Bind<TConfigService>.Factory,
             Bind<TConfigController>.Singleton];
end;

constructor TConfigModule.Create;
begin

  inherited;
end;

function TConfigModule.Imports: TImports;
begin
  Result := [];
end;

end.
