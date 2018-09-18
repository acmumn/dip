# Built-in Handlers

Dip comes with two built-in handlers: Bash commands and Github webhooks.

## Bash commands

Bash commands are invoked with `type = "command"`. It will run whatever is specified in the `command` field using `bash` (assuming it exists on the system).

An full config using a `command` handler follows:

```toml
[[handlers]]
type = "command"

# the command to run using bash
command = "echo hi"
```

## Github webhooks

For Github webhooks, dip will verify the webhook (by using the provided secret) and then clone the repository that the webhook was attached to into the temporary directory created for that specific invocation.

This handler isn't very useful by itself, so it's a good idea to follow up this handler with some `command`s or other handlers to use the newly cloned repository (for example, build or deploy).

An full config using a `github` handler follows:

```toml
[[handlers]]
type = "github"

# webhook secret
secret = "**************"

# turn off secret verification (false by default)
disable_hmac_verify = false

# path to clone to
path = "."
```
