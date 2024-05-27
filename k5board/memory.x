/* this memory layout, and all the examples, are for the UV-K5 radio
   currently this is the only known device to use a DP32G030 chip */

MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    /* reserve 0x1000 at end of flash for bootloader */
    FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 60K
    RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 16K
}

/* keep this even if it's not used */
EXTERN(VERSION);
