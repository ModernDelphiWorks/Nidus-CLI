unit app.module;

interface

uses
  app.route,
  nest4d.module;

type
  TGardNFeMiddleware = class(TRouteMiddleware)
  public
    class function Call(const AReq: IRouteRequest): Boolean; override;
  end;

  TAppModule = class(TModule)
  public
    constructor Create; override;
    function Routes: TRoutes; override;
    function Binds: TBinds; override;
    function RouteHandlers: TRouteHandlers; override;
    function ExportedBinds: TExportedBinds; override;
  end;

implementation

{ TAppModule }

function TAppModule.Binds: TBinds;
begin
  // Inject Config - When registered as singleton, this class can be accessed globally throughout the application
  Result := [{Bind<TConfig>.Singleton}];
end;

constructor TAppModule.Create;
begin
  inherited;
end;

function TAppModule.ExportedBinds: TExportedBinds;
begin
  // Export CoreLib - This class/interface can be imported and used by other modules
  Result := [{Bind<TCoreLib>.SingletonInterface<ICoreLib>}];
end;

function TAppModule.RouteHandlers: TRouteHandlers;
begin
  // Inject Config - When registered as singleton, this class can be accessed globally throughout the application
  Result := [{TMainRouteHandler}];
end;

function TAppModule.Routes: TRoutes;
begin
  // Here we define the route and which module will be called
  // Format: RouteModule('base_path', ModuleClass)
  // This will handle all requests to /api/v1/main using TMainModule
  Result := [{RouteModule('/api/v1/main', TMainModule)}];
end;

{ TGardNFeMiddleware }

class function TGardNFeMiddleware.Call(const AReq: IRouteRequest): Boolean;
begin
  Result := True;
end;

end.
