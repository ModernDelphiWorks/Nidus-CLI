program testproject;

{$APPTYPE CONSOLE}

{$R *.res}

uses
  SysUtils,
  Horse,
  nest4d,
  nest4d.horse,
  AppModule in 'src\AppModule.pas',
  UserModule in 'src\modules\user\UserModule.pas',
  UserHandler in 'src\modules\user\UserHandler.pas';

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
