<#
.SYNOPSIS
  Forwards the STM32L562E-DK on-board STLINK-V3E (USB 0483:374E / 0483:374F) to WSL with usbipd-win,
  so the dev container can use it through /dev/bus/usb.

.DESCRIPTION
  Run in PowerShell as Administrator (binding needs admin rights once per device).
  With -AutoAttach the script keeps running and re-attaches after a replug or ST-LINK reset.
  To give the ST-LINK back to Windows (STM32CubeProgrammer, COM port): usbipd detach --busid <BUSID>

.EXAMPLE
  .\attach-stlink.ps1 -AutoAttach
#>
#Requires -RunAsAdministrator
param(
    [switch]$AutoAttach,
    [switch]$Force   # use when usbipd warns about an incompatible USB filter driver
)

$line = usbipd list | Select-String -Pattern '0483:374[eEfF]' | Select-Object -First 1
if (-not $line) {
    Write-Error "STLINK-V3E (0483:374E/374F) not found. Is the board's CN17 (STLK) USB port connected?"
    exit 1
}

$text = $line.Line.Trim()
$busId = ($text -split '\s+')[0]
Write-Host "Found ST-LINK on BUSID $busId -> $text"

if ($text -match 'Not shared') {
    Write-Host "Binding $busId (one-time) ..."
    if ($Force) { usbipd bind --busid $busId --force } else { usbipd bind --busid $busId }
    if ($LASTEXITCODE -ne 0) {
        Write-Error "usbipd bind failed. If it warned about a USB filter, re-run with -Force."
        exit 1
    }
}

if ($text -match 'Attached') {
    Write-Host "Already attached to WSL."
    exit 0
}

if ($AutoAttach) {
    Write-Host "Attaching to WSL with auto-attach (keep this window open, Ctrl+C to stop) ..."
    usbipd attach --wsl --busid $busId --auto-attach
} else {
    usbipd attach --wsl --busid $busId
    if ($LASTEXITCODE -eq 0) { Write-Host "Attached. Check in WSL with: lsusb | grep -i st" }
}
