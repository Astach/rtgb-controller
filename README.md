# RTGB Controller

RTGB Controller is responsible for scheduling command to control active fermentations

## Overview

1. Receives an event from the RTGB API that includes the fermentation steps to send to a chamber
2. Convert the event to the corresponding scheduling commands
3. Store in a DB all the scheduling commands.
4. On event received by the hydrometer checks the DB, fire the command that needs to be sent to the hardware (send to MQTT broker)
5. Update the command as Running
6. On every check, verify that if the target_temperature is reached, it has been held for the specified duration.
7. Once the step is done, update the command to Executed.

### New Command description

- id: Command UUID
- sent_at: Epoch of the command sending
- version: Command Version
- session_data: The session uuid and the step position
- value: A temperature value, can represent a temperature in Celcius or an absolute delta
- value_holding_duration: The time amount to wait before executing the next command once the target temperature has been reached
- Status
  - Planned: The command will be sent
  - Running: The command is currently running
  - Executed: The command has been executed, we can move on to the next one.

## Start the project

### For testing and development purposes, generate a self signed certificate

1. Create the following folders `certs`, `certs/server` and `certs/client`

2. Generate CA private key and certificate

```bash
openssl req -x509 -nodes -newkey rsa:4096 -days 365 \
    -keyout ca.key -out ca.crt \
    -subj "/CN=RTGB CA/O=My Organization/C=US"
```

#### Client side

1. Generate client private key

```bash
openssl genrsa -out client/client.key 4096
```

2. Generate client Certificate Signing Request (CSR)

```bash
openssl req -new -key client/client.key -out client/client.csr \
    -subj "/CN=RTGB/O=My Organization/C=US"
```

3. Sign client certificate with CA

```bash
openssl x509 -req -days 365 -in client/client.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out client/client.crt
```

4. Set permissions

```bash
chmod 600 *.key
chmod 644 *.crt
```

#### Server side

1. Generate server private key

```bash
openssl genrsa -out server/server.key 4096
```

2. Generate server Certificate Signing Request (CSR)

```bash
openssl req -new -key server/server.key -out server/server.csr \
    -subj "/CN=RTGB/O=My Organization/C=US" \
    -addext "subjectAltName = DNS:localhost,IP:127.0.0.1"
```

3. Sign server certificate with CA

```bash
 echo $"subjectAltName=DNS:localhost,IP:127.0.0.1" |
 openssl x509 -req -days 365 -in server/server.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server/server.crt \
    -extfile /dev/stdin
```

### NATS

#### Configuration

1. Create or edit a `.env` file at `/docker/.env` by using `/docker/.env.template` and fill the NATS values
2. Add/Edit the nats server conf at `/docker/nats/server.conf`

```
tls {
  cert_file: "./certs/server.crt"
  key_file: "./certs/server.key"
  ca_file: "./certs/ca.crt"
  verify: true
}
```

#### Usage

1. Install `direnv` and create a `.envrc` file
2. add the following variables

```bash
export DATABASE_URL="postgres@..." # used by sqlx to check query at compile time
export NATS_URL=nats://localhost:4222
export NATS_CA=/path/to/certs/ca.crt
export NATS_CERT=/path/to/certs/client.crt
export NATS_KEY=/path/to/certs/client.key
export NATS_TLS_VERIFY=true
```

**Important**: use absolute paths in your database url for certificates' path, you can use `${PWD}` e.g.: `${PWD}/certs/root.ca`.

3. Install `nats` along with `nsc` cli and export the following variables
4. Create your [server.conf](./docker/nats/server.template.conf), check out the links in the template.
5. Create a nats context `nats context add myuser --creds ~/.local/share/nats/nsc/keys/creds/MyOperator/MyAccount/MyUser.creds`, you can edit it to add the variables from step 2 if you prefer. Don't forget to select the context `nats context select` afterwards.
6. Launch the nats server using `docker compose up`
7. Send a message `nats publish <subject> <message>`
8. Subscribe to subject `nats subscribe <subject>`

### Postgres

1. Create or edit a `.env` file at `/docker/.env` by using `/docker/.env.template` and fill out the POSTGRES values
2. Add/Edit the posgresql conf at `/docker/postgres/postgresql.conf`
3. Add/Edit the hba conf at `/docker/postgres/pg_hba.conf`

### Service Configuration

1. Create a `config.toml` file at `./app/config.toml` by using `./app/config.template.toml` and this the values accordingly
2. Run the app `RUST_LOG=debug cargo run`

## Rules

### Events

- You can find the documentation for the schedule events received from the API [there](https://github.com/Astach/rtgb?tab=readme-ov-file#command-description).
- You can fine the documentation for the events reveived from MQTT [there](). //TODO

### Scheduling Command

- After the last command is in Executed State, we stop the fermentation by sending a turn off to the heating and cooling device.

### Command firing rules

- The first command is not instantly triggered as we don't know what is the current temperature of the fermentation chamber. Once the first value of the hydrometer is received, the command will be sent and increase or decrease the temperature to reach the desired temperature.
- Once a command is has the status `Running`, on the next event received from the hydrometer, check if the `target_temperature` is reached, if yes we can consider that the step has started for its given duration.

## FAQ

- Access the pg container `docker exec -it <container_id>  /bin/bash`
- Access the scheduler database:

```bash
psql $"host=127.0.0.1 port=5432 dbname=<db_name> user=<db_user> sslmode=verify-full sslcert=certs/client/client.crt sslkey=certs/client/client.key sslrootcert=certs/ca.crt"
```

- Unable to parse the json received via Nats subject: make sure to wrap your payload with single quote (`'`) not doubles (`"`)
  e.g. :

```zsh
nats publish fermentation.schedule.command '{
     "id": "550e8400-e29b-41d4-a716-446655440000",
     "sent_at": "2024-12-15T12:34:56Z",
     "version": 1,
     "type": "Schedule",
     "data": {
         "session_id": "486190da-9691-4e52-b085-7e270829766b",
         "hardwares": [
            {
              "id": "hw#1",
              "hardware_type": "Cooling"
             },
            {
              "id": "hw#2",
              "hardware_type": "Heating"
             }
         ],
         "steps": [
             {
                 "position": 0,
                 "target_temperature": 20,
                 "duration": 96,
             },
             {
                 "position": 1,
                 "target_temperature": 24,
                 "rate": {
                     "value": 2,
                     "duration": 1
                 },
                 "duration": 72,
             },
             {
                 "position": 3,
                 "target_temperature": 2,
                 "rate": {
                     "value": 4,
                     "duration": 6
                 },
                 "duration": 48,
             }
         ]
     }
 }'
```

- Run unit tests:
  `cargo nextest run --profile test`
