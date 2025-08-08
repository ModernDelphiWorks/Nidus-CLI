unit Usermodule;

interface

uses
  System.SysUtils,
  Generics.Collections,
  nest4d.module,
  nest4d,
  Userservice,
  Userrepository,
  Usercontroller,
  Userinfra;

type
  TUserModule = class(TModule)
  public
    constructor Create; override;
    function Binds: TBinds; override;
    function Imports: TImports; override;
  end;

implementation

{ TUserModule }

function TUserModule.Binds: TBinds;
begin
  Result := [Bind<TUserInfra>.Factory,
             Bind<TUserRepository>.Factory,
             Bind<TUserService>.Factory,
             Bind<TUserController>.Singleton];
end;

constructor TUserModule.Create;
begin

  inherited;
end;

function TUserModule.Imports: TImports;
begin
  Result := [];
end;

end.
