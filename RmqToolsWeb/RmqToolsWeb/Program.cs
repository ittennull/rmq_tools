using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using MudBlazor;
using MudBlazor.Services;
using RmqToolsWeb;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

builder.Services.AddScoped<Api>();
builder.Services.AddMudServices(config =>
{
    config.SnackbarConfiguration.PositionClass = Defaults.Classes.Position.BottomCenter;
    config.SnackbarConfiguration.PreventDuplicates = false;
    config.SnackbarConfiguration.NewestOnTop = false;
    config.SnackbarConfiguration.ShowCloseIcon = true;
    config.SnackbarConfiguration.VisibleStateDuration = 2000;
    config.SnackbarConfiguration.HideTransitionDuration = 500;
    config.SnackbarConfiguration.ShowTransitionDuration = 500;
    config.SnackbarConfiguration.SnackbarVariant = Variant.Outlined;
});

var envUri = new Uri(builder.HostEnvironment.BaseAddress);
var apiUrl = builder.HostEnvironment.IsDevelopment() ? "http://localhost:3000" : builder.HostEnvironment.BaseAddress;
var wsUrl = builder.HostEnvironment.IsDevelopment() ? "ws://localhost:3000/api/ws" : $"ws://{envUri.Host}:{envUri.Port}/api/ws";
builder.Services.AddScoped(_ => new HttpClient { BaseAddress = new Uri(apiUrl) });
builder.Services.AddTransient(_ => new WebsocketApi(new Uri(wsUrl)));

await builder.Build().RunAsync();