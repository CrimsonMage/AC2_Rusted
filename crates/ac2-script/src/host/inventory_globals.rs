//! `gInventoryGlobals`: the `InventoryGlobals` singleton's coin rule, which the money bag asks
//! for the most coins a stack holds (`gInventoryGlobals::CallIFun_GetCoinMaxStackSize`).

use super::chargen::call_system;
use super::ScriptHost;
use crate::vm::InterpStatus;

const SYSTEM: &str = "InventoryGlobals";
/// `gInventoryGlobals::CallIFun_GetCoinMaxStackSize`'s function (`0xA60001`).
const GET_COIN_MAX_STACK_SIZE: u32 = 0x00A6_0001;

impl ScriptHost {
    /// `gInventoryGlobals::CallIFun_GetCoinMaxStackSize`: a zero return slot, then the singleton
    /// as the instance; the script's return is the maximum (`None` when the library has no such
    /// singleton, or the call stops for data).
    pub fn coin_max_stack_size(&mut self) -> Option<i32> {
        let call = call_system(&mut self.vm, SYSTEM, GET_COIN_MAX_STACK_SIZE, None, &[])?;
        (call.status == InterpStatus::Terminated).then_some(call.retval as i32)
    }
}
