# Split CheckTetherSupplyCommand

## Files

* src/types/command/check_tether_supply_command.rs

## Tasks

* Implement `TetherSupplyCommand`
  * Should have a `command: TetherSupplySubCommand` field
  * Should delegate execution to `command`, similar to src/types/cli.rs
* Implement `TetherSupplySubCommand`
  * Should have
  * Should split the execution into `TetherSupplyCheckCommand` and `TetherSupplyPrintCommand`, similar to src/types/command.rs
