MEMORY
{
    /* NOTE 1 K = 1 KiBi = 1024 bytes */
    /* reserve 0x1000 at end of flash for bootloader */
    FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 60K
    RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 16K
}
