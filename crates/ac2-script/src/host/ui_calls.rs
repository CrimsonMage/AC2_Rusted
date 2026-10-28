//! The UI's entries into the player's script (`ggmCWInterface::CallIFun_*`, `CPlayer`): the
//! IFun on the player's process, its output routed through [`outcome`], and the queries that
//! answer only when the VM owns the call ([`scripts_own`]). Written against [`ScriptPort`].

use ac2_weenie::SessionEvent;

use super::ifun::{IFunArg, IFunResult, IFunReturn};
use super::outcome::outcome;
use super::port::ScriptPort;

/// Whether the UI's IFun (or server call) `function` is the script VM's: it was handed to the VM
/// and the player's process runs.
pub fn scripts_own<P: ScriptPort + ?Sized>(port: &mut P, function: u32) -> bool {
    let Some(player) = port.player_id() else {
        return false;
    };
    port.script_state()
        .is_some_and(|s| s.owned.contains(&function) && s.running.contains(&player))
}

/// `ggmCWInterface::CallIFun_*`: the IFun on the player's process (a return slot shaped by `ret`,
/// the arguments, the instance, `VM::CallIFun`), its output routed into `out` as a call of the VM,
/// and the result returned.
pub fn ifun_return<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    args: &[IFunArg],
    ret: IFunReturn,
    out: &mut Vec<SessionEvent>,
) -> Result<IFunResult, String> {
    let player = port.player_id().ok_or("no player")?;
    let scripts = port.script_state().ok_or("no script VM")?;
    let result = scripts
        .host
        .query(player, function, args, ret)
        .map_err(|e| format!("{e:?}"));
    let outcome_result = result
        .as_ref()
        .map(|r| r.outcome.clone())
        .map_err(Clone::clone);
    outcome(port, player, None, outcome_result, out);
    result
}

/// [`ifun_return`] with the return slot as one word.
pub fn ifun_into<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    args: &[IFunArg],
    out: &mut Vec<SessionEvent>,
) -> Result<IFunResult, String> {
    ifun_return(port, function, args, IFunReturn::Word, out)
}

/// A UI call into the player's script: its server calls go out now, its notices join the next
/// frame's events (`ScriptState::produced`). A call that stops for data finishes when the data
/// lands, and its `retval` is then not read.
pub fn call_ifun<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    args: &[IFunArg],
) -> Result<IFunResult, String> {
    let mut out = Vec::new();
    let result = ifun_into(port, function, args, &mut out);
    if let Some(scripts) = port.script_state() {
        scripts.produced.extend(out);
    }
    result
}

/// A query on the player's script for the UI, answered only when the VM owns `function`: the
/// output is kept for the next frame, the result returned (`None` when the VM does not answer).
pub fn script_query<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    args: &[IFunArg],
    ret: IFunReturn,
) -> Option<IFunResult> {
    if !scripts_own(port, function) {
        return None;
    }
    let mut produced = Vec::new();
    let result = ifun_return(port, function, args, ret, &mut produced).ok()?;
    if let Some(scripts) = port.script_state() {
        scripts.produced.extend(produced);
    }
    Some(result)
}

/// A request's IFun on the player's script (a craft or skill request): its stub sends the
/// request; the events it raised come back. Its problem, when it has one, is noted.
pub fn script_request<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    args: &[IFunArg],
) -> Vec<SessionEvent> {
    let mut out = Vec::new();
    if scripts_own(port, function) {
        if let Err(e) = ifun_into(port, function, args, &mut out) {
            if let Some(scripts) = port.script_state() {
                scripts.problems.push(e);
            }
        }
    }
    out
}
