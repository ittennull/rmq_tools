using System.Text.Json;

namespace RmqToolsWeb;

public static class ApiSerialization
{
    public static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
    };
}