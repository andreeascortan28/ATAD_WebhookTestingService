# Webhook Testing Service

Service Application meant to be used by developers to interact with webhooks with the purpose of testing their functionalities.

## Executing program

* Set up this environment variable for the database:
```
DATABASE_URL=sqlite://./webhooks.db
```
* Run the command:
```
run --package webhook_tester --bin webhook_tester
```
* The app now runs on the URL:
```
http://localhost:3000
```

## Functionalities

* "/new" endpoint to generate unique webhook and save it to database
* "/webhook/:id" to store webhook temporarily before saving to SQLite Database
* "/dashboard/:id" endpoint to inspect existing webhooks and all requests made to them 
* "/ws/:id" to run a websocket in order to update dashboard with new info from Database in real-time
* "/replay" endpoint to replay a webhook
* "/webhook/:id/config" endpoint to configure custom responses