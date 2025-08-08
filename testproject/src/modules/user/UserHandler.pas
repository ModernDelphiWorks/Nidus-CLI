unit UserHandler;

interface

uses
  System.SysUtils,
  Horse,
  System.Evolution.ResultPair,
  nest4d,
  nest4d.route.handler.horse;

type
  TUserRouteHandler = class(TRouteHandlerHorse)
  private
    const CONTENTTYPE_JSON = 'application/json; charset=UTF-8';
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
  nest4d.horse,
  UserController;

{ TUserRouteHandler }

procedure TUserRouteHandler.RegisterRoutes;
begin
  inherited;
  RouteGet('/User', Find);
  RoutePost('/User', Insert);
  RoutePut('/User', Update);
  RouteDelete('/User', Delete);
end;

constructor TUserRouteHandler.Create;
begin
  inherited;
end;

procedure TUserRouteHandler.Delete(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<TUserController>.Delete;
  LResult.When(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: Exception)
    begin
      try
        raise Exception.Create(Error.Message);
      finally
        Error.Free;
      end;
    end);
end;

procedure TUserRouteHandler.Find(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<TUserController>.Find;
  LResult.When(
    procedure (Json: String)
    begin
      Res.Send(Json).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: Exception)
    begin
      try
        raise Exception.Create(Error.Message);
      finally
        Error.Free;
      end;
    end);
end;

procedure TUserRouteHandler.Insert(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<TUserController>.Insert(Req.Body);
  LResult.When(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: Exception)
    begin
      try
        raise Exception.Create(Error.Message);
      finally
        Error.Free;
      end;
    end);
end;

procedure TUserRouteHandler.Update(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<TUserController>.Update(Req.Body);
  LResult.When(
    procedure (Msg: String)
    begin
      Res.Send(Msg).ContentType(CONTENTTYPE_JSON).Status(200);
    end,
    procedure (Error: Exception)
    begin
      try
        raise Exception.Create(Error.Message);
      finally
        Error.Free;
      end;
    end);
end;

end.