# RTGB Controller

RTGB Controller is responsible for scheduling command to controller an active fermentations

## Overview

1. Receives an event from the RTGB API that includes the fermentation steps to send to a chamber
2. Convert the event to the corresponding scheduling commands
3. Store in a DB all the scheduling commands. Command description:
   - Type <Start|Increase|Decrease|Stop|>
   - Value <Float>
   - Session <Session ID>
   - Target <Cooling|Heating HardwareID>
   - Date <Epoch> Status<Planned|Sent|Acknowledged>
4. Every minute checks the DB, fire the command that needs to be sent to the hardware (send to MQTT broker)
5. Update the command as Sent
6. Update the command as Acknowledged when the socket responds to the command (via MQTT)
7. Delete the scheduled commands once the STOP command is Acknowledged/ (or sent?)

### Command description

- Type

  - Start: Start the fermentation at the given `Value` in degree Celcius. e.g. Start 22
  - Increase: Increase the temperature of the given `Value` in degree Celcius. e.g. Increase 1.5
  - Decrease: Decrease the temperature of the given `Value` in degree Celcius. e.g. Decrease 1.5
  - Stop: Stop the fermentation at the given `Value`. e.g. Stop 20

- Value: A temperature value, can represent a temperature in Celcius or an absolute delta
- Session : The session identifier associated with this command
- Target: The cooling or heating hardware identifier
- Date: When to fire the command
- Status
  - Planned: The command sent at `Date`
  - Sent: The command has been sent at `Date`
  - Acknowledged: The command has been received by the hardware

#### Examples

- `Start 22 da0ef064-a093-4fad-9a06-120ddaa9e87c #12ADFC 1729579120 Acknowledged`
- `Increase 4 da0ef064-a093-4fad-9a06-120ddaa9e87c #12ADFC 1729579120 Planned`

## Rules

- The first command must be a `START` command
- The last command must be a `STOP` command
- There can be only one `START` and one `STOP` command
