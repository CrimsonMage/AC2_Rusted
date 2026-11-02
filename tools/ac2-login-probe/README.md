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
| `--version V` | data version in the logon; by default the DATs' release's: `7230` for the 7230 set (the 7230 client's string), `7160` for the Legions DVD's. `1802` for the 2012-2017 relaunch client (build 0930, `ClientNet::SendLoginRequest` client0930 0x004D3B70). AC2e serves its 7230 data to `7230` logons and converted 7148 data to `7148`. |
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

## Local AC2e server

The client is tested against the AC2e server (github.com/Thwargle/AC2e), run on the 7230 DATs
(`AC2_DAT_DIR`, default `../dataset-7230`; `AC2E_DATA=legions` with the DVD set instead).
`scripts/ac2e-login-test.sh` is the live logon test: it starts a server on `127.0.0.1:19380`,
runs this probe against it and stops the server, exiting with the probe's code.

```
scripts/ac2e-login-test.sh
```

To keep a server running for the client or the probe, `scripts/ac2e-server.sh` starts one on
`127.0.0.1:9000` (the client's default `--server`). It builds AC2e from `AC2E_DIR` (default
`../third_party/ac2e-src/repo`) when needed, keeps only the placements these DATs hold, and runs
`ac2e serve` with AC2e's launcher options:

```
scripts/ac2e-server.sh
ac2-login-probe --host 127.0.0.1 --port 9000 --account local --dat-dir ../dataset-7230
```

AC2e speaks the admin 7148 layouts (7230 is experimental). Without `--auth-data` it takes a plain
account logon (auth type 1, no password) for its developer account, `local`.

## What it does not do

It does not enter the world or send queue 7 traffic; the ticket goes through `--glsticketdirect`
as retail's direct mode does.
