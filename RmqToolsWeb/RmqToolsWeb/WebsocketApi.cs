using System.Net.WebSockets;
using System.Text.Json;

namespace RmqToolsWeb;

public class WebsocketApi(Uri uri) : IAsyncDisposable
{
    readonly CancellationTokenSource _cts = new();
    readonly ClientWebSocket _webSocket = new();
    
    public async Task StartAsync(Func<Dictionary<string, QueueCounters>, Task> onReceive)
    {
        await _webSocket.ConnectAsync(uri, cancellationToken: _cts.Token);
        
        var bytes = new byte[10000];
        _ = Task.Run(async () =>
        {
            while (!_cts.Token.IsCancellationRequested)
            {
                (bytes, var length) = await ReadMessageBytesAsync(bytes);
                var list = JsonSerializer.Deserialize<List<QueueCounters>>(bytes[.. length], MySourceGenerationContext.Default.ListQueueCounters);
                var dict = list!.ToDictionary(x => x.QueueName);
                
                await onReceive(dict);
            }
        }, _cts.Token);
    }

    async Task<(byte[] buffer, int messageLength)> ReadMessageBytesAsync(byte[] buffer)
    {
        var receiveResult = await _webSocket.ReceiveAsync(buffer, _cts.Token);
        if (receiveResult.EndOfMessage)
            return (buffer, receiveResult.Count);
        
        var buffers = new List<byte[]> { buffer };
        while (!receiveResult.EndOfMessage)
        {
            var newBuffer = new byte[buffers[^1].Length * 2];
            buffers.Add(newBuffer);
         
            receiveResult = await _webSocket.ReceiveAsync(newBuffer, _cts.Token);
        }

        var finalBuffer = new byte[buffers.Select(x => x.Length).Sum()];
        var messageLength = 0;
        for (var i = 0; i < buffers.Count; i++)
        {
            var length = i == buffers.Count - 1 ? receiveResult.Count : buffers[i].Length;
            buffers[i].CopyTo(finalBuffer, messageLength);
            messageLength += length;
        }
        
        return (finalBuffer, messageLength);
    }
    
    public async ValueTask DisposeAsync()
    {
        await _cts.CancelAsync();
        _cts.Dispose();
        
        _webSocket.Abort();
        _webSocket.Dispose();
    }
}