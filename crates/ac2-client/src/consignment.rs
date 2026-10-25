//! The consignment calls (`gmCWInterface`'s consignment IFuns, on the player's script): enter and
//! leave the consignment, and the charge the consignment takes. The consignment's answers are the
//! `WM_Store` forwarders (`ViewConsignment`, `UpdateConsignment`) the script raises as notices.

use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::gameplay::Command;
use crate::scripts::*;

impl Client {
    /// The consignment commands: their IFuns' stubs send the request (`gmCWInterface::RequestEnterConsignment`
    /// and `RequestLeaveConsignment` pack no values).
    pub(crate) fn consignment_command(&mut self, command: &Command) -> Vec<SessionEvent> {
        match command {
            Command::EnterConsignment => self.store_ifun(IFUN_REQUEST_ENTER_CONSIGNMENT, &[]),
            Command::LeaveConsignment => self.store_ifun(IFUN_REQUEST_LEAVE_CONSIGNMENT, &[]),
            _ => Vec::new(),
        }
    }

    /// `gmCWInterface::GetConsignmentCharge`: the charge the script returns, a float (its out
    /// value's bits). `None` without the VM.
    pub fn consignment_charge(&mut self) -> Option<f32> {
        let result = self.script_query(
            IFUN_GET_CONSIGNMENT_CHARGE,
            &[IFunArg::Out(OutValue::Word(0))],
            IFunReturn::Word,
        )?;
        Some(f32::from_bits(result.out_word(0)))
    }
}
