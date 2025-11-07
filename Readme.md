A tool that gives you UI to work with RabbitMQ messages.

Standard RabbitMQ management UI allows you to read messages, send them from one queue to another and purge queues. 
The inconvenience is that it's not possible to for example delete the 5th message from a queue that has 10 messages. You can only delete all of them.



### How it works
Run the UI with a command:
```bash
rmq_tools -u https://user:password@my-server.com/api -v my-vhost
```
Open http://localhost:3000 (you can use another port with option `-p`)

The home page shows all queues. Select one of them to go to the queue page.

If the queue isn't empty, the messages will be shown on the page. You can't edit anything currently.

To be able to edit/delete/send the messages, click the button "Load messages". This operation takes the messages out of RabbitMQ queue and stores them in a local sqlite database. Now it's possible to work with messages in any order.

Once all changes are made, the messages can be sent back to the original queue or any other queue