//! The consignment calls on the player's script (`gmCWInterface`'s consignment IFuns): enter and
//! leave the consignment, and the charge the consignment takes. The consignment's answers are the
//! `WM_Store` forwarders (`ViewConsignment`, `UpdateConsignment`) the script raises as notices.

use ac2_weenie::SessionEvent;

use super::ifun::{IFunArg, IFunReturn, OutValue};
use super::ifun_ids::{
    IFUN_GET_CONSIGNMENT_CHARGE, IFUN_REQUEST_ENTER_CONSIGNMENT, IFUN_REQUEST_LEAVE_CONSIGNMENT,
};
use super::port::ScriptPort;
use super::ui_calls;

/// The consignment calls, on any [`ScriptPort`] (the client implements the port).
pub trait ConsignmentQueries: ScriptPort {
    /// `gmCWInterface::RequestEnterConsignment`: the request's IFun (no values packed); the
    /// events it raised come back.
    fn enter_consignment(&mut self) -> Vec<SessionEvent> {
        ui_calls::script_request(self, IFUN_REQUEST_ENTER_CONSIGNMENT, &[])
    }

    /// `gmCWInterface::RequestLeaveConsignment`: the request's IFun (no values packed).
    fn leave_consignment(&mut self) -> Vec<SessionEvent> {
        ui_calls::script_request(self, IFUN_REQUEST_LEAVE_CONSIGNMENT, &[])
    }

    /// `gmCWInterface::GetConsignmentCharge`: the charge the script returns, a float (its out
    /// value's bits). `None` without the VM.
    fn consignment_charge(&mut self) -> Option<f32> {
        let result = ui_calls::script_query(
            self,
            IFUN_GET_CONSIGNMENT_CHARGE,
            &[IFunArg::Out(OutValue::Word(0))],
            IFunReturn::Word,
        )?;
        Some(f32::from_bits(result.out_word(0)))
    }
}

impl<P: ScriptPort + ?Sized> ConsignmentQueries for P {}
