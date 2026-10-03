# OpenOCD script: RDP regression 1 -> 0 with TZEN=0 on STM32L5 (MASS-ERASES FLASH).
# Uses the non-secure FLASH registers (0x4002_2000), which a debugger may still reach
# at RDP level 1, where secure debug access is blocked. The core is never halted.
#
# Preconditions:
#   - RDP level 1 (0xDC). From level 0 set it with:
#       openocd -f interface/stlink.cfg -f target/stm32l5x.cfg -c init -c "reset halt" \
#         -c "stm32l4x option_write 0 0x40 0xDC 0xFF" -c "stm32l4x option_load 0" -c shutdown
#   - The core must not be in lockup. With an empty/invalid flash, boot with BOOT0 high
#     (system bootloader) and power-cycle first.
# Run:
#   openocd -f interface/stlink-dap.cfg -c "transport select dapdirect_swd" \
#     -f target/stm32l5x.cfg -c "stm32l5x.cpu configure -defer-examine" -f scripts/clear-tzen.tcl

init
stm32l5x.cpu arp_examine
# non-secure bus accesses (CSW HNONSEC)
stm32l5x.dap apcsw 0x40000000 0x40000000

set NSKEYR  0x40022008
set OPTKEYR 0x40022010
set NSSR    0x40022020
set NSCR    0x40022028
set OPTR    0x40022040

proc rd {addr} { return [read_memory $addr 32 1] }
proc wr {addr value} { write_memory $addr 32 [list $value] }

set optr [rd $OPTR]
echo [format "OPTR before: 0x%08x (RDP=0x%02x TZEN=%d)" $optr [expr {$optr & 0xFF}] [expr {($optr >> 31) & 1}]]

if {($optr & 0xFF) == 0xAA || ($optr & 0xFF) == 0xCC} {
    echo "RDP is not level 1 - set RDP=0xDC first. Nothing written."
    shutdown
    return
}

wr $NSKEYR 0x45670123
wr $NSKEYR 0xCDEF89AB
wr $OPTKEYR 0x08192A3B
wr $OPTKEYR 0x4C5D6E7F
echo [format "NSCR unlocked: 0x%08x" [rd $NSCR]]

# RDP=0xAA (level 0) and TZEN=0 in one write -> regression + mass erase
wr $OPTR [expr {($optr & ~0x800000FF) | 0xAA}]
wr $NSCR [expr {[rd $NSCR] | (1 << 17)}]   ;# OPTSTRT

echo "Regression running (mass erase)..."
set t 0
while {([rd $NSSR] & (1 << 16)) && $t < 300} { sleep 100; incr t }
echo [format "NSSR after: 0x%08x" [rd $NSSR]]

# OBL_LAUNCH reloads the option bytes and resets the chip, so the debug link may drop
catch { wr $NSCR [expr {[rd $NSCR] | (1 << 27)}] }
echo "Option bytes loaded. Power-cycle the board, then check TZEN."
shutdown
