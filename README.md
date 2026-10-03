# STM32L562E-DK RTT demo (VS Code + dev container)

This is a test project for the STM32L562E-DK. The on-board **STLINK-V3E** is passed through to a Docker dev container, and **probe-rs** in the container flashes and debugs the board and shows **defmt/RTT** logs.

```text
Windows ── usbipd-win ──► WSL2 Ubuntu-22.04 ── Docker Engine ──► dev container
 (VS Code UI)              /dev/bus/usb  ─────────────────────► probe-rs ──SWD──► STM32L562
```

The full background is in `../DevContainer_STM32L562E-DK.md`.

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

## Prerequisites (one-time)

1. **Docker Engine in WSL Ubuntu-22.04** (see guide step 3): enable systemd in `/etc/wsl.conf`, then run `curl -fsSL https://get.docker.com | sh` and `sudo usermod -aG docker $USER`.
2. **VS Code extensions on Windows:** *WSL* and *Dev Containers*.
3. **usbipd-win:** already installed (5.3).

## Steps

### 1. Copy the project into the WSL filesystem

Cargo builds are much faster there than on `/mnt/c`. In the Ubuntu-22.04 terminal:

```bash
mkdir -p ~/work
cp -r /mnt/c/Work/EmbeddedRust/dev_dd ~/work/
cd ~/work/dev_dd && code .
```

### 2. Forward the ST-LINK to WSL

Connect the board's **CN17 (STLK)** USB port. Then, in **PowerShell as Administrator**:

```powershell
cd C:\Work\EmbeddedRust\dev_dd\scripts
.\attach-stlink.ps1 -AutoAttach      # add -Force if usbipd warns about the 'hhddmsusb' USB filter
```

If PowerShell refuses to run the script, use `powershell -ExecutionPolicy Bypass -File .\attach-stlink.ps1 -AutoAttach`. Check it in Ubuntu with `lsusb | grep -i st`, which should show `0483:374e STMicroelectronics STLINK-V3`.

### 3. Open the folder in the container

Press `F1` and run **Dev Containers: Reopen in Container**. The first build takes about 5 to 10 minutes. Attach the board **before** this step if you want `/dev/ttyACM0` (the VCP) inside the container.

### 4. Check the probe

Run the task **probe: list connected probes**, or in the terminal:

```bash
probe-rs list        # [0]: STLINK-V3 -- 0483:374e:...
```

### 5. Run the demo

- **Terminal:** `cargo run` builds, flashes and streams the RTT log. Stop it with Ctrl+C. `cargo embed` gives the same in a terminal UI.
- **Debugger:** press `F5` with **L562: Flash & Debug (probe-rs + RTT)**. It stops at reset; press Continue. The RTT channel opens as a terminal tab in VS Code. Breakpoints, stepping and the peripheral view (from `.vscode/STM32L562.svd`) all work.
- **More log detail:** set `DEFMT_LOG = "trace"` in `.cargo/config.toml` to see the heartbeat lines. The filter is applied when the firmware is compiled, so rebuild afterwards.
- **Panic demo:** hold B2 for 3 s. probe-rs prints the panic message and a backtrace, then the core stays halted. Reset or re-flash to continue.

## Files

| File | Purpose |
| --- | --- |
| `.devcontainer/Dockerfile` | Ubuntu 24.04, Rust stable, Cortex-M targets (several side by side), probe-rs, gdb-multiarch, picocom, OpenOCD. |
| `.devcontainer/devcontainer.json` | USB passthrough (`--privileged`, `/dev/bus/usb`), extensions, cargo cache volumes. |
| `.cargo/config.toml` | Default target `thumbv8m.main-none-eabihf`, `probe-rs run` runner, linker scripts, `DEFMT_LOG`. |
| `memory.x` | 512K flash @ `0x08000000`, 256K RAM @ `0x20000000` (TrustZone off). |
| `Embed.toml` | `cargo embed` settings (RTT on). |
| `.vscode/launch.json` | probe-rs: debug, release debug, attach. All with the RTT defmt channel. |
| `.vscode/tasks.json` | Build, run, flash, list probes, erase, VCP terminal, size. |
| `.vscode/STM32L562.svd` | Copied from STM32CubeCLT 1.18 (`STMicroelectronics_CMSIS_SVD`). |
| `scripts/attach-stlink.ps1` | Finds the STLINK-V3E and binds/attaches it with usbipd. |

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `probe-rs list` is empty | Run `attach-stlink.ps1` again and check `lsusb` in WSL. Rebuild the container if `devcontainer.json` changed. |
| `Probe firmware is outdated` | `usbipd detach --busid <id>`, run `C:\ST\STM32CubeCLT_1.18.0\STLinkUpgrade.bat`, then attach again. |
| No RTT output in VS Code | Make sure `rttEnabled` is true and the binary was built from this source. With `haltAfterReset: true`, press Continue. |
| `program_page failed with code 1` at `0x08000000`, or a SecureFault | TrustZone is on (ST's demo enables it). Check with `probe-rs read --chip STM32L562QE b32 0x40022040 1`: bit 31 set means `TZEN=1`. Clear it from the container with OpenOCD (**mass-erases flash**), see [Clearing TrustZone (TZEN)](#clearing-trustzone-tzen). |
| Green LED never lights | Usually `PWR_CR2.IOSV` isn't set. It is in this demo; check the board's VDDIO2 jumpers if it still fails. |
| Windows COM port or CubeProgrammer lost the board | Expected while it is attached to WSL. Use `usbipd detach --busid <id>`. |

## Clearing TrustZone (TZEN)

ST's out-of-box demo ships with TrustZone enabled (`TZEN=1`). This project is built for a non-secure-only chip, so flashing fails until TZEN is cleared. OpenOCD in the container does this over the ST-LINK.

> **Warning:** TZEN can only be cleared during an RDP regression (level 1 → 0), which **mass-erases the whole flash**.

1. Stop anything else that uses the probe (probe-rs debug session, `cargo run`, `cargo embed`).
2. Check the current state (read-only):

   ```bash
   openocd -f interface/stlink.cfg -f target/stm32l5x.cfg \
     -c init -c "stm32l4x trustzone 0" -c shutdown
   ```

   `TZEN = 1 : TrustZone enabled by option bytes` means it has to be cleared. The warning `The selected adapter does not support debugging this device in secure mode` is expected and harmless here.

3. Clear TZEN (erases flash):

   ```bash
   openocd -f interface/stlink.cfg -f target/stm32l5x.cfg \
     -c init -c "stm32l4x trustzone 0 disable" -c shutdown
   ```

4. Run step 2 again. It should now report `TZEN = 0` and `RDP level 0 (0xAA)`. The same check with probe-rs: `probe-rs read --chip STM32L562QE b32 0x40022040 1` must have bit 31 clear.
5. Flash as usual with `cargo run`.

If OpenOCD can't connect after the regression, power-cycle the board (unplug/replug USB, then run `attach-stlink.ps1` again on Windows).
