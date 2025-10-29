using System.Text.Json.Serialization;

namespace RmqToolsWeb.Dtos;

public record RmqConnectionInfo(string Domain, string Vhost);

public record ListQueueResponse(RemoteQueue RemoteQueue, bool ExistsLocally);

public record RemoteQueue(string Name, uint MessageCount, bool Exclusive);

public record FindQueueByNameResponse(uint? QueueId);
public record LoadMessagesByQueueNameResponse(List<Message> Messages);
public record Message(uint Id, string Payload);



[JsonSerializable(typeof(Message))]
[JsonSerializable(typeof(LoadMessagesByQueueNameResponse))]
[JsonSerializable(typeof(List<ListQueueResponse>))]
[JsonSerializable(typeof(RmqConnectionInfo))]
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower)]
public partial class MySourceGenerationContext: JsonSerializerContext
{ }