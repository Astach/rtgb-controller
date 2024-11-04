# RTGB Controller

RTGB Controller is responsible for scheduling command to controller an active fermentations

## Overview

1. Receives an event from the RTGB API that includes the fermentation steps to send to a chamber
2. Convert the event to the corresponding scheduling commands
3. Store in a DB all the scheduling commands.
4. Every minute checks the DB, fire the command that needs to be sent to the hardware (send to MQTT broker)
5. Update the command as Sent
6. Update the command as Acknowledged when the socket responds to the command (via MQTT)
7. Delete the scheduled commands once the StopFermentation command is Acknowledged/ (or sent?)

### Command description

_METADATA_

- ID <Command ID>
- SentAt <Epoch of the command sending>
- Version <Command Version>
- Type <Command Type>
  - StartFermentation: Start the fermentation at the given `Value` in degree Celcius. e.g. Start 22
  - IncreaseTemperature: Increase the temperature of the given `Value` in degree Celcius. e.g. Increase 1.5
  - DecreaseTemperature: Decrease the temperature of the given `Value` in degree Celcius. e.g. Decrease 1.5
  - StopFermentation: Stop the fermentation at the given `Value`. e.g. Stop 20

_DATA_

- Value: A temperature value, can represent a temperature in Celcius or an absolute delta
- Session : The session identifier associated with this command
- Target: The cooling or heating hardware identifier
- Date: When to fire the command
- Status
  - Planned: The command sent at `Date`
  - Sent: The command has been sent at `Date`
  - Acknowledged: The command has been received by the hardware

#### Examples

- `StartFermentation 22 da0ef064-a093-4fad-9a06-120ddaa9e87c #12ADFC 1729579120 Acknowledged`
- `IncreaseTemperature 4 da0ef064-a093-4fad-9a06-120ddaa9e87c #12ADFC 1729579120 Planned`

## Start the project

### For testing and development purposes, generate a self signed certificate

1. Client side
   a. Generate CA private key and certificate

```
openssl req -x509 -nodes -newkey rsa:4096 -days 365 \
    -keyout ca.key -out ca.crt \
    -subj "/CN=NATS CA/O=My Organization/C=US"
```

b. Generate client private key

```
openssl genrsa -out client.key 4096
```

c. Generate client Certificate Signing Request (CSR)

```
openssl req -new -key client.key -out client.csr \
    -subj "/CN=nats-client/O=My Organization/C=US"
```

d. Sign client certificate with CA [!CAUTION]

```
openssl x509 -req -days 365 -in client.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out client.crt
```

e. Set permissions

```
chmod 600 *.key
chmod 644 *.crt
```

2. Server side
   a. Generate CA private key and certificate

```
openssl req -x509 -nodes -newkey rsa:4096 -days 365 \
    -keyout ca.key -out ca.crt \
    -subj "/CN=NATS CA/O=My Organization/C=US"
```

b. Generate server private key

```
openssl genrsa -out server.key 4096
```

c. Generate server Certificate Signing Request (CSR)

```
openssl req -new -key server.key -out server.csr \
    -subj "/CN=localhost/O=My Organization/C=US" \
    -addext "subjectAltName = DNS:localhost,DNS:nats-server,IP:127.0.0.1"
```

d. Sign server certificate with CA

```
openssl x509 -req -days 365 -in server.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server.crt \
    -extfile <(printf "subjectAltName=DNS:localhost,DNS:nats-server,IP:127.0.0.1")
```

e. Add the server conf

```
tls {
  cert_file: "./certs/server.crt"
  key_file: "./certs/server.key"
  ca_file: "./certs/ca.crt"
  verify: true
}
```

## Rules

- The first command must be a `StartFermentation` command
- The last command must be a `StopFermentation` command
- There can be only one `StartFermentation` and one `StopFermentation` command

The commands are sent over MQTT using MATTER protocol this means the payload is sent using protobuf.

- HEADER: Contains message metadata

  - ID
  - SentAt
  - Version
  - Type

- PAYLOAD:
  - ClusterID
  - AttributesID
  - Value
  - Target
