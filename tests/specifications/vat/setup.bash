# shellcheck shell=bash
# See README.md in this directory.
scenario_vat() {
  write_file elster.toml <<'EOF'
[euer.home_office_pauschale]
enabled = false
EOF
}
