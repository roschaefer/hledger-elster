# --force overwrites existing commit evidence

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_commit_evidence
```

```scrut
$ write_file ../commit-evidence.pdf <<'EOF'
> previous evidence
> EOF
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf --force
```

```scrut
$ grep --quiet --fixed-strings "$(git rev-parse HEAD)" ../commit-evidence.pdf
```

```scrut
$ ! grep --quiet --fixed-strings "previous evidence" ../commit-evidence.pdf
```
