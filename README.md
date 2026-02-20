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
    { owner = "octocat", name = "hello-world" },
    # Optionally specify the branch. Default is all branches.
    { owner = "octocat", name = "hello-world", branch = "main" },
    # Optionally specify the number of workflows to fetch. Default is 1.
    { owner = "octocat", name = "hello-world", count = 5 },
    # Optionally specify the actor. Default is <all>.
    { owner = "octocat", name = "hello-world", actor = "octocat" },
]
```

## Usage
Select your workflow with <up>/<down> or <j>/<k>
Press <enter> to open the workflow in your browser
Press <d> to see the details, and <esc> to close
Press <q> to quit

## Authentication
gh-dashboard needs a GitHub token to access the GitHub API. It will look for a token in the following locations, in that order of precedence:
- The `auth_token` field in the configuration file
- The `GH_DASHBOARD_AUTH_TOKEN` environment variable
- The `GITHUB_TOKEN` environment variable
- The `gh` CLI
