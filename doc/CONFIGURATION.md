# Configuring `twm`

Your config file should be located at $XDG_CONFIG_HOME/twm/twm.yaml (default: ~/.config/twm/twm.yaml).

`twm` has sensible defaults if you don't want to deal with a config file just yet, but it will definitely not suffice for everybody's directory structure.


## Example `twm` config

Here is an example configuration with all configuration options set:
```yaml
# ~/.config/twm/twm.yaml

search_paths:  # directories we should begin searching for workspaces in. i just use home. shell expansion is supported
    - "~"      # default: ["~"]

exclude_path_components:  # search branches will be pruned the path being explored contains any of these components
  - .git
  - .direnv
  - node_modules
  - venv
  - target

max_search_depth: 5  # how deep we should search for workspaces (default: 3)
session_name_path_components: 3    # how many parts of the workspace path to use in generating the session name (to avoid collisions) (default: 2)

workspace_definitions:             # our list of workspaces, each with different properties
    - name: python                 # they all have to be named
      has_any_file:                # if any file matches this list, we consider it a match, since its "has_any_file"
        - requirements.txt         # more complex matching isn't implemented currently
        - setup.py
        - pyproject.toml
        - Pipfile
      default_layout: python-dev   # the hierarchy for how a layout gets chosen is user opts to select manually > local layout > default for workspace type

    - name: node                   # the order of these definitions matters - if a directory matches multiple, the first one wins
      has_any_file:
        - package.json
        - yarn.lock
        - .nvmrc
      default_layout: node-dev

    - name: rust
      has_any_file:
        - Cargo.toml
        - Cargo.lock
      default_layout: rust-dev

    - name: other
      has_any_file:
        - .git
        - flake.nix
        - .twm.yaml

layouts:                           # our list of layouts just have names and a list of commands. the command get sent directly with tmux send-keys
    - name: python-dev             # i chose not to use any custom configuration becuase that would be a lot of work to basically maintain a subset of possible functionality
      commands:
        - tmux split-window -h
        - tmux resize-pane -x 80
        - tmux split-window -v
        - tmux send-keys -t 0 'nvim .' C-m

    - name: rust-dev
      commands:
        - tmux split-window -h
        - tmux resize-pane -x 80
        - tmux select-pane -t 0
        - tmux send-keys -t 1 'cargo watch -x test -x run' C-m
        - nvim .
```

### Example local config

**Note:** `twm` will search up the directory tree for a `.twm.yaml` file. If it finds one, it will be used instead of the default for your workspace type. This is useful for worktrees, where you may not want to check in the layout to source control, but have the same layout apply to all branches. You can simply put `.twm.yaml` in the worktree root to achieve this.

```yaml
# ~/dev/random/project/dir/.twm.yaml

layout:
  name: layout-for-this-project
  commands:
    - tmux split-window -h
    - tmux split-window -h
    - tmux split-window -h
    - tmux split-window -h
```
