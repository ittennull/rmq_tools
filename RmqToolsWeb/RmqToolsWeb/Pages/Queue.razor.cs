using System.Text;
using System.Text.Json;
using Microsoft.AspNetCore.Components;
using MudBlazor;
using RmqToolsWeb.Components;

namespace RmqToolsWeb.Pages;

public partial class Queue : IAsyncDisposable
{
    enum ShowMessageParts { Both, Headers, Payload }
    
    record MessageItem(int Index, uint MessageId, string CombinedString, List<string> HeaderLines, List<string> PayloadLines);
    
    [Parameter]
    public required string QueueName { get; set; }

    [Inject] Api Api { get; set; } = null!;
    [Inject] IDialogService DialogService { get; set; } = null!;
    [Inject] WebsocketApi WebsocketApi { get; set; } = null!;

    bool _loading = true;
    readonly List<MessageItem> _messages = [];
    HashSet<MessageItem> _selectedMessages = [];
    string _moveToQueue = "";
    string[] _queueNames = [];
    bool _numberOfRemoteMessagesIsOutOfDate;
    int _numberOfRemoteMessages;
    int _numberOfMessagesInDb;
    uint? _queueId;
    ShowMessageParts _showMessageParts = ShowMessageParts.Both;
    string _lineFilter = "";
    bool? _readonlyMode;
    string _groupBySelector = "";
    MudDataGrid<MessageItem> _dataGrid = null!;

    bool CanSendOrDeleteMessages => _queueId != null && _messages.Count != 0;
    bool GroupingEnabled => _groupBySelector != string.Empty;
    
    protected override async Task OnInitializedAsync()
    {
        _moveToQueue = QueueName;

        var queues = await Api.GetQueueSummariesAsync();
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
                    var messages = await Api.GetMessagesFromDbAsync(_queueId.Value);
                    CreateMessageItems(messages);
                    _numberOfMessagesInDb = messages.Count;
                }
            }
            
            if (_readonlyMode == null && requestedQueue.MessageCountInRmq > 0)
            {
                _readonlyMode = true;
                var messages = await Api.PeekRmqMessagesAsync(QueueName);
                CreateMessageItems(messages);
            }
        }

        await StartWebsocket();
        
        _loading = false;
    }
    
    async Task StartWebsocket()
    {
        await WebsocketApi.StartAsync(async queueCounters =>
        {
            await InvokeAsync(() =>
            {
                _numberOfRemoteMessages = queueCounters.SingleOrDefault(x => x.Key == QueueName).Value?.Messages ?? 0;
                _numberOfRemoteMessagesIsOutOfDate = false;
                StateHasChanged();
            });
        });
    }

    async Task LoadMessages()
    {
        _loading = true;
        try
        {
            var (queueId, messages) = await Api.LoadMessagesToDbAsync(QueueName);
            _queueId = queueId;
            CreateMessageItems(messages);
            _numberOfRemoteMessages = 0;
            _numberOfRemoteMessagesIsOutOfDate = false;
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

        var messageIds = _selectedMessages.Select(x => x.MessageId);
        await Api.DeleteMessagesAsync(_queueId!.Value, messageIds);

        ClearMessagesAfterOperation();

        //update indexes
        for (var i = 0; i < _messages.Count; i++)
            _messages[i] = _messages[i] with { Index = i + 1 };
        
        _loading = false;
    }
    
    async Task SendMessagesToQueue()
    {
        ShowLoadingForOperationOnMessages();
        
        var messageIds = _selectedMessages.Select(x => x.MessageId);
        await Api.SendMessagesToQueueAsync(_queueId!.Value, messageIds, _moveToQueue);

        if (_moveToQueue == QueueName)
            _numberOfRemoteMessagesIsOutOfDate = true;
        
        ClearMessagesAfterOperation();
        
        _loading = false;
    }
    
    async Task SendMessagesToQueueWithDelay()
    {
        var options = new DialogOptions
        {
            BackdropClick = false, 
            CloseOnEscapeKey = false,
            MaxWidth = MaxWidth.ExtraLarge
        };
        
        var messageIds = _selectedMessages.Select(x => x.MessageId).ToList();
        var messageCount = messageIds.Count > 0 ? messageIds.Count : _numberOfMessagesInDb;
        var parameters = new DialogParameters<SendWithDelayDialog>
        {
            { x => x.QueueId, _queueId!.Value },
            { x => x.MessageIds, messageIds},
            { x => x.MoveToQueue, _moveToQueue },
            { x => x.MessagesCount, messageCount }
        };
        
        var dialog = await DialogService.ShowAsync<SendWithDelayDialog>($"Send {messageCount} {MessageWord(messageCount)} with delay", parameters, options);
        var result = await dialog.Result;

        if (!result.Canceled)
        {
            if (_moveToQueue == QueueName)
                _numberOfRemoteMessagesIsOutOfDate = true;
        
            ClearMessagesAfterOperation();
        }
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
            .Select(x =>
            {
                var headerLines = CreateHeaderLines(x.Item.Headers);
                var payloadLines = CreatePayloadLines(x.Item.Payload);
                var combinedString = GetCombinedString(headerLines, payloadLines);
                return new MessageItem(x.Index + 1, x.Item.Id, combinedString, headerLines, payloadLines);
            })
            .OrderBy(x => x.Index));
    }

    List<string> CreatePayloadLines(string payload) => payload.Split('\n').Select(x => $"{x}\n").ToList();

    List<string> CreateHeaderLines(Dictionary<string, JsonElement> headers)
    {
        var list = new List<string>(headers.Count);
        foreach (var kvp in headers)
        {
            if (kvp.Value.ValueKind == JsonValueKind.Object)
            {
                list.Add($"{kvp.Key}:\n");
                foreach (var prop in kvp.Value.EnumerateObject())
                {
                    list.Add($"    {prop.Name}: {prop.Value}\n");
                }
            }
            else
            {
                list.Add($"{kvp.Key}: {kvp.Value}\n");
            }
        }

        return list;
    }

    string GetCombinedString(List<string> headerLines, List<string> payloadLines)
    {
        var sb = new StringBuilder();

        foreach (var line in headerLines)
            sb.AppendLine(line);
        foreach (var line in payloadLines)
            sb.AppendLine(line);

        return sb.ToString();
    }

    void SetLineFilter(string s)
    {
        _lineFilter = s;
    }

    string? FilterLine(string line, string filter) =>
        line.Contains(filter, StringComparison.InvariantCultureIgnoreCase)
            ? line
            : null;

    async Task ExportFilteredLines()
    {
        var sb = new StringBuilder();

        foreach (var messageItem in _dataGrid.FilteredItems)
        {
            if (_showMessageParts is ShowMessageParts.Headers or ShowMessageParts.Both)
            {
                foreach (var line in messageItem.HeaderLines)
                {
                    if (FilterLine(line, _lineFilter) is { } filteredLine)
                        sb.AppendLine(filteredLine.Trim());
                }
            }
        
            if (_showMessageParts is ShowMessageParts.Payload or ShowMessageParts.Both)
            {
                foreach (var line in messageItem.PayloadLines)
                {
                    if (FilterLine(line, _lineFilter) is { } filteredLine)
                        sb.AppendLine(filteredLine.Trim());
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
        var message = _messages[index];
        var parameters = new DialogParameters<EditMessageDialog>
        {
            { x => x.QueueId, _queueId!.Value },
            { x => x.MessageId, message.MessageId },
            { x => x.MessagePayload, string.Join("", message.PayloadLines) },
        };
        var options = new DialogOptions { CloseOnEscapeKey = true, MaxWidth = MaxWidth.ExtraLarge };
        var dialog = await DialogService.ShowAsync<EditMessageDialog>("Edit message payload", parameters, options);
        var result = await dialog.Result;
        if (!result!.Canceled)
        {
            var payloadLines = CreatePayloadLines((string)result.Data!);
            _messages[index] = _messages[index] with
            {
                PayloadLines = payloadLines,
                CombinedString = GetCombinedString(_messages[index].HeaderLines, payloadLines)
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

    object GroupMessagesBy(MessageItem messageItem)
    {
        foreach (var line in messageItem.HeaderLines)
        {
            if (FilterLine(line, _groupBySelector) != null)
                return line;
        }
        foreach (var line in messageItem.PayloadLines)
        {
            if (FilterLine(line, _groupBySelector) != null)
                return line;
        }

        return null!;
    }

    public ValueTask DisposeAsync()
    {
        return WebsocketApi.DisposeAsync();
    }
}