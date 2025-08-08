program <project>;

{$APPTYPE CONSOLE}

{$R *.res}

uses
  SysUtils,
  Horse,
  nest4d,
  nest4d.horse,
  AppModule in 'src\AppModule.pas';

begin
  THorse.Use( Nest4D_Horse(TAppModule.Create) );

  THorse.Listen(9000, '127.0.0.1',
  procedure
  begin
    Writeln('');
    Writeln('Server running at http://127.0.0.1:9000');
    Writeln('');
  end);
end.
