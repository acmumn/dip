# Configuration

The configuration directory should contain two subdirectories: `handlers` and `hooks`. The `hooks` directory should contain configuration files for webhooks. These files are defined in [TOML](https://github.com/toml-lang/toml). For example, here is a config file for a website deployment using gutenberg:

```toml
[[handlers]]
type = "github"
secret = "**************"

[[handlers]]
type = "command"
command = "rm -rf /var/www/default/*"

[[handlers]]
type = "command"
command = "gutenberg build --output-dir /var/www/default"
```

If this config file existed at `$DIP_ROOT/hooks/website`, then it would be served from `http://localhost:5000/webhook/website`.

## Config File Format

The hook config should contain an array called `handlers`, which is a sequence of `handler` tables.

#### The `type` key

Each `handler` must contain at least one key required by dip, `type`, which determines what kind of hook will be run.

Dip comes with two built-in handlers:

- [`command`](handlers.html#bash-commands)
- [`github`](handlers.html#github-webhooks)

If you set `type` to one of these values, dip will automatically use the built-in. Otherwise, dip will look into the `handlers` subdirectory for an executable matching the `type` that you specified. For example, if you set `type = "mkdir"`, then it will look for `$DIP_ROOT/handlers/mkdir`, which must be an executable file. It will _not_ run the system `mkdir`. If you want to run the system `mkdir`, use `type = "command"`, so it will run a bash command instead.

## Handler Input and Output

Think of a handler as a function that takes two inputs: a configuration and per-instance data. The configuration is specified in the configuration file, while the per-instance data is specific to that run of the webhook.

For example, suppose we have the following setup:

```toml
[[handlers]]
type = "github"
secret = "hunter2"
```

If a new Github webhook is deployed, then the first config input will be:

```json
{
    "secret": "hunter2"
}
```

This input will be provided to the executable using `--config` in JSON format. For example, if `github` was not a builtin, then a call to the `github` executable might look like:

```bash
/usr/bin/github --config '{"secret":"hunter2"}'
```

The second input that's provided to the handler is information specific to this run. For the first handler in the sequence, this will be data serialized from the HTTP request, and for subsequent handlers, it will be the output of the previous handler. This input is provided through standard input directly as is.

Think of it as a fold over the list of handlers:

```hs
foldl (\input next_handler -> next_handler input) http_data handlers
```

### Environment Variables

Every process spawned by Dip as part of a webhook will have certain variables set to give it information about its environment:

- `DIP_ROOT`: the root config directory for Dip.
- `DIP_WORKDIR`: the temporary directory created for this specific hook invocation.