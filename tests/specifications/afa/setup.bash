# shellcheck shell=bash
# See README.md in this directory.
scenario_afa() {
  write_file elster.toml <<'EOF'
[euer.home_office_pauschale]
enabled = false
EOF
}
