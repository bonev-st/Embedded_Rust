# Lab 00: STM32L562E-DK RTT demo

A first test project for the STM32L562E-DK: GPIO, the USER button, **defmt/RTT** logging and a panic, flashed and debugged with **probe-rs** from the dev container. Container setup and probe forwarding are in the [top-level README](../README.md).

## What the demo does

| Board part | Behaviour |
| --- | --- |
| Red LED LD9 (PD3) | Heartbeat, toggles every 500 ms. Logged at `trace` level. |
| Green LED LD10 (PG12) | On while the button is held. Shows the VDDIO2/`PWR_CR2.IOSV` setup. |
| USER button B2 (PC13) | Each press and release is logged. Every 5th press logs a `warn`. **Holding it for 3 s triggers a demo panic.** |
| RTT log | Timestamped (ms) lines at all defmt levels, a `#[derive(Format)]` struct every second, an array and a register value in hex. |

Expected output of `cargo run`:

```text
      Erasing ✔ ...
  Programming ✔ ...
==== STM32L562E-DK RTT demo ====
0.000 INFO  core clock 4000000 Hz (MSI reset default)
0.000 INFO  GPIO ready: LD9=PD3 (heartbeat), LD10=PG12 (button), B2=PC13
1.000 INFO  Status { uptime_ms: 1000, presses: 0, button: Released, heartbeat_on: false }
1.000 DEBUG GPIOD ODR = 0x00000008, PD3 high = true
2.315 INFO  button pressed (#1)
2.315 DEBUG last presses at [0, 0, 0, 2315] ms
2.480 DEBUG button released after 165 ms
```

## Run the demo

Run `cargo` from this folder (`cd l562-rtt-demo`), so `.cargo/config.toml` is used.

- **Terminal:** `cargo run` builds, flashes and streams the RTT log. Stop it with Ctrl+C. `cargo embed` gives the same in a terminal UI.
- **Debugger:** with `workspaces/l562-rtt-demo.code-workspace` open, press `F5` and pick **L562: Flash & Debug (probe-rs + RTT)**. It stops at reset; press Continue. The RTT channel opens as a terminal tab in VS Code. Breakpoints, stepping and the peripheral view (from `.vscode/STM32L562.svd`) all work. Stop any `cargo run` first, only one tool can use the probe.
- **More log detail:** set `DEFMT_LOG = "trace"` in `.cargo/config.toml` to see the heartbeat lines. The filter is applied when the firmware is compiled, so rebuild afterwards.
- **Panic demo:** hold B2 for 3 s. probe-rs prints the panic message and a backtrace, then the core stays halted. Reset or re-flash to continue.

## Files

| File | Purpose |
| --- | --- |
| `.cargo/config.toml` | Default target `thumbv8m.main-none-eabihf`, `probe-rs run` runner, linker scripts, `DEFMT_LOG`. |
| `memory.x` | 512K flash @ `0x08000000`, 256K RAM @ `0x20000000` (TrustZone off). |
| `build.rs` | Copies `memory.x` where the linker finds it. |
| `Embed.toml` | `cargo embed` settings (RTT on). |
| `rust-toolchain.toml` | Stable toolchain with the Cortex-M33 target. |
| `.vscode/launch.json` | probe-rs: debug, release debug, attach. All with the RTT defmt channel. |
| `.vscode/tasks.json` | Build, run, flash, list probes, erase, VCP terminal, size. |
| `.vscode/STM32L562.svd` | Copied from STM32CubeCLT 1.18 (`STMicroelectronics_CMSIS_SVD`). |

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| Green LED never lights | Usually `PWR_CR2.IOSV` isn't set. It is in this demo; check the board's VDDIO2 jumpers if it still fails. |
| Flashing fails at `0x08000000` | TrustZone is probably on, see [TrustZone (TZEN)](../README.md#trustzone-tzen). |
| Breakpoint stays hollow, hover says *"Cannot set breakpoint here… reduce optimization"* | The debug build used `opt-level = 1`, which removed the breakpoint locations from more than half of the lines. `Cargo.toml` now builds this crate with `opt-level = 0` and only the dependencies with `opt-level = "s"` (`[profile.dev.package."*"]`), so every code line takes a breakpoint. |
| More than 8 breakpoints, the extra ones stay hollow | The Cortex-M33 has 8 hardware breakpoints. Remove some. |
| `SwdApFault` / `UNWIND: Error while checking for exception context` at the first stop | Harmless. probe-rs tries to unwind the stack at the reset halt (`haltAfterReset: true`). Press Continue. |
| Breakpoints stop working after renaming the lab folder | The binary still points to the old source path. Run `cargo clean` once. |

Probe and container problems are in the [top-level troubleshooting](../README.md#troubleshooting).
