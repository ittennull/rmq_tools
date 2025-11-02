using System.Text.Json;
using System.Text.Json.Serialization;

namespace RmqToolsWeb.Dtos;

public record RmqConnectionInfo(string Domain, string Vhost);

public record QueueSummary(uint? QueueId, string Name, bool Exclusive, int MessageCountInRmq, int? MessageCountInDb);

public record LoadMessagesByQueueNameResponse(uint QueueId, List<Message> Messages);
public record Message(uint Id, string Payload, Dictionary<string, JsonElement> Headers);

public record DeleteMessagesRequest(IEnumerable<uint> MessageIds);
public record SendMessagesRequest(string DestinationQueueName, IEnumerable<uint> MessageIds);


[JsonSerializable(typeof(Message))]
[JsonSerializable(typeof(LoadMessagesByQueueNameResponse))]
[JsonSerializable(typeof(List<QueueSummary>))]
[JsonSerializable(typeof(RmqConnectionInfo))]
[JsonSerializable(typeof(List<Message>))]
[JsonSerializable(typeof(DeleteMessagesRequest))]
[JsonSerializable(typeof(SendMessagesRequest))]
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
public partial class MySourceGenerationContext: JsonSerializerContext
{ }