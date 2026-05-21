# Contributing

When contributing to this repository, please first discuss the change you wish to make via issue, email, or any other method with the owners of this repository before making a change.

## Development

1. Fork this repository.

You can fork this repo by clicking the fork button in the top right corner of this page.

2. Clone the repository.

```bash
git clone https://github.com/KurutoDenzeru/envexa.git
```

3. Install dependencies using uv.

```bash
uv sync
```

4. Start the MCP server.

```bash
uv run python -m src.server
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

Tests are run via uv. You can run all tests from the root of the repository.

```bash
uv run python -c "from src.scanner import run_scan; run_scan('all')"
```

Please ensure that the tests are passing when submitting a pull request. If you're adding new components or features, creating tests would be appreciated.
