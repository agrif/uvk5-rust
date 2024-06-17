MEMORY
{
    /* NOTE 1 K = 1 KiB = 1024 bytes */
    /* constructing the Header relies on the origin at 0 */
    /* limit the length to 2K as a sanity check */
    RAM   (xrw) : ORIGIN = 0x0000, LENGTH = 2K
}
