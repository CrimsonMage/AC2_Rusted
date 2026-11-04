use ac2_net::protocol::{
    GameplayOptionsProfile, HashKey, HeapObject, NativeObject, PackValue, PackedEntry,
    PackedReference, ScriptCall, ScriptHash, ScriptRequest, StringInfo, StringVariable,
};

use super::handler::*;
use super::*;
use crate::notice::GameplayNotice;
use crate::tables::StringTableIds;

/// The names retail's table 0x41000007 gives the ported commands, and one emote.
fn commands() -> ChatCommands {
    let mut c = ChatCommands::new("say");
    for (names, handler) in [
        ("say, s, c", SAY),
        ("tell, t, send", TELL),
        ("retell, rt", RETELL),
        ("reply, r", REPLY),
        ("squelch", SQUELCH),
        ("f, group, g", FELLOWSHIP),
        ("a, gu, guild, as, o", ALLEGIANCE),
        ("ct, auction", AUCTION),
        ("cr, broadcast, br, y, yell, shout", BROADCAST),
        ("ck", CK),
        ("emote, em, me, e", EMOTE),
        ("who, w", 0x163),
    ] {
        c.register(names, handler, true);
    }
    c.register("alias", 0xB5, false);
    c.add_emote("wave", 0x1D);
    // A later registration keeps the first handler.
    c.add_emote("say", 0x99);
    c.set_help(TELL, "usage: /tell".into());
    for (id, word) in SQUELCH_WORDS.iter().zip(["add", "remove", "status"]) {
        c.set_word(*id, word.into());
    }
    c
}

fn command(line: &str) -> Parsed {
    process_text(line, &commands())
}

fn parsed(handler: u32, text: &str, args: &[&str]) -> Parsed {
    Parsed::Command {
        handler,
        text: text.into(),
        args: args.iter().map(|a| a.to_string()).collect(),
    }
}

#[test]
fn lines_split_into_the_command_its_text_and_tokens() {
    assert_eq!(
        command("  /Tell  Birch,  hi there "),
        parsed(TELL, "Birch,  hi there", &["Tell", "Birch,", "hi", "there"])
    );
    // Speech gets the default command; leading slashes all go.
    assert_eq!(command("hello"), parsed(SAY, "hello", &["say", "hello"]));
    assert_eq!(command("//s hi"), parsed(SAY, "hi", &["s", "hi"]));
    // The text loses the command's characters from its front, as a set (up to the space).
    assert_eq!(
        command("/s sss x"),
        parsed(SAY, "sss x", &["s", "sss", "x"])
    );
    // Quotes make one token and skip the character after the closing quote.
    assert_eq!(
        command(r#"/t "Old Man"x hi"#),
        parsed(TELL, r#""Old Man"x hi"#, &["t", "Old Man", "hi"])
    );
    assert_eq!(command(r#"/t "Old Man hi"#), Parsed::Failure);
    assert_eq!(command("   "), Parsed::Empty);
    assert_eq!(command("//"), Parsed::Empty);
    assert_eq!(command("/nonsense"), Parsed::Unknown);
    assert_eq!(command("/WAVE"), parsed(EMOTE_BY_NAME, "", &["WAVE"]));
}

/// `gmCWInterface::ExpandText` / `HandleAlias`: each `;alias` token is replaced in place by its
/// text, expanded again; a token being expanded stays as it is (a loop); a `;` alone is left;
/// a token without an alias is kept.
#[test]
fn alias_tokens_expand_in_place_and_loops_stop() {
    let aliases = [
        (";hi", "/s hello ;who"),
        (";who", "friend"),
        (";a1", ";a2"),
        (";a2", ";a1"),
    ];
    let mut inq = |alias: &str| {
        aliases
            .iter()
            .find(|(a, _)| *a == alias)
            .map(|(_, t)| t.to_string())
    };
    assert_eq!(expand_text(";hi", &mut inq), "/s hello friend");
    assert_eq!(
        expand_text("/s ;who there; ok", &mut inq),
        "/s friend there; ok"
    );
    assert_eq!(expand_text(";a1 x", &mut inq), ";a1 x");
    assert_eq!(expand_text(";none, ;who!", &mut inq), ";none, friend!");
}

fn tables() -> StringTableIds {
    let mut t = StringTableIds::default();
    for (e, did) in [
        (0x4100_0001, 0x2500_0001),
        (0x4100_0004, 0x2500_0004),
        (0x4100_000D, 0x2500_000D),
        (0x4100_0018, 0x2500_0018),
        (0x4100_001B, 0x2500_001B),
    ] {
        t.insert(e, did);
    }
    t
}

fn state() -> ChatState {
    let mut s = ChatState::default();
    s.set_commands(commands());
    s
}

const ME: Speaker = Speaker {
    name: "Explorer",
    monarch: 0,
};

fn run(s: &mut ChatState, line: &str) -> Vec<ShellAction> {
    s.process_text(line, ME, &tables())
}

fn tell(to: &str, text: &str) -> ShellAction {
    ShellAction::Send(ScriptRequest::Tell {
        mode: 0,
        text: lines::literal(text),
        recipient: lines::literal(to),
    })
}

/// A shown line's display type, table, string and the literal variables' text.
fn shown(action: &ShellAction) -> (u32, u32, u32, Vec<String>) {
    let ShellAction::Notice(GameplayNotice::DisplayStringInfo { kind, text }) = action else {
        panic!("{action:?}");
    };
    let vars = text
        .variables
        .iter()
        .map(|(_, v)| match v {
            StringVariable::Nested(n) => n
                .literal
                .clone()
                .unwrap_or_else(|| format!("{:#x}", n.string_id)),
            other => format!("{other:?}"),
        })
        .collect();
    (*kind, text.table_id, text.string_id, vars)
}

#[test]
fn tells_split_the_name_and_remember_it_for_retell() {
    let mut s = state();
    assert_eq!(run(&mut s, "/rt again"), []);
    assert_eq!(
        run(&mut s, "/t Birch hi there"),
        [tell("Birch", "hi there")]
    );
    assert_eq!(
        run(&mut s, "/tell Old Man, hi  there"),
        [tell("Old Man", "hi there")]
    );
    assert_eq!(run(&mut s, "/rt again"), [tell("Old Man", "again")]);
    // Usage, a short name, no message.
    let help = run(&mut s, "/tell Birch");
    assert_eq!(shown(&help[0]), (8, 0, 0, vec![]));
    let short = run(&mut s, "/t , hi");
    assert_eq!(
        shown(&short[0]),
        (33, 0x2500_0004, 0x913D_4776, vec![" ,".into()])
    );
    let empty = run(&mut s, "/t Birch, ");
    assert_eq!(empty.len(), 1, "one token: usage");
    let empty = run(&mut s, "/t Old Man,");
    assert_eq!(
        shown(&empty[0]),
        (33, 0x2500_0004, 0x2BA2_9D26, vec!["Old Man".into()])
    );
    // A GUN name sends nothing but is still the retell name.
    assert_eq!(run(&mut s, "/t a@b hi"), []);
    assert_eq!(run(&mut s, "/rt hi"), []);
}

#[test]
fn replies_go_to_the_last_literal_teller() {
    let mut s = state();
    assert_eq!(run(&mut s, "/r hi"), []);
    s.heard_tell(&StringInfo::default());
    assert_eq!(run(&mut s, "/r hi"), []);
    s.heard_tell(&lines::literal("Birch"));
    assert_eq!(run(&mut s, "/r hi you"), [tell("Birch", "hi you")]);
    s.heard_tell(&lines::literal(""));
    let shown_line = run(&mut s, "/r hi");
    assert_eq!(
        shown(&shown_line[0]),
        (33, 0x2500_0004, 0x3CD0_9224, vec![])
    );
}

#[test]
fn speech_emotes_and_unknown_commands() {
    let mut s = state();
    let say = |t: &str| {
        ShellAction::Send(ScriptRequest::Say {
            mode: 0,
            text: lines::literal(t),
        })
    };
    assert_eq!(run(&mut s, "Hello there"), [say("Hello there")]);
    assert_eq!(run(&mut s, "/say"), [lines::show(8, lines::literal(""))]);
    assert_eq!(
        run(&mut s, "/me waves"),
        [ShellAction::Send(ScriptRequest::EmoteText("waves".into()))]
    );
    assert_eq!(
        run(&mut s, "/Wave at you"),
        [ShellAction::Send(ScriptRequest::Emote(0x1D))]
    );
    let unknown = run(&mut s, "/dance");
    assert_eq!(shown(&unknown[0]), (1, 0x2500_0018, 0xA968_48DC, vec![]));
    assert_eq!(
        run(&mut s, "/w Birch"),
        [ShellAction::Command {
            handler: 0x163,
            text: "Birch".into(),
            args: vec!["w".into(), "Birch".into()],
        }]
    );
}

fn heap_entry(pkg_id: u32, slots: Vec<u32>, native: Option<NativeObject>) -> PackedEntry {
    PackedEntry::Heap {
        flags: 0x4400_0100,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native,
            tags: vec![2; slots.len()],
            slots,
        },
    }
}

#[test]
fn rooms_come_from_the_server_calls() {
    let mut s = state();
    let t = tables();
    let f = run(&mut s, "/f hi");
    assert_eq!(shown(&f[0]), (1, 0x2500_000D, 0x6B9F_0584, vec![]));
    let fellowship = PackedReference {
        ids: vec![1],
        entries: vec![heap_entry(0x1F0, vec![0, 0, 0, 0x77, 0], None)],
    };
    let call = ScriptCall::new(0x820C_002B, vec![PackValue::Reference(fellowship)]).unwrap();
    assert_eq!(s.server_call(&call, &t), []);
    let room = |room, text: &str, flags| ShellAction::SendToRoom {
        room,
        text: text.into(),
        flags,
    };
    assert_eq!(run(&mut s, "/f hi all"), [room(0x77, "hi all", 1)]);
    let clear = ScriptCall::new(0x820C_0028, vec![]).unwrap();
    s.server_call(&clear, &t);
    assert_eq!(run(&mut s, "/f hi").len(), 1);

    // The allegiance: a monarch, then a room (and its "joined" line).
    let a = run(&mut s, "/a hi");
    assert_eq!(shown(&a[0]), (1, 0x2500_0001, 0x257A_6C5D, vec![]));
    let me = Speaker { monarch: 5, ..ME };
    let a = s.process_text("/a hi", me, &t);
    assert_eq!(shown(&a[0]), (1, 0x2500_001B, 0xEEF5_B265, vec![]));
    let joined = s.server_call(
        &ScriptCall::new(0x8203_0030, vec![PackValue::Int(0x99)]).unwrap(),
        &t,
    );
    let GameplayNotice::DisplayStringInfo { kind, text } = &joined[0] else {
        panic!();
    };
    assert_eq!((*kind, text.string_id), (8, 0x6B5B_5AAD));
    assert_eq!(s.process_text("/a hi", me, &t), [room(0x99, "hi", 1)]);
    let left = s.server_call(
        &ScriptCall::new(0x8203_0030, vec![PackValue::Int(0)]).unwrap(),
        &t,
    );
    let GameplayNotice::DisplayStringInfo { text, .. } = &left[0] else {
        panic!();
    };
    assert_eq!(text.string_id, 0x34C2_844D);

    // Channels: not listed (subscribed through a filter or not), then the table's room.
    let ct = run(&mut s, "/ct selling");
    assert_eq!(
        shown(&ct[0]),
        (1, 0x2500_0004, 0x556D_D29F, vec!["0x6ee382e5".into()])
    );
    let mut profile = GameplayOptionsProfile::default();
    profile.filters.pairs = vec![(2, 0x10_0000)];
    profile.aliases.1 = vec![(";sell".into(), "/ct selling".into())];
    s.login_options(&profile);
    assert_eq!(shown(&run(&mut s, ";sell")[0]).2, 0x9FEE_074C);
    let table = PackedReference {
        ids: vec![1, 2],
        entries: vec![
            heap_entry(
                u32::MAX,
                vec![],
                Some(NativeObject::Hash(ScriptHash {
                    native_type: ac2_net::protocol::ntype::AR_HASH,
                    bucket_index: 0,
                    keys: vec![HashKey::Value(0x10_0000)],
                    values: vec![2],
                })),
            ),
            heap_entry(0x2023, vec![0, 1, 1, 0x55], None),
        ],
    };
    let call = ScriptCall::new(0x8205_001B, vec![PackValue::Reference(table)]).unwrap();
    s.server_call(&call, &t);
    assert_eq!(run(&mut s, ";sell"), [room(0x55, "selling", 0)]);
    assert_eq!(run(&mut s, "/ct"), []);
    assert_eq!(run(&mut s, "/ck").len(), 1, "usage");
}

#[test]
fn squelch_reports_the_missing_service_and_keeps_the_gun_list() {
    let mut s = state();
    let t = tables();
    let add = run(&mut s, "/squelch add Old Man");
    assert_eq!(
        shown(&add[0]),
        (8, 0x2500_0004, 0x309D_0867, vec!["Old Man".into()])
    );
    assert_eq!(shown(&add[1]), (1, 0x2500_0004, 0x3E6E_8EF5, vec![]));
    assert_eq!(shown(&run(&mut s, "/squelch ADD")[0]).2, 0xF911_D128);
    assert_eq!(
        shown(&run(&mut s, "/squelch add explorer")[0]).2,
        0x7DDA_2C3C
    );
    assert_eq!(shown(&run(&mut s, "/squelch remove x")[0]).2, 0x6AE5_BC25);
    assert_eq!(shown(&run(&mut s, "/squelch status")[0]).2, 0x3E6E_8EF5);
    assert_eq!(run(&mut s, "/squelch").len(), 1, "usage");

    // The GUN results keep the list.
    let ok = s.ignored_added(0, "Birch", &t);
    assert!(matches!(
        ok,
        GameplayNotice::DisplayStringInfo { kind: 8, .. }
    ));
    let failed = s.ignored_added(0xC740, "Oak", &t);
    assert!(matches!(
        failed,
        GameplayNotice::DisplayStringInfo { kind: 1, .. }
    ));
    assert!(s.has_squelched("BIRCH") && !s.has_squelched("Oak"));
    s.ignored_list(0, false, vec!["Birch".into(), "B+x".into()], &t);
    assert!(!s.has_squelched("B+x"));
    s.ignored_deleted(0, "birch", &t);
    assert!(!s.has_squelched("Birch"));
    // Showing the list empties it, as retail's listing pops the names it keeps.
    s.ignored_list(0, false, vec!["Ash".into()], &t);
    let shown_list = s.ignored_list(0, true, vec!["Ash".into()], &t);
    assert_eq!(shown_list.len(), 2);
    assert!(s.squelched().is_empty());
}

#[test]
fn histories_follow_the_lines_tells_and_teller_names() {
    use super::ChatHistory::{Command, Reply, Retell};
    let mut s = state();
    run(&mut s, "/t Birch hi");
    run(&mut s, "/t Oak, yo");
    run(&mut s, "/t a@b hi");
    run(&mut s, "hello");
    assert_eq!(s.history(Command, true).as_deref(), Some("hello"));
    assert_eq!(s.history(Command, true).as_deref(), Some("/t a@b hi"));
    assert_eq!(s.history(Command, false).as_deref(), Some("hello"));
    // GUN names never reach the told history.
    assert_eq!(s.history(Retell, true).as_deref(), Some("Oak"));
    assert_eq!(s.history(Retell, true).as_deref(), Some("Birch"));
    s.heard_tell(&lines::literal("Ash"));
    assert_eq!(s.history(Reply, true).as_deref(), Some("Ash"));
    // A processed line restarts the told and heard walks (`ProcessText_Done`).
    run(&mut s, "/rt again");
    assert_eq!(s.history(Retell, true).as_deref(), Some("Oak"));
    assert_eq!(s.history(Reply, false), None);
}

#[test]
fn heard_room_chat_shows_in_the_window_of_its_room() {
    use crate::shell::lines::{KIND_FELLOWSHIP, KIND_INFO};
    use ac2_net::protocol::GunChatExtraInfo;
    let mut s = state();
    let t = tables();
    let fellowship = PackedReference {
        ids: vec![1],
        entries: vec![heap_entry(0x1F0, vec![0, 0, 0, 0x77, 0], None)],
    };
    let call = ScriptCall::new(0x820C_002B, vec![PackValue::Reference(fellowship)]).unwrap();
    s.server_call(&call, &t);
    let flagged = GunChatExtraInfo::new(0, String::new());
    let (kind, line) = s.hear_from_room(0x77, "Ann", "hi", Some(&flagged));
    assert_eq!(kind, KIND_FELLOWSHIP);
    assert_eq!(line.literal.as_deref(), Some("[Ann] hi"));
    let plain = GunChatExtraInfo {
        misc_flags: 1,
        ..flagged
    };
    let (_, line) = s.hear_from_room(0x77, "Ann", "hi", Some(&plain));
    assert_eq!(line.literal.as_deref(), Some("hi"));
    // A room the player has no window for: the standard window, and the prefix.
    let (kind, line) = s.hear_from_room(0x55, "Bo", "yo", None);
    assert_eq!(
        (kind, line.literal.as_deref()),
        (KIND_INFO, Some("[Bo] yo"))
    );
}

/// `CWInterface::DisplayHelpText`: a command with help shows its text (type 8), an unknown one
/// the unrecognized-command line (type 1) naming it, and an empty command the label then every
/// help text in name order.
#[test]
fn help_is_shown_for_a_command_an_unknown_one_and_all() {
    let mut state = ChatState::default();
    let mut c = commands();
    c.set_help(SAY, "usage: /say".into());
    state.set_commands(c);
    let tables = StringTableIds::default();
    let none = crate::shell::ScriptHelp::default();
    let kinds = |actions: Vec<ShellAction>| -> Vec<(u32, Option<String>)> {
        actions
            .into_iter()
            .filter_map(|a| match a {
                ShellAction::Notice(GameplayNotice::DisplayStringInfo { kind, text }) => {
                    Some((kind, text.literal))
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        kinds(state.display_help("Tell", &tables, &none)),
        vec![(8, Some("usage: /tell".into()))]
    );
    let unknown = state.display_help("nosuch", &tables, &none);
    let ShellAction::Notice(GameplayNotice::DisplayStringInfo { kind, text }) = &unknown[0] else {
        panic!("a notice");
    };
    assert_eq!(*kind, 1);
    assert_eq!(text.string_id, ac2_net::tokenize(b"ID_UnrecognizedCommand"));
    assert_eq!(text.variables.len(), 1);
    // Every command of a handler with help appears, sorted by name: c, s, say, send, t, tell.
    let all = kinds(state.display_help("", &tables, &none));
    assert_eq!(all[0].0, 8);
    assert_eq!(
        all[1].1.as_deref(),
        Some(
            "  c -- usage: /say  s -- usage: /say  say -- usage: /say  send -- usage: /tell  \
             t -- usage: /tell  tell -- usage: /tell"
        )
    );
}

/// A shown line's literal text and display type.
fn shown_text(action: &ShellAction) -> (u32, String) {
    let ShellAction::Notice(GameplayNotice::DisplayStringInfo { kind, text }) = action else {
        panic!("{action:?}");
    };
    (*kind, text.literal.clone().expect("a literal line"))
}

fn ui_state() -> ChatState {
    let mut c = commands();
    c.register_client_commands();
    let mut s = ChatState::default();
    s.set_commands(c);
    s
}

#[test]
fn saveui_and_loadui_name_the_file_and_say_so() {
    let mut s = ui_state();
    for (line, expect) in [
        (
            "/saveui mine",
            (
                "Saving UI to file mine.txt.\n",
                ShellAction::SaveUiLayout {
                    name: "mine".into(),
                },
            ),
        ),
        (
            "/loadui mine",
            (
                "Loading UI from file mine.txt.\n",
                ShellAction::LoadUiLayout {
                    name: "mine".into(),
                },
            ),
        ),
        (
            "/SAVEUI UI-Default",
            (
                "Saving UI to file UI-Default.txt.\n",
                ShellAction::SaveUiLayout {
                    name: "UI-Default".into(),
                },
            ),
        ),
        // No name: "UI-Default"; Turbine's loadui shows the saving line here.
        (
            "/saveui",
            (
                "Saving UI to file UI-Default.txt.\n",
                ShellAction::SaveUiLayout {
                    name: "UI-Default".into(),
                },
            ),
        ),
        (
            "/loadui",
            (
                "Saving UI to file UI-Default.txt.\n",
                ShellAction::LoadUiLayout {
                    name: "UI-Default".into(),
                },
            ),
        ),
        // A quoted name keeps its space.
        (
            r#"/loadui "my ui""#,
            (
                "Loading UI from file my ui.txt.\n",
                ShellAction::LoadUiLayout {
                    name: "my ui".into(),
                },
            ),
        ),
    ] {
        let out = run(&mut s, line);
        assert_eq!(out.len(), 2, "{line}");
        assert_eq!(shown_text(&out[0]), (8, expect.0.to_owned()), "{line}");
        assert_eq!(out[1], expect.1, "{line}");
    }
}

#[test]
fn saveui_and_loadui_with_more_than_one_argument_show_their_usage() {
    let mut s = ui_state();
    for (line, cmd) in [("/saveui a b", "saveui"), ("/loadui a b c", "loadui")] {
        let out = run(&mut s, line);
        assert_eq!(out.len(), 1, "{line}");
        assert_eq!(
            shown_text(&out[0]),
            (
                8,
                format!("usage: /{cmd} < filename > (filename is optional)\n")
            )
        );
    }
    // The 7230 script's commands are untouched, and the new ones sit outside its handler ids.
    assert_eq!(s.commands().handler("say"), Some(SAY));
    const { assert!(SAVE_UI > 0xFFFF && LOAD_UI > 0xFFFF) };
}

#[test]
fn a_command_name_holding_a_space_is_refused_and_the_first_handler_stays() {
    // `ShellCmdTable::AddCmdHandler`: the lowered name must hold no space and be new.
    let mut c = ChatCommands::new("say");
    c.register("Two Words; Wave ", 0x10, true);
    c.register("wave", 0x11, true);
    assert_eq!(c.handler("two words"), None);
    assert_eq!(c.handler("WAVE"), Some(0x10));
}

/// `CWAllocator::ResetShell` at `gmGamePlayUI::EndSession`: the next character starts with an
/// empty command history; the registered commands stay.
#[test]
fn ending_the_session_empties_the_command_history() {
    let mut s = state();
    run(&mut s, "/say hello");
    assert_eq!(
        s.history(ChatHistory::Command, true).as_deref(),
        Some("/say hello")
    );
    s.reset_shell();
    assert_eq!(s.history(ChatHistory::Command, true), None);
    assert_eq!(s.commands().handler("say"), Some(SAY));
}
