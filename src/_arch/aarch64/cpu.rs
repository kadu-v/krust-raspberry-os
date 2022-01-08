use cortex_a::asm;

//-------------------------------------------------------------------------------------------------
// Archtectural Public Reexports
//-------------------------------------------------------------------------------------------------

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}