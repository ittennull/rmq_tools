using RmqToolsWeb.Dtos;

namespace RmqToolsWeb.Pages;

public partial class Queue
{
    List<Message> GetMessages()
    {
        return Enumerable.Range(0, 10000).Select(i =>
        {
            return new Message((uint)i,
                $$"""
                  	
                  {
                    "messageId": "{{i}} - 01000000-f499-3ecf-519c-08de116932bd",
                    "conversationId": "01000000-f499-3ecf-5221-08de116932bd",
                    "sourceAddress": "rabbitmqs://cloudamqp.subscribe.rtl.nl/subscribe/subscription55_RtlSubscriptionApi_bus_yryyyy8wur9c9xzwbdxbnaqjrq?temporary=true",
                    "destinationAddress": "rabbitmqs://cloudamqp.subscribe.rtl.nl/subscribe/Rtl.Subscription.IntegrationEvents:TrustedPartyOfferSwitched",
                    "messageType": [
                      "urn:message:Rtl.Subscription.IntegrationEvents:TrustedPartyOfferSwitched"
                    ],
                    "message": {
                      "subscriptionId": "xZvUr7W0m1gD0ERE6PUXo",
                      "subscriptionNumber": "S-8726720",
                      "userId": "07a706a7704a4ee491fe8eb974b958f5",
                      "previousMainOfferId": "Basis_TrustedParty",
                      "newMainOfferId": "Plus_TrustedParty",
                      "trustedParty": "tmobile"
                    },
                    "sentTime": "2025-10-22T12:47:45.3695388Z",
                    "headers": {
                      "MT-Activity-Id": "00-4fea208c5599cbeedeac543443352c94-08958af1777c0ed2-01"
                    },
                    "host": {
                      "machineName": "subscription-55bc9779d6-2s6k8",
                      "processName": "Rtl.Subscription.Api",
                      "processId": 1,
                      "assembly": "Rtl.Subscription.Api",
                      "assemblyVersion": "1.0.0.0",
                      "frameworkVersion": "9.0.10",
                      "massTransitVersion": "8.5.3.0",
                      "operatingSystemVersion": "Unix 5.15.0.1091"
                    }
                  }
                  """);
        }).ToList();
    }
}