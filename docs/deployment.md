# Deployment

## Download Dip

The easiest way to get a copy of Dip is to download a binary from the [releases page](https://github.com/acmumn/dip/releases).

Alternatively, clone the repository and compile from source.

### HTTP Server setup

Dip runs an HTTP server. By default it serves on port 5000 on all interfaces. Use `--bind` on the executable to change this.

It's recommended to vhost your Dip through a server such as Nginx. Here's a sample Nginx config:

```
location / {
    proxy_set_header X-Real-IP  $remote_addr;
    proxy_set_header X-Forwarded-For $remote_addr;
    proxy_set_header Host $host;
    proxy_pass http://localhost:5000;
}
```

### Root configuration directory

Delegate a directory on your server to act as a root config directory. 

### Systemd

Systemd is useful for keeping Dip running as a daemon, and starting it automatically on startup. Here's a sample systemd config:

```
[Unit]
Description=Configurable Webhook Server

[Service]
User=dip
Environment="RUST_BACKTRACE=1"
ExecStart=/usr/bin/dip --root /etc/dip --bind "127.0.0.1:5000"

[Install]
WantedBy=default.target
```
