# RTGB Controller

RTGB Controller is responsible for scheduling command to controller an active fermentations

## Overview

1. Receives command from the RTGB API that includes the fermentation steps to send to a chamber
2. Store in a DB all the commands to send like Command Type <Start|Increase|Decrease|Stop|> Value <Float> Target <Cooling|Heating HardwareID> Date <timestamp> Status<Planned|Sent|Acknowledged>
3. Every minute check the DB, fire the command that needs to be sent to the hardware

### Command description

- Type

  - Start: Start the fermentation at the given `Value` in degree Celcius. e.g. Start 22
  - Increase: Increase the temperature of the given `Value` in degree Celcius. e.g. Increase 1.5
  - Decrease: Decrease the temperature of the given `Value` in degree Celcius. e.g. Increase 1.5
  - Stop: Stop the fermentation at the given `Value`. e.g. Stop 20

- Value: A temperature value, can represent a temperature in Celcius or an absolute delta
- Target: The cooling or heating hardware identifier
- Date: When to fire the command
- Status
  - Planned: The command sent at `Date`
  - Sent: The command has been sent at `Date`
  - Acknowledged: The command has been received by the hardware
