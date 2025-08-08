unit {{mod}}Handler;

interface

uses
  System.SysUtils,
  Horse,
  System.Evolution.ResultPair,
  nest4d,
  nest4d.route.handler.horse;

type
  T{{mod}}RouteHandler = class(TRouteHandlerHorse)
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
  {{mod}}Controller;

{ T{{mod}}RouteHandler }

procedure T{{mod}}RouteHandler.RegisterRoutes;
begin
  inherited;
  RouteGet('/{{mod}}', Find);
  RoutePost('/{{mod}}', Insert);
  RoutePut('/{{mod}}', Update);
  RouteDelete('/{{mod}}', Delete);
end;

constructor T{{mod}}RouteHandler.Create;
begin
  inherited;
end;

procedure T{{mod}}RouteHandler.Delete(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<T{{mod}}Controller>.Delete;
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

procedure T{{mod}}RouteHandler.Find(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<T{{mod}}Controller>.Find;
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

procedure T{{mod}}RouteHandler.Insert(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<T{{mod}}Controller>.Insert(Req.Body);
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

procedure T{{mod}}RouteHandler.Update(Req: THorseRequest; Res: THorseResponse);
var
  LResult: TResultPair<String, Exception>;
begin
  LResult := GetNest4D.Get<T{{mod}}Controller>.Update(Req.Body);
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