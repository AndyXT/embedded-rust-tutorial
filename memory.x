/* Memory layout for ARM Cortex-R5 target */
MEMORY
{
  /* Typical Cortex-R5 memory layout - adjust for specific chip */
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K
  RAM : ORIGIN = 0x08000000, LENGTH = 128K
  
  /* Additional regions for Cortex-R5 */
  TCM : ORIGIN = 0x00800000, LENGTH = 64K  /* Tightly Coupled Memory */
}