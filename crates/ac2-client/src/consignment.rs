//! The consignment commands: the client's `Command`s become the consignment's requests on the
//! player's script (`ac2_script::host::consignment`), and the charge is read back from it.

use ac2_script::host::consignment::ConsignmentQueries;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::gameplay::Command;

impl Client {
    /// The consignment commands: their IFuns' stubs send the request (`gmCWInterface::RequestEnterConsignment`
    /// and `RequestLeaveConsignment` pack no values).
    pub(crate) fn consignment_command(&mut self, command: &Command) -> Vec<SessionEvent> {
        match command {
            Command::EnterConsignment => self.enter_consignment(),
            Command::LeaveConsignment => self.leave_consignment(),
            _ => Vec::new(),
        }
    }

    /// `gmCWInterface::GetConsignmentCharge`: the charge the script returns, a float (its out
    /// value's bits). `None` without the VM.
    pub fn consignment_charge(&mut self) -> Option<f32> {
        ConsignmentQueries::consignment_charge(self)
    }
}
