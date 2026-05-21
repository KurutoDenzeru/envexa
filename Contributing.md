# Contributing

## Development

1. Fork this repository.

2. Clone the repository.

```bash
git clone https://github.com/KurutoDenzeru/envexa.git
```

3. Build the project.

```bash
cargo build
```

4. Test the MCP server.

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"brew"}}}\n' | cargo run | python3 -c "
import sys, json
for line in sys.stdin:
    d = json.loads(line)
    if d.get('result',{}).get('content'):
        print(d['result']['content'][0]['text'])
"
```

## Project Layout

```
src/
├── main.rs           # Entry point + tokio stdin/stdout loop
├── transport.rs      # JSON-RPC MCP protocol (hand-rolled)
├── server.rs         # Tool/prompt/resource dispatch
├── scanner.rs        # Scan orchestration + formatting + cache
└── toolchains/
    ├── mod.rs        # ScanResult type + concurrent scan_all()
    ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
    ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
```

## Commit Convention

Before you create a Pull Request, please check whether your commits comply with
the commit conventions used in this repository.

When you create a commit we kindly ask you to follow the convention
`category: message` in your commit message while using one of
the following categories:

- `feat / feature`: all changes that introduce completely new code or new
  features
- `fix`: changes that fix a bug (ideally you will additionally reference an
  issue if present)
- `refactor`: any code related change that is not a fix nor a feature
- `docs`: changing existing or creating new documentation (i.e. README)
- `build`: all changes regarding the build of the software, changes to
  dependencies or the addition of new dependencies
- `test`: all changes regarding tests (adding new tests or changing existing
  ones)
- `ci`: all changes regarding the configuration of continuous integration (i.e.
  github actions, ci system)
- `chore`: all changes to the repository that do not fit into any of the above
  categories

  e.g. `feat: add new toolchain scanner for pnpm`

If you are interested in the detailed specification you can visit
https://www.conventionalcommits.org/ or check out the
[Angular Commit Message Guidelines](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines).

## Testing

Build and test the full scan:

```bash
cargo build
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"all"}}}\n' | cargo run > /dev/null && echo "PASS"
```

Please ensure the server compiles and responds to MCP requests correctly when submitting a pull request.
