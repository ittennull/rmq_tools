using System.Text.Json.Serialization;

namespace RmqToolsWeb.Dtos;

public record RmqConnectionInfo(string Domain, string Vhost);

public record QueueSummary(uint? QueueId, string Name, bool Exclusive, int MessageCountInRmq, int? MessageCountInDb);

public record LoadMessagesByQueueNameResponse(List<Message> Messages);
public record Message(uint Id, string Payload);

public record DeleteMessagesRequest(IEnumerable<uint> MessageIds);


[JsonSerializable(typeof(Message))]
[JsonSerializable(typeof(LoadMessagesByQueueNameResponse))]
[JsonSerializable(typeof(List<QueueSummary>))]
[JsonSerializable(typeof(RmqConnectionInfo))]
[JsonSerializable(typeof(List<Message>))]
[JsonSerializable(typeof(DeleteMessagesRequest))]
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
public partial class MySourceGenerationContext: JsonSerializerContext
{ }