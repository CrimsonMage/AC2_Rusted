//! `gmAssistanceUI` on the option toolbar: the page's messages reach it, and a complete call goes
//! to the GUN service as `RequestGUN_CallCreate` ([`crate::Client::call_create`]). The answer
//! (queued or not) is the page's: it resets, or opens its "can't connect" box.

use ac2_ui::{AssistancePanel, AssistanceRequest, DialogFactory, DialogOutcome, UiMessage, UiTree};

use crate::ui_sources::ClientPlayer;

/// The page's message, with the world's name (`Turbine::Client::GetWorldName`) as the call's world.
pub(crate) fn assistance_message(
    page: &mut AssistancePanel,
    tree: &mut UiTree,
    dialogs: &mut DialogFactory,
    player: &ClientPlayer<'_>,
    message: &UiMessage,
) {
    let world = player.client.borrow().world_name().to_owned();
    let Some(request) = page.message(tree, dialogs, message, &world) else {
        return;
    };
    let AssistanceRequest::CallCreate {
        call_type,
        world,
        target,
        description,
    } = request;
    let queued = player
        .client
        .borrow_mut()
        .call_create(call_type, &world, &target, &description);
    page.send_answered(tree, dialogs, queued);
}

/// The message box the page opened is answered (`RecvNotice_CloseMessageDialog`).
pub(crate) fn assistance_outcomes(page: &mut AssistancePanel, outcomes: &[DialogOutcome]) {
    for outcome in outcomes {
        if let DialogOutcome::Message { context } = *outcome {
            page.close_message(context);
        }
    }
}
