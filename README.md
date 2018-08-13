Dip
===

[![](https://img.shields.io/badge/rewritten-in%20rust-%23dea584.svg)](https://github.com/ansuz/RIIR)

Configurable webhook server.

Set up some kind of directory structure like this:

```
root/
  - config.toml
  - hooks/
      - website.com
```

where every file in the `hooks` subdirectory is a TOML document of the following format:

```toml
# The handler to use. This must match one of the handlers defined in the
# handlers directory or one of the builtins such as "github"
type = "github"

# wip lol
```

Then run the `dip` binary with the option `-d <root>` where `root` points to the root directory you made above.

Contact
-------

Author: Michael Zhang

License: MIT
