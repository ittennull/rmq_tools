using System.Net.Http.Json;
using System.Text.Json;

namespace RmqToolsWeb;

public class Api(HttpClient http)
{
    public async Task DeleteMessagesAsync(uint queueId, IEnumerable<uint> messageIds)
    {
        var body = new DeleteMessagesRequest(messageIds);
        using var httpRequestMessage = new HttpRequestMessage(HttpMethod.Delete, $"/api/queues/{queueId}/messages");
        httpRequestMessage.Content = new ByteArrayContent(JsonSerializer.SerializeToUtf8Bytes(body, MySourceGenerationContext.Default.DeleteMessagesRequest));
        httpRequestMessage.Content.Headers.ContentType = new("application/json");
        using var response = await http.SendAsync(httpRequestMessage);
        response.EnsureSuccessStatusCode();
    }
    
    public async Task SendMessagesToQueueAsync(uint queueId, IEnumerable<uint> messageIds, string moveToQueue)
    {
        var body = new SendMessagesRequest(moveToQueue, messageIds);
        using var response = await http.PostAsJsonAsync($"/api/queues/{queueId}/messages/send", body, MySourceGenerationContext.Default.SendMessagesRequest);
        response.EnsureSuccessStatusCode();
    }

    public async Task<List<QueueSummary>> GetQueueSummariesAsync()
    {
        return (await http.GetFromJsonAsync<List<QueueSummary>>("/api/queues", MySourceGenerationContext.Default.ListQueueSummary))!;
    }

    public async Task<List<Message>> GetMessagesFromDbAsync(uint queueId)
    {
        return (await http.GetFromJsonAsync<List<Message>>($"/api/queues/{queueId}/messages", MySourceGenerationContext.Default.ListMessage))!;
    }
    
    public async Task<List<Message>> PeekRmqMessagesAsync(string queueName)
    {
        return (await http.GetFromJsonAsync<List<Message>>($"/api/queue/peek?queue_name={queueName}", MySourceGenerationContext.Default.ListMessage))!;
    }
    
    public async Task<LoadMessagesByQueueNameResponse> LoadMessagesToDbAsync(string queueName)
    {
        using var response = await http.PostAsync($"/api/queue/load?queue_name={queueName}", null);
        return (await response.Content.ReadFromJsonAsync<LoadMessagesByQueueNameResponse>(MySourceGenerationContext.Default.LoadMessagesByQueueNameResponse))!;
    }

    public async Task<EnvInfo> GetEnvInfoAsync()
    {
        return (await http.GetFromJsonAsync<EnvInfo>("/api/env_info", MySourceGenerationContext.Default.EnvInfo))!;
    }

    public async Task SaveMessageAsync(uint queueId, uint messageId, string messagePayload)
    {
        using var body = new StringContent(messagePayload);
        using var response = await http.PutAsync($"/api/queues/{queueId}/messages/{messageId}", body);
        response.EnsureSuccessStatusCode();
    }
}