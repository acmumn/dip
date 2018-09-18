Dip: Configurable Webhook Server
================================

[![](https://api.travis-ci.org/acmumn/dip.svg?branch=master)](https://travis-ci.org/acmumn/dip)
[![](https://img.shields.io/badge/rewritten-in%20rust-%23dea584.svg)](https://github.com/ansuz/RIIR)

[Documentation](https://acmumn.github.io/dip/)

Express your webhooks in terms of composable blocks such as:

```toml
[[handlers]]
type = "github"
secret = "hunter2"

[[handlers]]
type = "command"
command = "cargo build"
```

Contact
-------

Author: Michael Zhang

License: MIT
