# gh-dashboard

Github Workflow Dashboard

## Configuration
gh-dashboard looks for configurations in the following locations:
- `./config.toml`
- `<config-dir>/gh-dashboard/config.toml`
  - On linux: `${XDG_CONFIG_PATH:-${HOME}/.config}/gh-dashboard/config.toml`
  - On macOS: `${HOME}/Library/Application Support/gh-dashboard/config.toml`
  - On Windows: `${FOLDERID_RoamingAppData}/gh-dashboard/config.toml`
- `/etc/gh-dashboard.toml`

The repositories to watch are specified in the `repos` section of the configuration file.
Example:
```toml
repos = [
    { owner = "mathieu-lemay", name = "gh-dashboard" },  # branch is "main" by default
    { owner = "mathieu-lemay", name = "pipeline-runner", branch = "master" },
]
```

## Authentication
gh-dashboard needs a GitHub token to access the GitHub API. It will look for a token in the following locations, in that order of precedence:
- The `auth_token` field in the configuration file
- The `GH_DASHBOARD_AUTH_TOKEN` environment variable
- The `GITHUB_TOKEN` environment variable
- The `gh` CLI
