# shellcheck shell=bash
# See README.md in this directory.
scenario_commit_evidence() {
  new_git_repository
  write_file journal.journal <<'EOF'
account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
EOF
  write_file elster.toml <<'EOF'
[euer.home_office_pauschale]
enabled = false
EOF
  git add .
  git commit --quiet --message "scenario state"
}
