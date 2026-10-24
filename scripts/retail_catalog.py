#!/usr/bin/env python3
"""The retail behaviour catalog: every function of the AC2 admin 7148 client, grouped by the
AC2_Rusted crate that owns it, as a checklist under docs/retail-catalog/.

usage:
  retail_catalog.py build [--index PATH]
  retail_catalog.py mark STATUS NAME_OR_ADDRESS... [--note TEXT] [--index PATH]
  retail_catalog.py status [--index PATH]

STATUS is one of: todo, done, partial, skip (written ' ', 'x', '~', '-').
NAME_OR_ADDRESS is an admin address (8 hex digits), a full name (`Class::Method`, every
overload) or `Class::*` for a whole class.

The catalog reads the decompile index (`INDEX/ac2client_index.sqlite` and
`INDEX/ac2_db_type_catalog.csv` beside it); `--index` or `AC2_INDEX` points at the sqlite file.
Explicit statuses live in docs/retail-catalog/marks.tsv; a checkbox edited by hand in the
markdown is picked up into it on the next build or mark.
"""

import argparse
import csv
import os
import re
import sqlite3
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
OUT = REPO / "docs" / "retail-catalog"
DEFAULT_INDEX = REPO.parent / "INDEX" / "ac2client_index.sqlite"

STATUS = {"todo": " ", "done": "x", "partial": "~", "skip": "-"}
SKIPPED_FAMILIES = {"static_initializer", "exception_funclet", "deleting_destructor"}

# Crates in milestone order (ARCHITECTURE.md), then the replaced/out-of-scope bucket.
CRATES = [
    ("ac2-core", "Core runtime: ids, strings, timers, errors, serialization helpers"),
    ("ac2-math", "Math and geometry"),
    ("ac2-dat", "DAT files and the database layer"),
    ("ac2-db", "Database objects, properties, enum maps, string tables, client cache"),
    ("ac2-net", "Network transport, RPC, crypto, GUN, chat, HTTP"),
    ("ac2-weenie", "Weenie objects, client events, actions, the session"),
    ("ac2-anim", "Animation system"),
    ("ac2-assets", "Render asset codecs: materials, meshes, appearance, images"),
    ("ac2-world", "Cells, landblocks, regions, terrain, sky, water, game time"),
    ("ac2-render", "Scene and world rendering"),
    ("ac2-gpu", "The render device"),
    ("ac2-physics", "Physics objects, collision, motion, paths, BSP"),
    ("ac2-smartbox", "SmartBox, camera, world manager, object maintenance"),
    ("ac2-client", "Client entry, shell, preferences, platform devices"),
    ("ac2-ui", "UI framework, input maps, IME, game panels, maps"),
    ("ac2-script", "WSL interpreter and native glue"),
    ("ac2-gameplay", "Characters, qualities, quests, fellowships, crafting, trade"),
    ("ac2-audio", "Sound"),
    ("replaced", "Third-party libraries, CRT and container templates replaced by Rust"),
]

# (path prefix, crate, default status, note). Longest matching prefix wins.
RULES = [
    ("_libraries/", "replaced", "skip", "third-party library"),
    ("core/src/debug", "ac2-core", "skip", "crash reporting and debug tooling"),
    ("core/src/", "ac2-core", "todo", ""),
    ("portal/shared/src/error", "ac2-core", "todo", ""),
    ("portal/shared/src/utils", "ac2-core", "todo", ""),
    ("portal/shared/src/timestamper", "ac2-core", "todo", ""),
    ("portal/shared/src/version", "ac2-core", "todo", ""),
    ("portal/shared/src/random", "ac2-core", "todo", ""),
    ("portal/shared/src/stringtoken", "ac2-core", "todo", ""),
    ("portal/shared/src/filenode", "ac2-core", "skip", "text source files (admin tools)"),
    ("portal/shared/src/pparser", "ac2-core", "skip", "text source files (admin tools)"),
    ("portal/shared/src/perfmon", "ac2-core", "skip", "performance monitor (admin tools)"),
    ("portal/shared/src/reg_edit", "ac2-core", "skip", "registry editing (admin tools)"),
    ("portal/shared/src/setproject", "ac2-core", "skip", "project setup (admin tools)"),
    ("portal/client/src/bench", "ac2-core", "skip", "benchmarks (admin tools)"),
    ("portal/client/src/platformstring", "ac2-core", "todo", ""),
    ("portal/client/src/cerror", "ac2-core", "todo", ""),
    ("portal/shared/src/geometry", "ac2-math", "todo", ""),
    ("portal/shared/src/database", "ac2-dat", "todo", ""),
    ("portal/shared/src/filedb", "ac2-dat", "todo", ""),
    ("portal/shared/src/diskcon", "ac2-dat", "todo", ""),
    ("portal/shared/src/datasrc", "ac2-dat", "todo", ""),
    ("portal/shared/src/datcompat", "ac2-dat", "todo", ""),
    ("portal/shared/src/pheader", "ac2-dat", "todo", ""),
    ("portal/shared/src/fileformat", "ac2-dat", "todo", ""),
    ("portal/shared/src/property", "ac2-db", "todo", ""),
    ("portal/shared/src/enummap", "ac2-db", "todo", ""),
    ("portal/shared/src/stringtable", "ac2-db", "todo", ""),
    ("portal/shared/src/taboo", "ac2-db", "todo", ""),
    ("portal/shared/src/languagespecific", "ac2-db", "todo", ""),
    ("ac2/shared/src/languagespecific", "ac2-db", "todo", ""),
    ("ac2/shared/src/database", "ac2-db", "todo", ""),
    ("portal/client/src/clcache", "ac2-db", "todo", ""),
    ("ac2/client/src/ccache", "ac2-db", "todo", ""),
    ("portal/shared/src/newnet", "ac2-net", "todo", ""),
    ("portal/shared/src/rpc", "ac2-net", "todo", ""),
    ("portal/shared/src/crypto", "ac2-net", "todo", ""),
    ("portal/shared/src/vlong", "ac2-net", "todo", ""),
    ("portal/client/src/clinet", "ac2-net", "todo", ""),
    ("portal/client/src/http", "ac2-net", "todo", ""),
    ("portal/client/src/ckeystone", "ac2-net", "todo", ""),
    ("portal/client/src/cchat", "ac2-net", "todo", ""),
    ("ac2/client/src/gun", "ac2-net", "todo", ""),
    ("ac2/shared/src/gun", "ac2-net", "todo", ""),
    ("portal/shared/src/weenie", "ac2-weenie", "todo", ""),
    ("portal/client/src/weenie", "ac2-weenie", "todo", ""),
    ("ac2/shared/src/weenie", "ac2-weenie", "todo", ""),
    ("ac2/client/src/weenie", "ac2-weenie", "todo", ""),
    ("portal/client/src/events", "ac2-weenie", "todo", ""),
    ("ac2/client/src/events", "ac2-weenie", "todo", ""),
    ("portal/client/src/action", "ac2-weenie", "todo", ""),
    ("portal/client/src/cfactory", "ac2-weenie", "todo", ""),
    ("ac2/client/src/cfactory", "ac2-weenie", "todo", ""),
    ("portal/shared/src/anim", "ac2-anim", "todo", ""),
    ("portal/shared/src/animsys", "ac2-anim", "todo", ""),
    ("portal/client/src/render/material", "ac2-assets", "todo", ""),
    ("portal/client/src/image", "ac2-assets", "todo", ""),
    ("portal/preproc/src/imagedec", "ac2-assets", "todo", ""),
    ("portal/preproc/src/empgen", "ac2-assets", "skip", "asset preprocessing (admin tools)"),
    ("portal/shared/src/mesh", "ac2-assets", "todo", ""),
    ("portal/shared/src/objdesc", "ac2-assets", "todo", ""),
    ("ac2/shared/src/objdesc", "ac2-assets", "todo", ""),
    ("portal/shared/src/entitydesc", "ac2-assets", "todo", ""),
    ("portal/shared/src/cell", "ac2-world", "todo", ""),
    ("portal/shared/src/region", "ac2-world", "todo", ""),
    ("portal/shared/src/gametime", "ac2-world", "todo", ""),
    ("portal/shared/src/places", "ac2-world", "todo", ""),
    ("portal/client/src/smartbox/worldrender", "ac2-render", "todo", ""),
    ("portal/client/src/render/core/d3d", "ac2-gpu", "todo", ""),
    ("portal/client/src/render/ui", "ac2-ui", "todo", ""),
    ("portal/client/src/render", "ac2-render", "todo", ""),
    ("portal/client/src/scenerender", "ac2-render", "todo", ""),
    ("portal/client/src/ctrilist", "ac2-render", "todo", ""),
    ("portal/shared/src/physobj", "ac2-physics", "todo", ""),
    ("portal/shared/src/physics", "ac2-physics", "todo", ""),
    ("portal/client/src/physics", "ac2-physics", "todo", ""),
    ("portal/shared/src/path", "ac2-physics", "todo", ""),
    ("portal/shared/src/bsp", "ac2-physics", "todo", ""),
    ("portal/client/src/smartbox", "ac2-smartbox", "todo", ""),
    ("portal/client/src/objects", "ac2-smartbox", "todo", ""),
    ("portal/shared/src/objects", "ac2-smartbox", "todo", ""),
    ("portal/client/src/main", "ac2-client", "todo", ""),
    ("ac2/client/src/main", "ac2-client", "todo", ""),
    ("portal/client/src/initcli", "ac2-client", "todo", ""),
    ("portal/client/src/shell", "ac2-client", "todo", ""),
    ("portal/shared/src/shell", "ac2-client", "todo", ""),
    ("portal/client/src/clienttriggers", "ac2-client", "todo", ""),
    ("portal/client/src/device", "ac2-client", "todo", ""),
    ("portal/shared/src/prefs", "ac2-client", "todo", ""),
    ("portal/client/src/uielement", "ac2-ui", "todo", ""),
    ("portal/client/src/cinputmap", "ac2-ui", "todo", ""),
    ("portal/client/src/cimelib", "ac2-ui", "todo", ""),
    ("portal/client/src/cregion/cdetailmap", "ac2-ui", "todo", ""),
    ("portal/client/src/cregion/cmapnote", "ac2-ui", "todo", ""),
    ("ac2/client/src/ui", "ac2-ui", "todo", ""),
    ("portal/shared/src/interp/debugger", "ac2-script", "skip", "script debugger (admin tools)"),
    ("portal/shared/src/interp", "ac2-script", "todo", ""),
    ("ac2/shared/src/interp", "ac2-script", "todo", ""),
    ("portal/shared/src/gameplay", "ac2-gameplay", "todo", ""),
    ("portal/shared/src/conversation", "ac2-gameplay", "todo", ""),
    ("ac2/shared/src/", "ac2-gameplay", "todo", ""),
    ("portal/client/src/sound", "ac2-audio", "todo", ""),
    ("portal/shared/src/sound", "ac2-audio", "todo", ""),
    ("portal/client/src/cregion/csound", "ac2-audio", "todo", ""),
]
RULES.sort(key=lambda rule: -len(rule[0]))

# Methods that read or write the text source formats only the admin tools use.
TEXT_FORMAT_METHODS = re.compile(r"::(FromFileNode|ToFileNode|CheckDupAdd)$")
ADMIN_LOADER = re.compile(r"::Load[A-Z]\w*$")

LINE = re.compile(r"^- \[(.)\] `(.+?)` · ([0-9a-f]{8})(?: · [^—]*)?(?: — (.*))?$")
DB_LINE = re.compile(r"^- \[(.)\] `(DB_TYPE_[A-Z0-9_]+)`(?:.*?)(?: — (.*))?$")


def rule_for(rel_path):
    for prefix, crate, status, note in RULES:
        if rel_path.startswith(prefix):
            return crate, status, note
    return None


def index_path(arg):
    return Path(arg or os.environ.get("AC2_INDEX") or DEFAULT_INDEX)


MARKS = OUT / "marks.tsv"


def read_marks():
    """Explicit statuses from marks.tsv: key (address or DB type) -> (status char, note)."""
    marks = {}
    if MARKS.exists():
        for line in MARKS.read_text(encoding="utf-8").splitlines()[1:]:
            key, status, note = (line.split("\t") + ["", ""])[:3]
            marks[key] = (status if status != "_" else " ", note)
    return marks


def write_marks(marks):
    rows = ["key\tstatus\tnote"] + [
        f"{key}\t{'_' if status == ' ' else status}\t{note}" for key, (status, note) in sorted(marks.items())
    ]
    MARKS.write_text("\n".join(rows) + "\n", encoding="utf-8")


def read_rendered():
    """Statuses as the markdown currently shows them, to pick up checkboxes edited by hand."""
    shown = {}
    if OUT.exists():
        for path in OUT.glob("*.md"):
            for line in path.read_text(encoding="utf-8").splitlines():
                if m := LINE.match(line):
                    shown[m.group(3)] = m.group(1)
                elif m := DB_LINE.match(line):
                    shown[m.group(2)] = m.group(1)
    return shown


def load_functions(index):
    con = sqlite3.connect(index)
    rows = con.execute(
        "SELECT f.admin_address, f.legions_address, f.name, f.class, f.family, f.rel_path, f.line,"
        " f.signature FROM functions f ORDER BY f.rel_path, f.line"
    ).fetchall()
    functions = []
    unmapped = Counter()
    first_copy = {}
    for addr, legions, name, cls, family, rel_path, line, signature in rows:
        if family in SKIPPED_FAMILIES:
            continue
        rule = rule_for(rel_path)
        if rule is None:
            unmapped[rel_path.rsplit("/", 1)[0]] += 1
            continue
        crate, status, note = rule
        if family in ("container_template", "crt_runtime"):
            crate, status, note = "replaced", "skip", "Rust std replaces it"
        elif status == "todo" and TEXT_FORMAT_METHODS.search(name):
            status, note = "skip", "text source format (admin tools)"
        elif status == "todo" and not legions and ADMIN_LOADER.search(name):
            status, note = "skip", "text source loader, admin build only"
        elif status == "todo" and (name, signature) in first_copy:
            status, note = "skip", f"inline copy of {first_copy[(name, signature)]}"
        first_copy.setdefault((name, signature), addr)
        functions.append(
            dict(addr=addr, legions=legions or "", name=name, cls=cls or "", family=family,
                 path=rel_path, line=line, crate=crate, status=STATUS[status], note=note)
        )
    if unmapped:
        sys.exit("unmapped source directories:\n" + "\n".join(f"  {n:5d} {d}" for d, n in unmapped.most_common()))
    return functions


def progress(counter):
    total = sum(counter.values())
    done = counter["x"]
    return total, done, counter["~"], counter["-"], total - done - counter["~"] - counter["-"]


def render_crate(crate, title, functions):
    counter = Counter(f["status"] for f in functions)
    total, done, partial, skip, todo = progress(counter)
    lines = [
        f"# {crate}: {title}",
        "",
        f"{total} retail functions: {done} done, {partial} partial, {todo} to do, {skip} skipped.",
        "Legend and workflow: [README](README.md).",
        "",
    ]
    by_file = defaultdict(list)
    for f in functions:
        by_file[f["path"]].append(f)
    for path, file_functions in by_file.items():
        c = Counter(f["status"] for f in file_functions)
        lines.append(f"## `{path}` ({c['x']}/{len(file_functions)})")
        lines.append("")
        current = None
        for f in file_functions:
            cls = f["cls"] or "(free functions)"
            if cls != current:
                if current is not None:
                    lines.append("")
                lines.append(f"### {cls}")
                current = cls
            addresses = f["addr"] + (f" · legions {f['legions']}" if f["legions"] else "")
            note = f" — {f['note']}" if f["note"] else ""
            lines.append(f"- [{f['status']}] `{f['name']}` · {addresses}{note}")
        lines.append("")
    return "\n".join(lines)


def render_formats(index, db_marks):
    db_marks = {k: v for k, v in db_marks.items() if k.startswith("DB_TYPE_")}
    csv_path = Path(index).parent / "ac2_db_type_catalog.csv"
    rows = list(csv.DictReader(open(csv_path, newline="", encoding="utf-8")))
    counter = Counter()
    lines = []
    for row in rows:
        status, note = db_marks.get(row["db_type"], (" ", ""))
        counter[status] += 1
        records = sum(int(row[k] or 0) for k in ("portal_records", "cell_records", "local_records", "highres_records"))
        unpack = f" · `{row['unpack_functions']}`" if row["unpack_functions"] else ""
        note = f" — {note}" if note else ""
        lines.append(
            f"- [{status}] `{row['db_type']}` {row['value']} · {row['classes'] or '?'} · "
            f"{row['did_ranges']} · {records} records{unpack}{note}"
        )
    total, done, partial, skip, todo = progress(counter)
    head = [
        "# Data formats",
        "",
        "Every `DBObj` type the client registers (`MasterDBMap::InitDBTypeDef_Internal`), with its",
        "decoding class, DID ranges, record count in the Legions DATs and unpack function. A format is",
        "done when its `StreamPack`/`UnPack` decodes every record the DATs hold.",
        "",
        f"{total} formats: {done} done, {partial} partial, {todo} to do, {skip} skipped.",
        "",
    ]
    return "\n".join(head + lines) + "\n", counter


def render_readme(summary, formats_counter):
    lines = [
        "# Retail behaviour catalog",
        "",
        "Every function of the Asheron's Call 2 admin client (build 2.0.20.7148, the PDB-named",
        "decompile under `decompiled-pdb/admin7148`), grouped by the AC2_Rusted crate that owns it.",
        "Work goes down this list: port a function from the decompile, cite it in the code, mark it.",
        "",
        "Generated by `scripts/retail_catalog.py` from the decompile index; statuses are kept across",
        "rebuilds. Function names and addresses are catalogue keys only; no decompiled code is here.",
        "",
        "## Legend",
        "",
        "- `[ ]` to do",
        "- `[x]` done: behaviour matches the decompile, cited in the code, covered by a test",
        "- `[~]` partial: the note says what is missing",
        "- `[-]` skipped: admin tooling, third-party code or a container Rust replaces; the note says why",
        "",
        "Each line is `` `Class::Method` · admin address · legions address``. A line without a legions",
        "address had no match in the Legions DVD client: often admin-only code, sometimes a function the",
        "matcher missed or that changed between builds. Read the function at the",
        "`rel_path`/line the index gives (`INDEX/ac2client_index.sqlite`). Where Ghidra dropped x87",
        "float math, read `decompiled-recovery7148/recovery/exports/ac2client.exe/functions/<address>.c`",
        "or disassemble the admin exe.",
        "",
        "## Marking",
        "",
        "```sh",
        "scripts/retail_catalog.py mark done 'CDayDesc::GetLighting'",
        "scripts/retail_catalog.py mark partial 0058c8c0 --note 'no cloud shadow UVs'",
        "scripts/retail_catalog.py mark skip 'FrillRender::*' --note 'why'",
        "scripts/retail_catalog.py status",
        "```",
        "",
        "## Progress",
        "",
        "| Crate | Functions | Done | Partial | To do | Skipped |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for crate, title, counter in summary:
        total, done, partial, skip, todo = progress(counter)
        lines.append(f"| [{crate}]({crate}.md) | {total} | {done} | {partial} | {todo} | {skip} |")
    total, done, partial, skip, todo = progress(formats_counter)
    lines += [
        "",
        f"Data formats ([data-formats.md](data-formats.md)): {total} `DBObj` types, {done} done,"
        f" {partial} partial, {todo} to do.",
        "",
    ]
    return "\n".join(lines)


def build(index, overrides=None, db_overrides=None):
    marks = read_marks()
    shown = read_rendered()
    functions = load_functions(index)
    # A checkbox that differs from what the last build wrote was edited by hand.
    for f in functions:
        current = marks.get(f["addr"], (f["status"], ""))[0]
        if f["addr"] in shown and shown[f["addr"]] != current:
            marks[f["addr"]] = (shown[f["addr"]], marks.get(f["addr"], ("", ""))[1])
    for key, status in shown.items():
        if key.startswith("DB_TYPE_") and status != marks.get(key, (" ", ""))[0]:
            marks[key] = (status, marks.get(key, ("", ""))[1])
    marks.update(overrides or {})
    marks.update(db_overrides or {})
    for f in functions:
        if f["addr"] in marks:
            f["status"], note = marks[f["addr"]]
            f["note"] = note or f["note"]
    db_marks = marks
    OUT.mkdir(parents=True, exist_ok=True)
    summary = []
    for crate, title in CRATES:
        members = [f for f in functions if f["crate"] == crate]
        (OUT / f"{crate}.md").write_text(render_crate(crate, title, members), encoding="utf-8")
        summary.append((crate, title, Counter(f["status"] for f in members)))
    formats, formats_counter = render_formats(index, db_marks)
    (OUT / "data-formats.md").write_text(formats, encoding="utf-8")
    (OUT / "README.md").write_text(render_readme(summary, formats_counter), encoding="utf-8")
    write_marks(marks)
    return summary


def resolve(index, targets):
    con = sqlite3.connect(index)
    found = []
    for target in targets:
        if re.fullmatch(r"[0-9a-fA-F]{8}", target):
            rows = con.execute("SELECT admin_address FROM functions WHERE admin_address=?", (target.lower(),))
        elif target.endswith("::*"):
            rows = con.execute("SELECT admin_address FROM functions WHERE class=?", (target[:-3],))
        else:
            rows = con.execute("SELECT admin_address FROM functions WHERE name=?", (target,))
        addresses = [a for (a,) in rows]
        if not addresses:
            sys.exit(f"no retail function matches {target!r}")
        found += addresses
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["build", "mark", "status"])
    parser.add_argument("args", nargs="*")
    parser.add_argument("--note", default="")
    parser.add_argument("--index")
    opts = parser.parse_args()
    index = index_path(opts.index)
    if opts.command == "mark":
        if len(opts.args) < 2 or opts.args[0] not in STATUS:
            sys.exit("mark STATUS NAME_OR_ADDRESS...")
        status = STATUS[opts.args[0]]
        db_targets = [t for t in opts.args[1:] if t.startswith("DB_TYPE_")]
        targets = [t for t in opts.args[1:] if not t.startswith("DB_TYPE_")]
        overrides = {a: (status, opts.note) for a in resolve(index, targets)} if targets else {}
        db_overrides = {t: (status, opts.note) for t in db_targets}
        build(index, overrides, db_overrides)
        print(f"marked {len(overrides) + len(db_overrides)} as [{status}]")
    elif opts.command == "build":
        build(index)
    summary = build(index) if opts.command == "status" else None
    if opts.command == "status":
        for crate, _, counter in summary:
            total, done, partial, skip, todo = progress(counter)
            print(f"{crate:14} {done:5}/{total - skip:<5} done  {partial:4} partial  {todo:5} to do  {skip:5} skipped")


if __name__ == "__main__":
    main()
