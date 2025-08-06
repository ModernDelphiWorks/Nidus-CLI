unit config.route.handler;

interface

uses
  IniFiles,
  core.include,
  core.types,
  core.exception,
  core.constantes,
  dmfbr.modular,
  dmfbr.route.handler.horse;

type
  TConfigRouteHandler = class(TRouteHandlerHorse)
  protected
    procedure RegisterRoutes; override;
  public
    constructor Create; override;
    procedure Find(Req: THorseRequest; Res: THorseResponse);
    procedure Insert(Req: THorseRequest; Res: THorseResponse);
    procedure Update(Req: THorseRequest; Res: THorseResponse);
    procedure Delete(Req: THorseRequest; Res: THorseResponse);
  end;

implementation

uses
  app.route,
  app.module,
  horse.modular,
  config.controller;

{ TEmpresaRouteHandler }

procedure TConfigRouteHandler.RegisterRoutes;
begin
  inherited;
  RouteGet(Rota.Configurar, Find);
  RoutePost(Rota.Configurar, Insert);
  RoutePut(Rota.Configurar, Update);
  RouteDelete(Rota.Configurar, Delete);
end;

constructor TConfigRouteHandler.Create;
begin
  inherited;

end;

procedure TConfigRouteHandler.Delete(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TConfigResponse;
begin
  LResult := Modular.Get<TConfigController>.Delete;
  LResult.TryException(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: ERequestException)
    begin
      try
        raise ERequestException.Create(Error.Message, Error.Status);
      finally
        Error.Free;
      end;
    end);
end;

procedure TConfigRouteHandler.Find(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TConfigResponse;
begin
  LResult := Modular.Get<TConfigController>.Find;
  LResult.TryException(
    procedure (Json: String)
    begin
      Res.Send(Json).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: ERequestException)
    begin
      try
        raise ERequestException.Create(Error.Message, Error.Status);
      finally
        Error.Free;
      end;
    end);
end;

procedure TConfigRouteHandler.Insert(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TConfigResponse;
begin
  LResult := Modular.Get<TConfigController>.Insert(Req.Body);
  LResult.TryException(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: ERequestException)
    begin
      try
        raise ERequestException.Create(Error.Message, Error.Status);
      finally
        Error.Free;
      end;
    end);
end;

procedure TConfigRouteHandler.Update(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TConfigResponse;
begin
  LResult := Modular.Get<TConfigController>.Update(Req.Body);
  LResult.TryException(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: ERequestException)
    begin
      try
        raise ERequestException.Create(Error.Message, Error.Status);
      finally
        Error.Free;
      end;
    end);
end;

end.



