program {{project}};

{$APPTYPE CONSOLE}

{$R *.res}

uses
  SysUtils,
  Horse,
  Nidus,
  Nidus.horse,
  AppModule in 'src\AppModule.pas';

begin
  THorse.Use( Nidus_Horse(TAppModule.Create) );

  THorse.Listen(9000, '127.0.0.1',
  procedure
  begin
    Writeln('');
    Writeln('Server running at http://127.0.0.1:9000');
    Writeln('');
  end);
end.
