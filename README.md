# youarel
A simple, blazing fast URL shortener serving over HTTPS or HTTP, along with a clean frontend.

Youarel is built on the `axum` web framework and in turn `tokio` and `hyper`, with the `sled` embedded database being used for persistence.

Use `--default-features=false --features = "axum"` to serve HTTP (no TLS).



```
Usage: youarel [OPTIONS]

Options:
  -v, --verbose...           More output per occurrence
  -q, --quiet...             Less output per occurrence
      --compact              Use compact formatting for log messages
      --pretty               Whether log messages should be pretty printed. The --compact option will override this if set
  -a, --address <ADDRESS>    The address to bind to [default: ::1]
  -p, --port <PORT>          The port to bind to [default: 3000]
  -l, --length <LENGTH>      The number of base64 characters used in shortened URLs. A smaller number increases the chances of collisions [default: 8]
  -d, --db <DB>              Path to the database root. Defaults to the appropriate data directory according to XDG/Known Folder/Standard directories specifications based on OS [default: /home/thor/.local/share/youarel]
  -k, --key <KEY>            TLS private key in DER format
  -c, --cert <CERT>          TLS certificate in DER format
      --stateless-retry      Enable stateless retries
      --hostname <HOSTNAME>  [default: localhost]
  -h, --help                 Print help
  -V, --version              Print version
  ```
