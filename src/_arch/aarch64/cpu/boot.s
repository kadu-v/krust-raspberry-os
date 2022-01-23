
// 相対アドレスを計算
// 現在の命令からPCを計算する
// adrp xr, label: 4KB単位(Page)でPCを取得する (e.g. 0x4050 -> 0x4000)
//                 下位12bitを０にして加算     (e.g. label = 0x1000 -> 0x4000 + 0x1000 -> 0x5000)

// adrp と add を用いて、$KB単位の(Page)の完全な相対アドレスを計算する
// adrp xr, label              : 上位12ビットを加算
// add  xr, xr, :lo12:label    : 下位12ビットを加算

.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

.equ _EL2, 0x08
.equ _core_id_mask, 0b11

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------

.section .text._start // 新しいセクション .text._start を定義

//-------------------------------------------------------------------------------------------------
// fn _start()
//-------------------------------------------------------------------------------------------------
_start:
	// Only proceed if the core executes in EL2. Park it otherwise
	mrs 	x0, CurrentEL
	cmp 	x0, _EL2
	b.ne 	.L_parking_loop

	// Only proceed on the boot core. Park it otherwise.
	mrs		x1, MPIDR_EL1
	and		x1, x1, _core_id_mask
	ldr		x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp		x1, x2
	b.ne	.L_parking_loop

	// If execution reaches here, it is the boot core.

	// Initialize DRAM.
	ADR_REL	x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp		x0, x1
	b.eq	.L_prepare_rust
	stp		xzr, xzr, [x0], #16
	b		.L_bss_init_loop

	// Prepare the jump to Rust code.
.L_prepare_rust:
	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive   // 
	mov		sp, x0                                    // sp <- x0

	// Jump to Rust code.
	b		_start_rust

	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b		.L_parking_loop


.size	_start, . - _start
.type	_start, function
.global	_start
