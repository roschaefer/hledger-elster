# shellcheck shell=bash

set -o pipefail

# Scrut runs commands in the C locale; hledger needs UTF-8 to read the journals.
# macOS has no C.UTF-8.
if [[ $(uname) == Darwin ]]; then
  export LC_ALL=en_US.UTF-8
else
  export LC_ALL=C.UTF-8
fi

SPECIFICATION_WORKING_DIRECTORY=$PWD

# Start every scenario in a new empty directory, scenario.XXXXXX/work, so files
# "outside the repository" (../) stay inside the scenario too. Nothing is
# deleted, so sourcing this file to try a scenario by hand is safe.
new_scenario() {
  cd "$(mktemp -d "$SPECIFICATION_WORKING_DIRECTORY/scenario.XXXXXX")" || return
  mkdir work && cd work || return
}

# build_scenario FUNCTION starts a scenario from the state that FUNCTION, from
# the document's setup.bash, builds. Chapters whose scenarios share their
# setup keep it there, so every document in the folder starts the same way.
# FUNCTION runs with errexit, so its commands need no && chain: Scrut saves
# functions with declare -f, and bash 5.2 prints an && after a heredoc on a
# line of its own, which it then cannot parse.
build_scenario() {
  new_scenario || return
  # shellcheck disable=SC1091
  source "$TESTDIR/setup.bash" || return
  (set -o errexit; "$1")
}

# write_file PATH <<'EOF' ... EOF
write_file() {
  mkdir -p "$(dirname "$1")" && cat >"$1"
}

new_git_repository() {
  git init --quiet &&
    git config user.email specifications@example.invalid &&
    git config user.name Specifications &&
    git config commit.gpgsign false
}

# xlsx_sheet WORKBOOK SHEET prints a workbook tab as ;-separated CSV. All
# cells this tool writes are strings, so the shared-strings table and inline
# strings are all there is to read.
xlsx_sheet() {
  python3 -I - "$1" "$2" <<'PYTHON'
import csv, re, sys, zipfile
import xml.etree.ElementTree as ET

ns = {
    "m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
}
workbook_path, sheet_name = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(workbook_path) as z:
    workbook = ET.fromstring(z.read("xl/workbook.xml"))
    rels = ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))
    targets = {r.get("Id"): r.get("Target") for r in rels}
    sheet = next(s for s in workbook.iterfind("m:sheets/m:sheet", ns) if s.get("name") == sheet_name)
    target = targets[sheet.get("{%s}id" % ns["r"])].lstrip("/")
    target = target if target.startswith("xl/") else "xl/" + target
    shared = []
    if "xl/sharedStrings.xml" in z.namelist():
        for si in ET.fromstring(z.read("xl/sharedStrings.xml")).iterfind("m:si", ns):
            shared.append("".join(t.text or "" for t in si.iter("{%s}t" % ns["m"])))
    rows = {}
    for row in ET.fromstring(z.read(target)).iterfind("m:sheetData/m:row", ns):
        cells = {}
        for c in row.iterfind("m:c", ns):
            column = 0
            for ch in re.match(r"[A-Z]+", c.get("r"))[0]:
                column = column * 26 + ord(ch) - 64
            if c.get("t") == "s":
                value = shared[int(c.find("m:v", ns).text)]
            elif c.get("t") == "inlineStr":
                value = "".join(t.text or "" for t in c.iter("{%s}t" % ns["m"]))
            else:
                v = c.find("m:v", ns)
                value = v.text if v is not None else ""
            cells[column] = value
        rows[int(row.get("r"))] = [cells.get(i, "") for i in range(1, max(cells, default=0) + 1)]
rows = [rows.get(i, []) for i in range(1, max(rows, default=0) + 1)]
width = max((len(r) for r in rows), default=0)
writer = csv.writer(sys.stdout, delimiter=";", lineterminator="\n")
writer.writerows(r + [""] * (width - len(r)) for r in rows)
PYTHON
}

# The xlsx renders a "# " section marker as a bold row instead of as text.
strip_section_markers() {
  sed 's/^# //' "$1"
}
