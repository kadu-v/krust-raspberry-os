
PAGE_SIZE = 64K;
PAGE_MASK = PAGE_SIZE - 1;

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
    . = __rpi_phys_dram_start_addr;
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

    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")


    /**********************************************************************************************
    * Code + R0 Data + Global Offset Table
    ***********************************************************************************************/
    __code_start = .;
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

    . = ALIGN(PAGE_SIZE);
    __code_end_exclusive = .;

    /**********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/

    .data : { *(.data*) } :segment_data

    
    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}
