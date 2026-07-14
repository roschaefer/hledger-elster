# Command-line help

`hledger-elster` has one implicit default command (report generation, driven by
flags) and explicit subcommands (like `init-config`). All subcommands must be
discoverable from `--help`, so a user is not left guessing that `init-config`
exists.

```gherkin
Feature: Command-line help

  Scenario: --help lists all subcommands
    When I run "hledger-elster --help"
    Then stdout should contain:
      """
      Commands:
        init-config  Write a default hledger-elster TOML config file
      """
```
