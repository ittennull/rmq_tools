namespace RmqToolsWeb.Dtos;

public record ListQueueResponse(RemoteQueue RemoteQueue, bool ExistsLocally);

public record RemoteQueue(string Name, uint MessageCount, bool Exclusive);
