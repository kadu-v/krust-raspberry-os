use cortex_a::registers::*;
use tock_registers::interfaces::Readable;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

trait DaifField {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

struct Debug;
struct SError;
struct IRQ;
struct FIQ;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

// DAIF Register : daif という名称のレジスタを使用してD,A,I,Fビットを操作し，各種例外および割込みの禁止/許可を制御する
impl DaifField for Debug {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// #[rustfmt::skip]
pub fn print_state() {
    use crate::info;

    let to_mask_str = |x| -> _ {
        if x {
            "Masked"
        } else {
            "Unmasked"
        }
    };

    info!("    Debug : {}", to_mask_str(is_masked::<Debug>()));
    info!("    SError: {}", to_mask_str(is_masked::<SError>()));
    info!("    IRQ   : {}", to_mask_str(is_masked::<IRQ>()));
    info!("    FIQ   : {}", to_mask_str(is_masked::<FIQ>()));
}
