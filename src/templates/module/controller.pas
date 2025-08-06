unit $mod$Controller;

interface

uses
  $mod$Service;

type
  T$mod$Controller = class
  private
    F$mod$Service: T$mod$Service;
  public
    constructor Create;
    destructor Destroy; override;
    function Register$mod$(const AName, AEmail: String): Boolean;
  end;

implementation

{{ T$mod$Controller }}

constructor T$mod$Controller.Create;
begin
  F$mod$Service := T$mod$Service.Create;
end;

destructor T$mod$Controller.Destroy;
begin
  F$mod$Service.Free;
  inherited;
end;

function T$mod$Controller.Register$mod$(const AName, AEmail: String): Boolean;
begin
  Result := F$mod$Service.Create$mod$(AName, AEmail);
end;

end.
