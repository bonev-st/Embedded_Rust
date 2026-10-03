/* STM32L562QE, TrustZone disabled (TZEN = 0, factory default) */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K   /* SRAM1 192K + SRAM2 64K, contiguous */
}
