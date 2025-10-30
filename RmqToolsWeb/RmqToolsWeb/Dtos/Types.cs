using System.Text.Json.Serialization;

namespace RmqToolsWeb.Dtos;

public record RmqConnectionInfo(string Domain, string Vhost);

public record QueueSummary(string Name, bool Exclusive, int MessageCountInRmq, int? MessageCountInDb);

public record LoadMessagesByQueueNameResponse(List<Message> Messages);
public record Message(uint Id, string Payload);



[JsonSerializable(typeof(Message))]
[JsonSerializable(typeof(LoadMessagesByQueueNameResponse))]
[JsonSerializable(typeof(List<QueueSummary>))]
[JsonSerializable(typeof(RmqConnectionInfo))]
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
public partial class MySourceGenerationContext: JsonSerializerContext
{ }