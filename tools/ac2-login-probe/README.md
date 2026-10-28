# ac2-login-probe

Headless network logon only: no window, renderer or scripts. It sends the retail logon, answers
the DAT interrogation, waits for the character set, and quits the way the client does at
character selection (`Turbine::Client::Disconnect` -> `ClientNet::LogOffServer`). Each stage
prints as it happens. The password or ticket is never printed.

```
cargo run -p ac2-login-probe -- --host HOST --port PORT --account NAME [options]
```

| Option | Meaning |
|---|---|
| `--host NAME\|IP` | resolved to its first IPv4 address |
| `--port N` | the server's logon port |
| `--account NAME` | lowercased first, as retail does (`_strlwr`, `EvaluateCommandLineArg`) |
| `--glsticketdirect PASS` | password logon, auth type `0x40000002` (`gmClient::InitAuth`); omit for a plain account logon (auth type 1) |
| `--version V` | data version in the logon, default `7160` (Legions DVD). `1802` for the 2012-2017 relaunch client (build 0930, `ClientNet::SendLoginRequest` client0930 0x004D3B70). Pyrealium accepts only `7148` or `7230`. |
| `--dat-dir DIR` | DATs for the interrogation response, default `AC2_DAT_DIR`; without it the response lists no files |
| `--timeout SECS` | wait for the character set, default 45 (the client gives up on the logon after 20 tries, 2 s apart) |

## Exit codes

| Code | Stage |
|---|---|
| 0 | character set reached, logoff sent |
| 1 | usage, unreadable exe, unusable DATs |
| 2 | host did not resolve to an IPv4 address |
| 3 | no reply, or no character set before the timeout, or the logon retries ran out |
| 4 | refused: the server sent a reason (named when it is a client `ID_ConnectionError_*` / `ID_NetError_*` string) or closed the connection |
| 5 | protocol error: a malformed answer, or a character set rejection |

## icefire

```
ac2-login-probe --host icefire.portalspace.net --port 8610 --version 1802 --account NAME --glsticketdirect PASSWORD
```

icefire runs the final relaunch client (NetVersion `1802`), which packs the ticket exactly as
Legions does (auth type `0x40000002`). The probe never sends the RSA session key: auth flags are
0 and the key blob is empty. After the character list the relaunch server may send 0930-era
messages the 7160 decoder does not know; the probe reports each as `undecodable server message`
with its opcode and length and keeps going.

## Local Pyrealium (admin 7148 emulator)

Pyrealium listens on `127.0.0.1:9000` (and 9001 for the connect-ack) by default, or on the
`--bind` address it was started with. It speaks only the admin 7148 layouts:

```
ac2-login-probe --host 127.0.0.1 --port 9000 --version 7148 --account local
```

Its auth rules (`Login::supports_developer_auth`, `Login::native_ticket`): version `7148` or
`7230`; a plain account logon (auth type 1, no password) must carry the configured account name;
a `--glsticketdirect` logon is accepted only when the value is a 64-character lowercase hex
token (the client's `/glsticket` form).

## What it does not do

It does not enter the world, send queue 7 traffic or run the GUN/MSN ticket exchange; the
ticket goes through `--glsticketdirect` as retail's direct mode does.
