/* 物理アドレスの開始アドレス */
__rpi_phys_dram_start_addr = 0;

/* raspberrypiのプログラムの開始アドレス */
__rpi_phys_binary_load_addr = 0x80000;

ENTRY(__rpi_phys_binary_load_addr)

/*
    FLAGS:
        4 == R
        5 == RX
        6 == RW
*/

PHDRS
{   
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code            PT_LOAD FLAGS(5);
    segment_data            PT_LOAD FLAGS(6);
}


SECTIONS {
    . = __rpi_phys_binary_load_addr;
    /**********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack (NOLOAD) : {
                                                /* ^            */
                                                /* | stack      */
        . += __rpi_phys_binary_load_addr;       /* | growth     */
                                                /* | direction  */
        __boot_core_stack_end_exclusive = .;    /* |            */
    } :segment_boot_core_stack


    /**********************************************************************************************
    * Code + R0 Data + Global Offset Table
    ***********************************************************************************************/
    .text : {
        KEEP(*(.text._start))
        *(.text._start_arguments)   /* _start 関数の引数        */
        *(.text._start_rust)        /* Rustのエントリーポイント   */ 
        *(.text*)                   /* その他全てのtextセクション */
    } :segment_code

    .rodata : ALIGN(8) {
        *(.rodata*)
    } :segment_code

    .got : ALIGN(8) {
        *(.got)
    } :segment_code

    /**********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/

    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}
