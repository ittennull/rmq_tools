using System.Text.Json;
using System.Text.Json.Serialization;

namespace RmqToolsWeb;

public record RmqConnectionInfo(string Domain, string? ServerName, string Vhost);
public record EnvInfo(RmqConnectionInfo RmqConnectionInfo, int ImportanceLevel);
public record QueueSummary(uint? QueueId, string Name, bool Exclusive, int MessageCountInRmq, int MessageCountInDb);
public record LoadMessagesByQueueNameResponse(uint QueueId, List<Message> Messages);
public record Message(uint Id, string Payload, Dictionary<string, JsonElement> Headers);
public record DeleteMessagesRequest(IEnumerable<uint> MessageIds);
public record SendMessagesRequest(string DestinationQueueName, IEnumerable<uint> MessageIds);
public record QueueCounters(string QueueName, int Messages);


[JsonSerializable(typeof(List<QueueSummary>))]
[JsonSerializable(typeof(List<Message>))]
[JsonSerializable(typeof(List<QueueCounters>))]
[JsonSerializable(typeof(EnvInfo))]
[JsonSerializable(typeof(LoadMessagesByQueueNameResponse))]
[JsonSerializable(typeof(DeleteMessagesRequest))]
[JsonSerializable(typeof(SendMessagesRequest))]
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
public partial class MySourceGenerationContext: JsonSerializerContext;