//! The chat shell's script commands (`CWInterface::DoCmdInterp`): a typed `/command` a script
//! registered with `AddShellCmd` runs that script function on the player's process, with the text
//! after the command word and the line's tokens (`CWInterface::HandleCmdArgs`: a `wpstring` and
//! an `RList` of `wpstring`s, the command word first). Written against [`ScriptPort`].

use ac2_weenie::SessionEvent;

use super::ifun::{IFunArg, IFunReturn};
use super::port::ScriptPort;
use super::queries::ScriptQueries;
use super::ui_calls::ifun_into;
use super::ScriptHost;
use crate::native::{ntype, NativeObject};

impl ScriptHost {
    /// The function a script registered for the shell command word `word`
    /// (`CWInterface::m_cmdFuncMap`, matched in lower case).
    pub fn shell_command(&self, word: &str) -> Option<u32> {
        let wanted: Vec<u16> = word.to_lowercase().encode_utf16().collect();
        self.vm
            .shell
            .commands
            .iter()
            .find(|c| c.command == wanted)
            .map(|c| c.function)
    }

    /// The words a script registered help for that reach the shell's help table
    /// (`ShellCmdTable::AddHelpHandler` refuses a word with a space), with their functions.
    /// A word the native shell already has help for keeps the native one (`AddHelpHandler`
    /// adds only a word not yet in `helpHash`); the caller leaves those out.
    pub fn shell_help_words(&self) -> Vec<(String, u32)> {
        self.vm
            .shell
            .help
            .iter()
            .filter(|c| !c.command.contains(&0x20))
            .map(|c| (String::from_utf16_lossy(&c.command), c.function))
            .collect()
    }

    /// The help function a script registered for `word` (`CWInterface::m_helpFuncMap`, lower case).
    pub fn shell_help(&self, word: &str) -> Option<u32> {
        let wanted = word.to_lowercase();
        self.shell_help_words()
            .into_iter()
            .find(|(w, _)| *w == wanted)
            .map(|(_, f)| f)
    }
}

/// `CWInterface::DoHelpInterp(list, cmd, &text)`: runs the help `function` on the player's
/// process (the return slot an int 0, then `list`, then the instance) and reads the return slot
/// back as a `wpstring`: `Some(text)` is `Success_ShellHelpStatus`, `None` `Failure` (no string,
/// or the call did not end).
pub fn help_text<P: ScriptQueries + ?Sized>(
    port: &mut P,
    function: u32,
    list: bool,
) -> Option<String> {
    port.query(function, &[IFunArg::Int(u32::from(list))], IFunReturn::Text)
        .ok()?
        .ret_text
}

/// The wide string native `HandleCmdArgs` makes for one token.
fn wide(text: &str) -> NativeObject {
    NativeObject::WString(text.encode_utf16().collect())
}

/// Runs the script commands among `events` (`ShellCommand`s the session did not run natively):
/// each whose command word a script registered runs its function on the player's process
/// (`DoCmdInterp`), and the events it raised replace the command; every other event stays.
pub fn run_script_commands<P: ScriptPort + ?Sized>(
    port: &mut P,
    events: Vec<SessionEvent>,
) -> Vec<SessionEvent> {
    let mut out = Vec::with_capacity(events.len());
    for event in events {
        let SessionEvent::ShellCommand { text, args, .. } = &event else {
            out.push(event);
            continue;
        };
        let function = args
            .first()
            .and_then(|word| port.script_host().and_then(|host| host.shell_command(word)));
        let Some(function) = function else {
            out.push(event);
            continue;
        };
        // The arguments as `HandleCmdArgs` pushes them: the text first, then the token list.
        let call = [
            IFunArg::Text(text.clone()),
            IFunArg::VmList(ntype::WPSTRING, args.iter().map(|a| wide(a)).collect()),
        ];
        if let Err(error) = ifun_into(port, function, &call, &mut out) {
            if let Some(scripts) = port.script_state() {
                scripts
                    .problems
                    .push(format!("shell command {function:#010x}: {error}"));
            }
        }
    }
    out
}
