unit $mod$Service;

interface

uses
  $mod$, $mod$Repository;

type
  T$mod$Service = class
  private
    F$mod$Repository: T$mod$Repository;
  public
    constructor Create;
    destructor Destroy; override;
    function Create$mod$(const AName, AEmail: String): Boolean;
  end;

implementation

{ T$mod$Service }

constructor T$mod$Service.Create;
begin
  F$mod$Repository := T$mod$Repository.Create;
end;

destructor T$mod$Service.Destroy;
begin
  F$mod$Repository.free;
  inherited;
end;

function T$mod$Service.Create$mod$(const AName, AEmail: String): Boolean;
var
  L$mod$: T$mod$;
begin
  L$mod$ := T$mod$.Create;
  try
  L$mod$.Name := AName;
  L$mod$.Email := AEmail;
  Result := F$mod$Repository.Save($mod$);
  finally
    L$mod$.Free;
  end;
end;

end.
