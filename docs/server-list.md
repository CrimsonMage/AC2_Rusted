# Servers in the launcher

The launcher (`crates/ac2-launcher`, the `ac2-launcher` program) works the way ThwargLauncher
does. A server is a **name**, a **connect string** (`host:port`) and a **type** (`emu`). The
launcher builds the client's command line for that type from the connect string and the
player's account and password, and starts the client with it, behind the scenes. Players never
write a command line.

The launcher starts with one server, an AC2e on this computer (`127.0.0.1:9000`). No
community server is built in. Add a server with **Add server** (name, connect string, type), or
take it from a published list.

## Server types and what the launcher sends

| Type (`emu`) | For | The command line the launcher sends |
|---|---|---|
| `AC2E` | AC2e servers (the Legions-era protocol) | `--host HOST --port PORT --version 7148 --account ACCOUNT`, plus `--glsticket TICKETFILE` when the server has a login server |
| `Relaunch` | relaunch servers (the 2012-2017 client) | `--host HOST --port PORT --version 1802 --account ACCOUNT --glsticket TICKETFILE --no-gun` |

The command line also always carries `--dat-dir` with the DAT folder chosen in Settings.

**Sign-in:**

- **AC2E without a login server:** the developer account (`local` when the account box is
  empty), with no password.
- **AC2E with a login server** (`LoginServerUrlKey`):
  - The launcher signs in at `POST {login server}/api/login` with the account and password.
  - It does this twice: one single-use ticket for the game, one for the server's GUN service
    (passed as `--gunticketoverride ACCOUNT:TICKET`).
  - It sends a password only over HTTPS, or over HTTP to this computer.
- **Relaunch:** the password is the ticket, as retail's `glsticketdirect` passes it.

`TICKETFILE` is a file only the player can read. The client reads it and deletes it, and the
launcher waits for that. A ticket left behind is removed. A ticket or password never goes on
the command line, and the launcher never saves a password.

**Friends (GUN):**

- **AC2E:** the server's own GUN service (AC2e starts one with every server). This is the
  client's default, so nothing extra is sent unless the port isn't 2310 (then `--gunport`).
- **Relaunch:** none (`--no-gun`). The relaunch servers have no GUN service.
- **`gun_server`:** overrides either default. `none`, or a shared GUN server's `host:port`,
  sent as `--gunserver HOST --gunport PORT --gunticket`.

## The server lists

The player's servers (`launcher-servers.xml` in the AC2 configuration directory) and the
published server list share ThwargLauncher's `ServerItem` XML. Set the published list's
address (a web address or a file) in the launcher's Settings.

```xml
<?xml version="1.0" encoding="utf-8"?>
<ArrayOfServerItem>
  <ServerItem>
    <id>7f3b9c21-4e8a-4d16-b2f7-1a9e5c0d3b84</id>
    <name>IceFire</name>
    <description>The relaunch community server.</description>
    <emu>Relaunch</emu>
    <server_host>icefire.portalspace.net</server_host>
    <server_port>8610</server_port>
    <type>PvE</type>
    <status>Stable</status>
    <website_url></website_url>
    <discord_url></discord_url>
  </ServerItem>
  <ServerItem>
    <id>3c1d0e55-6a2b-4f8e-9d7c-2b5a1e4f0c99</id>
    <name>Example AC2e</name>
    <description>An AC2e server with accounts.</description>
    <emu>AC2E</emu>
    <server_host>ac2.example.net</server_host>
    <server_port>9000</server_port>
    <LoginServerUrlKey>https://ac2.example.net</LoginServerUrlKey>
    <status>Development</status>
  </ServerItem>
</ArrayOfServerItem>
```

| Element | Meaning |
|---|---|
| `id` | A GUID for the server. |
| `name`, `description` | Shown in the launcher. |
| `emu` | `AC2E` or `Relaunch`. Entries for other games (ThwargLauncher's `ACE`, `GDLE`) are skipped. |
| `server_host` and `server_port`, or `connect_string` | The address. A published list uses `server_host` and `server_port`; the player's own servers use `connect_string` (`host:port`), as ThwargLauncher does. |
| `LoginServerUrlKey` | AC2E only: the AC2e account service. Empty means the developer account. |
| `gun_server` | Optional: `none`, or a shared GUN server's `host:port`. Empty means the type's default. |
| `type`, `status`, `website_url`, `discord_url` | Shown in the launcher. |

The launcher skips an entry with a bad address, or one listed twice (the same address), and
says so. It ignores elements it does not know.
