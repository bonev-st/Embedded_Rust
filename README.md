# Embedded Rust labs (VS Code + dev container)

One Docker dev container for all Rust labs on the STM32L562E-DK (and the course's other Cortex-M boards). The on-board **STLINK-V3E** is passed through to the container, and **probe-rs** in the container flashes and debugs the board and shows **defmt/RTT** logs.

```text
Windows ── usbipd-win ──► WSL2 Ubuntu-22.04 ── Docker Engine ──► dev container
 (VS Code UI)              /dev/bus/usb  ─────────────────────► probe-rs ──SWD──► STM32L562
```

The full background is in `../DevContainer_STM32L562E-DK.md`.

## Layout

```text
dev_dd/
├── .devcontainer/          one container for all labs
├── scripts/                ST-LINK attach (Windows), TZEN tools, Claude restore
├── workspaces/             one VS Code workspace per lab (F5/tasks of that lab only)
├── labs.code-workspace     all labs at once, for browsing
├── LICENSE                 MIT
├── l562-rtt-demo/          one folder per lab, each a standalone Cargo project
├── hello_world/
└── 100-exercises-to-learn-rust/   Mainmatter's Rust course (git subtree), see below
```

Each lab has its own `.cargo/config.toml` (target, runner/chip), `memory.x`, `.vscode/launch.json` and `README.md`, so labs for different chips can live side by side. They are deliberately **not** one Cargo workspace.

| Lab | Board | What it shows |
| --- | --- | --- |
| [l562-rtt-demo](l562-rtt-demo/README.md) | STM32L562E-DK | GPIO, button, defmt/RTT logging, panic handling, debugging |
| [hello_world](hello_world) | PC (in the container) | Lab 1: first Rust program |
| [100-exercises-to-learn-rust](100-exercises-to-learn-rust) | PC (in the container) | Lab 2: [100 Exercises To Learn Rust](https://rust-exercises.com/100-exercises/), see [below](#100-exercises-to-learn-rust) |

## 100 Exercises To Learn Rust

Mainmatter's course ([book](https://rust-exercises.com/100-exercises/01_intro/00_welcome), [repo](https://github.com/mainmatter/100-exercises-to-learn-rust)). It runs on the PC, not on the board. Everything it needs is in the container: stable Rust, `rust-analyzer` and the workshop runner `wr`.

**Working on it**

```bash
cd 100-exercises-to-learn-rust
wr          # checks your solutions and opens the next exercise
```

`wr` needs the `.wr.toml` in the `dev_dd` root: it looks for its config in the git root, and that file points it at `100-exercises-to-learn-rust/exercises`. Run `wr` from inside `100-exercises-to-learn-rust/` (not from a lab folder, whose `.cargo/config.toml` would switch to the embedded target).

Each exercise is a package under `exercises/`. The instructions are in its `src/lib.rs`, the tests next to them. `wr` keeps its progress in `exercises/progress.db`, which is not in git (upstream ignores it); it re-checks your solutions when it runs. Commit your solutions as usual in this repo and `git push` them to `origin`.

**How it is wired into git:** the course is a *git subtree*. Its files are ordinary files in this repo (no nested repo, no submodule), and Mainmatter's repo is the remote `exercises-upstream`. It was added with:

```bash
git remote add exercises-upstream https://github.com/mainmatter/100-exercises-to-learn-rust.git
git subtree add --prefix=100-exercises-to-learn-rust exercises-upstream main --squash
```

**Getting Mainmatter's updates** (from the `dev_dd` root, with a clean working tree):

```bash
git subtree pull --prefix=100-exercises-to-learn-rust exercises-upstream main --squash
```

This merges their changes into your solutions like a normal `git merge`; resolve conflicts if both changed the same lines. After a fresh clone of this repo, add the remote again first (`git remote add …` as above).

**Peeking at the official solutions** (branch `solutions` upstream):

```bash
git fetch exercises-upstream solutions
git show exercises-upstream/solutions:exercises/01_intro/00_welcome/src/lib.rs
```

## Prerequisites (one-time)

1. **Docker Engine in WSL Ubuntu-22.04** (see guide step 3): enable systemd in `/etc/wsl.conf`, then run `curl -fsSL https://get.docker.com | sh` and `sudo usermod -aG docker $USER`.
2. **VS Code extensions on Windows:** *WSL* and *Dev Containers*.
3. **usbipd-win:** already installed (5.3).

## Setup

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

### 4. Open a lab's workspace

In the container: **File → Open Workspace from File… → `workspaces/<lab>.code-workspace`**, for example `workspaces/l562-rtt-demo.code-workspace`. You see the `dev_dd` root (scripts, READMEs) and that one lab; `F5` and **Run Task** offer only that lab's configurations, and rust-analyzer loads only that lab.

Switching labs: open another file from `workspaces/` (or **File → Open Recent**). VS Code reloads the window but stays in the same container. The `dev_dd` root is the first folder in every workspace, so Claude Code keeps one chat history for all labs.

`labs.code-workspace` opens all labs at once, which is handy for browsing and searching across labs, but then `F5` and **Run Task** list the configurations of every lab. (If you open only the `dev_dd` folder, `cargo` still works from a lab's terminal, but `F5` finds no configurations.)

### 5. Check the probe

```bash
probe-rs list        # [0]: STLINK-V3 -- 0483:374e:...
```

Then continue in the lab's README, for example [l562-rtt-demo](l562-rtt-demo/README.md).

## Adding a new lab

1. Copy an existing lab: `cp -r l562-rtt-demo <new-lab>` (skip its `target/` folder if there is one).
2. In the copy, change `name` (package and `[[bin]]`) in `Cargo.toml`, and the binary path `.../debug/<name>` and `.../release/<name>` in `.vscode/launch.json`.
3. For another chip, change `target` and the `--chip` of the runner in `.cargo/config.toml`, `memory.x`, `Embed.toml`, `chip` and `svdFile` in `launch.json`, and the PAC/HAL crate in `Cargo.toml`. The container already has the `thumbv8m.main-none-eabihf`, `thumbv7em-none-eabihf` and `thumbv7m-none-eabi` targets.
4. Give it a workspace: `cp workspaces/l562-rtt-demo.code-workspace workspaces/<new-lab>.code-workspace` and replace `l562-rtt-demo` and the display name in the copy (for a PC lab also set `rust-analyzer.check.allTargets` to `true`). Add the folder to `folders` and `files.exclude` in `labs.code-workspace` too, and a row to the table above.
5. Run `cargo` commands **from inside the lab folder**, since that is where its `.cargo/config.toml` is picked up.

## Container files

| File | Purpose |
| --- | --- |
| `.devcontainer/Dockerfile` | Ubuntu 24.04, Rust stable, Cortex-M targets (several side by side), probe-rs, cargo-binutils, workshop-runner (`wr`), gdb-multiarch, picocom, OpenOCD. |
| `.devcontainer/devcontainer.json` | USB passthrough (`--privileged`, `/dev/bus/usb`), VS Code extensions installed on every rebuild, volumes for the cargo cache and Claude Code data. |
| `scripts/attach-stlink.ps1` | Finds the STLINK-V3E and binds/attaches it with usbipd (run on Windows). |
| `scripts/restore-claude.sh` | Restores Claude Code chats from `.claude-backup/` into a fresh volume after a rebuild. |
| `scripts/clear-tzen.tcl` | OpenOCD attempt at the RDP 1 → 0 regression with `TZEN=0`. **Did not work on this board**, see below. |

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `probe-rs list` is empty | Run `attach-stlink.ps1` again and check `lsusb` in WSL. Rebuild the container if `devcontainer.json` changed. |
| `Probe firmware is outdated` | `usbipd detach --busid <id>`, run `C:\ST\STM32CubeCLT_1.18.0\STLinkUpgrade.bat`, then attach again. |
| No RTT output in VS Code | Make sure `rttEnabled` is true and the binary was built from this source. With `haltAfterReset: true`, press Continue. |
| `program_page failed with code 1` at `0x08000000`, or a SecureFault | TrustZone is on (ST's demo enables it), see [TrustZone (TZEN)](#trustzone-tzen). |
| `F5` shows no configurations | Open the lab's `workspaces/<lab>.code-workspace` instead of the plain folder. |
| Windows COM port or CubeProgrammer lost the board | Expected while it is attached to WSL. Use `usbipd detach --busid <id>`. |

## TrustZone (TZEN)

ST's out-of-box demo ships with TrustZone enabled (`TZEN=1`). The labs are built for a non-secure-only chip, so flashing fails until TZEN is cleared.

Check the state (read-only):

```bash
openocd -f interface/stlink.cfg -f target/stm32l5x.cfg \
  -c init -c "stm32l4x trustzone 0" -c shutdown
```

`TZEN = 1 : TrustZone enabled by option bytes` means it has to be cleared. `probe-rs read --chip STM32L562QE b32 0x40022040 1` shows the same: bit 31 set means `TZEN=1`.

What we learned clearing it on this board:

- TZEN can only be cleared during an RDP regression (level 1 → 0), which **mass-erases the whole flash**. OpenOCD's `stm32l4x trustzone 0 disable` refuses to do this on its own.
- At RDP level 1 with `TZEN=1`, SWD only works while the CPU is in the non-secure state. With an empty or invalid flash the core locks up in the secure world and the debugger cannot reach the option bytes. **Do not raise RDP to level 1 unless you know the regression path works.**
- Booting with **BOOT0 high** (system bootloader) avoids the lockup; ST describes the procedure in [How to disable TrustZone in STM32L5xx devices](https://wiki.st.com/stm32mcu/wiki/Security:How_to_disable_TrustZone_in_STM32L5xx_devices_during_development_phase).

<!-- TODO: add the exact steps that finally cleared TZEN on this board -->

## License

[MIT](LICENSE), covering all labs and scripts in this repository.

Exception: `.vscode/STM32L562.svd` in the labs is copied from STMicroelectronics' STM32CubeCLT and stays under ST's own license terms.
