using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using Microsoft.AspNetCore.Components;
using MudBlazor;
using RmqToolsWeb.Components;

namespace RmqToolsWeb.Pages;

public partial class Queue
{
    enum ShowMessageParts { Both, Headers, Payload }
    
    record MessageItem(int Index, Message Message, string CombinedString);
    
    [Parameter]
    public required string QueueName { get; set; }
    
    [Inject] public HttpClient Http { get; set; }
    [Inject] IDialogService DialogService { get; set; }

    bool _loading = true;
    readonly List<MessageItem> _messages = [];
    HashSet<MessageItem> _selectedMessages = [];
    string _moveToQueue = "";
    string[] _queueNames = [];
    int _numberOfRemoteMessages;
    int _numberOfMessagesInDb;
    uint? _queueId;
    ShowMessageParts _showMessageParts = ShowMessageParts.Both;
    string _lineFilter = "";
    bool? _readonlyMode;
    string _groupBySelector = "";
    Func<MessageItem, object>? _groupBy;
    MudDataGrid<MessageItem> _dataGrid;

    bool CanSendOrDeleteMessages => _queueId != null && _messages.Count != 0;
    private bool GroupingEnabled => _groupBySelector != string.Empty;
    
    protected override async Task OnInitializedAsync()
    {
        _moveToQueue = QueueName;

        _groupBy = x =>
        {
            var (wholeLine, _) = FindString(x.CombinedString, _groupBySelector, 0);
            return wholeLine;
        };

        var queues = (await Http.GetFromJsonAsync<List<QueueSummary>>("/api/queues", MySourceGenerationContext.Default.ListQueueSummary))!;
        _queueNames = queues
            .Where(x => !x.Exclusive)
            .Select(x => x.Name)
            .Order()
            .ToArray();

        if (queues.SingleOrDefault(x => x.Name == QueueName) is { } requestedQueue)
        {
            _numberOfRemoteMessages = requestedQueue.MessageCountInRmq;

            if (requestedQueue.QueueId != null)
            {
                _queueId = requestedQueue.QueueId;

                if (requestedQueue.MessageCountInDb > 0)
                {
                    _readonlyMode = false;
                    var messages = (await Http.GetFromJsonAsync<List<Message>>($"/api/queues/{_queueId}/messages", MySourceGenerationContext.Default.ListMessage))!;
                    CreateMessageItems(messages);
                    _numberOfMessagesInDb = messages.Count;
                }
            }
            
            if (_readonlyMode == null && requestedQueue.MessageCountInRmq > 0)
            {
                _readonlyMode = true;
                var messages = (await Http.GetFromJsonAsync<List<Message>>($"/api/queue/peek?queue_name={QueueName}", MySourceGenerationContext.Default.ListMessage))!;
                CreateMessageItems(messages);
            }
        }
        
        _loading = false;
    }

    async Task LoadMessages()
    {
        _loading = true;
        try
        {
            using var response = await Http.PostAsync($"/api/queue/load?queue_name={QueueName}", null);
            var content = (await response.Content.ReadFromJsonAsync<LoadMessagesByQueueNameResponse>(MySourceGenerationContext.Default.LoadMessagesByQueueNameResponse))!;
            _queueId = content.QueueId;
            CreateMessageItems(content.Messages);
            _numberOfRemoteMessages = 0;
            _numberOfMessagesInDb = _messages.Count;
            _readonlyMode = false;
        }
        finally
        {
            _loading = false;
        }
    }

    void UnselectAll()
    {
        _selectedMessages.Clear();
    }

    async Task DeleteMessages()
    {
        ShowLoadingForOperationOnMessages();
        
        var body = new DeleteMessagesRequest(_selectedMessages.Select(x => x.Message.Id));
        using var httpRequestMessage = new HttpRequestMessage(HttpMethod.Delete, $"/api/queues/{_queueId}/messages");
        httpRequestMessage.Content = new ByteArrayContent(JsonSerializer.SerializeToUtf8Bytes(body, MySourceGenerationContext.Default.DeleteMessagesRequest));
        httpRequestMessage.Content.Headers.ContentType = new("application/json");
        using var response = await Http.SendAsync(httpRequestMessage);
        response.EnsureSuccessStatusCode();

        ClearMessagesAfterOperation();
        
        _loading = false;
    }
    
    async Task SendMessagesToQueue()
    {
        ShowLoadingForOperationOnMessages();
        
        var body = new SendMessagesRequest(_moveToQueue, _selectedMessages.Select(x => x.Message.Id));
        using var response = await Http.PostAsJsonAsync($"/api/queues/{_queueId}/messages/send", body, MySourceGenerationContext.Default.SendMessagesRequest);
        response.EnsureSuccessStatusCode();

        if (_moveToQueue == QueueName)
            _numberOfRemoteMessages += _selectedMessages.Count == 0 ? _messages.Count : _selectedMessages.Count;
        
        ClearMessagesAfterOperation();
        
        _loading = false;
    }

    void ClearMessagesAfterOperation()
    {
        if (_selectedMessages.Count == 0)
            _messages.Clear();
        else
            _messages.RemoveAll(x => _selectedMessages.Contains(x));
        _numberOfMessagesInDb = _messages.Count;
        _selectedMessages.Clear();
    }
    
    string MessageWord(int count) => count == 1 ? "message" : "messages";

    void ShowLoadingForOperationOnMessages()
    {
        const int threshold = 100;
        if (_selectedMessages.Count > threshold || (_selectedMessages.Count == 0 && _messages.Count > threshold))
            _loading = true;
    }

    void CreateMessageItems(IEnumerable<Message> messages)
    {
        _messages.Clear();
        _messages.AddRange(messages
            .Index()
            .Select(x => new MessageItem(x.Index + 1, x.Item, GetCombinedString(x.Item)))
            .OrderBy(x => x.Index));
    }

    string GetCombinedString(Message message)
    {
        var sb = new StringBuilder();
        
        foreach (var kvp in message.Headers)
        {
            sb.Append(kvp.Key);
            if (kvp.Value.ValueKind == JsonValueKind.Object)
            {
                foreach (var prop in kvp.Value.EnumerateObject())
                {
                    sb.Append(prop.Name);
                    sb.Append(": ");
                    sb.Append(prop.Value);
                    sb.AppendLine();
                }
            }
            else
            {
                sb.Append(": ");
                sb.Append(kvp.Value);
                sb.AppendLine();
            }
        }

        sb.Append(message.Payload);

        return sb.ToString();
    }

    void SetLineFilter(string s)
    {
        _lineFilter = s;
    }

    IEnumerable<string> GetFilteredLines(string line)
    {
        if (string.IsNullOrWhiteSpace(_lineFilter))
        {
            yield return line;
            yield break;
        }

        int index = 0;
        while (true)
        {
            var (wholeLine, endIndex) = FindString(line, _lineFilter, index);
            if (wholeLine != null)
                yield return wholeLine;
            
            if (wholeLine == null || endIndex == line.Length)
                yield break;
            
            index = endIndex;
        }
    }

    (string? wholeLine, int endIndex) FindString(string line, string token, int lineStartIndex)
    {
        if (line.Length == 0)
            return default;
        
        var index = line.IndexOf(token, lineStartIndex, StringComparison.InvariantCultureIgnoreCase);
        if (index == -1)
            return default;
        
        // find a start of the string
        var startIndex = line.LastIndexOf('\n', index);
        startIndex += 1;
        
        // find an end of the string
        var endIndex = line.IndexOf('\n', index + token.Length);
        endIndex = endIndex == -1 ? line.Length : endIndex + 1;

        return (line[startIndex .. endIndex], endIndex);
    }

    async Task ExportFilteredLines()
    {
        var sb = new StringBuilder();

        foreach (var messageItem in _dataGrid.FilteredItems)
        {
            if (_showMessageParts is ShowMessageParts.Headers or ShowMessageParts.Both)
            {
                foreach (var kvp in messageItem.Message.Headers)
                {
                    if (kvp.Value.ValueKind == JsonValueKind.Object)
                    {
                        if (GetFilteredLines($"{kvp.Key}: ").FirstOrDefault() is { } line1)
                            sb.AppendLine(line1.Trim());
                        
                        foreach (var prop in kvp.Value.EnumerateObject())
                        {
                            foreach (var line2 in GetFilteredLines($"{prop.Name}: {prop.Value.ToString()}"))
                            {
                                sb.AppendLine(line2.Trim());
                            }
                        }
                    }
                    else
                    {
                        if (GetFilteredLines($"{kvp.Key}: {kvp.Value}").FirstOrDefault() is { } line)
                            sb.AppendLine(line.Trim());
                    }
                }
            }
        
            if (_showMessageParts is ShowMessageParts.Payload or ShowMessageParts.Both)
            {
                foreach (var line in GetFilteredLines(messageItem.Message.Payload))
                {
                    sb.AppendLine(line.Trim());
                }
            }
        }
        
        var parameters = new DialogParameters<ExportFilteredLinesDialog>
        {
            { x => x.Lines, sb.ToString() },
        };
        var options = new DialogOptions { CloseOnEscapeKey = true, MaxWidth = MaxWidth.Large };
        await DialogService.ShowAsync<ExportFilteredLinesDialog>("Filtered lines", parameters, options);
    }

    async Task EditMessage(int messageIndex)
    {
        var index = messageIndex - 1;
        var message = _messages[index].Message;
        var parameters = new DialogParameters<EditMessageDialog>
        {
            { x => x.QueueId, _queueId!.Value },
            { x => x.Message, message },
        };
        var options = new DialogOptions { CloseOnEscapeKey = true, MaxWidth = MaxWidth.ExtraLarge };
        var dialog = await DialogService.ShowAsync<EditMessageDialog>("Edit message payload", parameters, options);
        var result = await dialog.Result;
        if (!result!.Canceled)
        {
            _messages[index] = _messages[index] with
            {
                Message = _messages[index].Message with { Payload = (string)result.Data! }
            };
        }
    }

    void SelectAllInGroup(IGrouping<object, MessageItem> group)
    {
        foreach (var messageItem in group)
        {
            _dataGrid.SelectedItems.Add(messageItem);
        }
    }
}