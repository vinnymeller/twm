# Configuring `twm`

Your config file should be located at $XDG_CONFIG_HOME/twm/twm.yaml (default: ~/.config/twm/twm.yaml).

`twm` has sensible defaults if you don't want to deal with a config file just yet, but it will definitely not suffice for everybody's directory structure.


## Configuration options

- `search_paths`: a list of strings representing directories to search for workspaces in. Defaults to `~`. Shell expansion is supported.
- `exclude_path_components`: a list of strings representing folders that will not be searched when searching for workspaces
- `max_search_depth`: integer, how many directories deep to search your `search_paths` for workspaces
- `session_name_path_components`: integer, how many directories deep to use by default when generating the name of your tmux session. e.g. if your workspace is `/home/vinny/dev/rust/twm` and `session_name_path_components` is 2, the tmux session will be named `rust/twm`. In the case of name conflicts (e.g. I also have a workspace in `/home/vinny/cache/rust/twm`), the new session would be called `cache/rust/twm`.
- `workspace_definitions`: optional, a list of workspace definitions. if no workspaces are defined, git repositories and directories with a `.twm.yaml` config are considered a workspace by default. other than `name` and `default_layout`, the other properties all configure workspace match conditions. any number of conditions can be used in combination with each other, e.g. `has_all_files: [ ".git", "requirements.txt" ]` and `missing_all_files: [ "pyproject.toml", "Pipfile", "poetry.lock"]` can be used together. each workspace definition has the following properties:
  - `name`: string, the name describing the workspace type. must be unique.
  - `has_any_file`: optional list of strings, tells twm to only consider a directory to be a workspace of this type if at least one filename in this list is present
  - `has_all_files`: optional list of strings, tells twm to only consider a directory to be a workspace of this type if all filenames in this list are present
  - `missing_any_file`: optional list of strings, tells twm to only consider a directory to be a workspace of this type if at least one of the filenames in this list is missing
  - `missing_all_files`: optional list of strings, tells twm to only consider a directory to be a workspace of this type if all the filenames in this list are missing
  - `default_layout`: optional string, the name of the layout to open this workspace with if the user does not select a layout manually. must match a defined layout name
- `layouts`: optional, a list of layout definitions. each layout definition has the following properties:
  - `name`: string, the name of the layout. must be unique
  - `inherits`: optional list of strings, the names of layouts to "inherit" from, i.e. run the commands listed in that layout before the `commands` defined in this layout. useful for setting up base layouts to be used in specific development environments. e.g. it might set up one big pane on the left, with two horizontally split panes off to the right, with nothing running inside them. then the `commands` of layouts that inherit from the base can start environment-specific processes.
  - `commands`: list of strings, the commands to run to set up the layout. these are sent directly to tmux with `tmux send-keys`. generally one will use tmux commands to configure the panes and windows, and then open programs in specific places.


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
session_name_path_components: 3    # how many parts of the workspace path to use in generating the session name by default
                                   # if you attempt to open two separate workspaces that would generate the same session name,
                                   # this value will be incremented until a unique session name is found

follow_links: false                # whether to follow symlinks when searching for worksapces (default: true)

workspace_definitions:             # our list of workspaces, each with different properties
    - name: python                 # they all have to be named
      has_any_file:                # if any file matches this list, we consider it a match, since its "has_any_file"
        - requirements.txt         # more complex matching isn't implemented currently
        - setup.py
        - pyproject.toml
        - Pipfile
      default_layout: python-dev   # the hierarchy for how a layout gets chosen is user opts to select manually > local layout > default for workspace type

    - name: go
      has_all_files:               # if all files match this list, we consider it a match, since its "has_all_files"
        - go.mod
        - go.sum

    - name: docker-compose         # you can also combine conditions, as in this example, a docker-compose workspace is matched only if we have *any* of the docker-compose files and both `.git` folder and a `Dockerfile`
      has_any_file:
        - docker-compose.yaml
        - docker-compose.yml
      has_all_files:
        - Dockerfile
        - .git

    - name: node                   # the order of these definitions matters - if a directory matches multiple, the first one wins
      has_any_file:
        - package.json
        - yarn.lock
        - .nvmrc
      default_layout: node-dev

    - name: catchall               # without any conditions, all directories will match this wworkspace
      default_layout: catchall-dev # this is the default layout for this workspace type

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

    - name: catchall-dev
      commands:
        - nvim .

    - name: split-bottom-panes
      commands:
        - tmux split-window -v
        - tmux resize-pane -y 20
        - tmux split-window -h
        - tmux select-pane -t 0

    - name: syslog-monitor
      inherits:
        - split-bottom-panes       # you can also inherit layouts, which will just run the commands from the inherited layout before running your specified commands. these can be nested arbitrarily. this is useful when you have many layouts that should look similar but, for example, have different commands they should run
      commands:
        - tmux send-keys -t 1 'tail -f /var/log/syslog' C-m
        - tmux send-keys -t 2 'journalctl -f' C-m
```

### Example local config

**Note:** `twm` will search up the directory tree for a `.twm.yaml` file. If it finds one, it will be used instead of the default for your workspace type. This is useful for worktrees, where you may not want to check in the layout to source control, but have the same layout apply to all branches. You can simply put `.twm.yaml` in the worktree root to achieve this.

Local configuration files can also inherit from globally-defined layouts.

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
