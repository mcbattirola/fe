# file explorer

(WIP)

fe is a simple file explorer.

It allows users to create custom commands for files and dirs, and runs then via the context menu on right click.

## Config

The first time it runs, fe will create a config file in `$HOME/.config/fe`. There, you can add custom commands.

### Custom Commands

See the [example](./example/) dir for a valid config file that adds two custom commands, `Open URL` and `Print File Name`, and their respectives scripts.

Commands can either be for files or directories, and they will appear when you right click one of those kinds of entries in the file system, given the extensions match what you added in the `extensions` field in the case of file commands:

```toml
# this will only show up in the context menu
# when you right click a .url file.
# Since `clickable` is true, it highlight .url files like executables,
# and right clicking it will invoke the script.
[[commands.file]]
name = "Open URL"
interpreter = "python"
script = "open_url.py"
extensions = ["url"]
args = []
clickable = true
```

When you select the custom command, the `interpreter` will be called with your script's path as argument, and the path of the file you selected as the first argument.

If you want to pass the path to a program without an interpreter, just use the program as the interpreter with an empt script:

```toml
[[commands.dir]]
name = "Open VSCode"
interpreter = "path/to/vscode"
script = ""
```

This will make the item `Open VSCode` appear when you right click any dir, and invoke `path/to/vscode current/path`.

Make sure you use double backlashes on Windows paths.

## Shortcuts

- Ctrl + L: focus on path bar.
- Ctrl + F: focus on search bar.
- Ctrl + O: go back to previous dir.
- Ctrl + B: favorite current path.
- Ctrl + N: open new file dialog.
- Ctrl + R: reload current dir.

## Config

## TODO

- [ ] Config file
  - [ ] Style and colors
- [ ] UI improvements
  - [ ] Hover effect
  - [ ] Mouse cursor on hover
  - [ ] Move favorites
  - [ ] Hightlight files as user types
  - [ ] Navigate files via keyboard
- [ ] Data persistence
  - [x] Pinned / Favorites
    - [ ] make desktop and home part of the favorites (not hardcoded)
  - [ ] Subscribe to changes between multiple processes
- [ ] Tabs
  - [ ] Ctrl + T -> new tab
  - [ ] Ctrl + [1-9] -> go to tab [1-9]
- [ ] Context menu
  - [ ] Copy/paste files
  - [ ] File Properties
- [ ] Custom user commands
  - [ ] Allow users to create submenu inside their command
  - [ ] Print command's stdout/stderr in the screen
- [ ] Drag and drop files from/to explorer
- [ ] Expand env vars in search and path bar
- [ ] Files view
  - [ ] Show additional info
    - [ ] Perms
  - [ ] Add cards view with image previews
- [ ] Check actions permissions
  - [ ] Disable file operations if no perm, etc
- [ ] Handle symlinks properly
  - [ ] Show symlink as dir or file, based on the pointed file
  - [ ] Show visually that file is a symlink
- [ ] Handle signals
- [ ] Improve Test coverage
- [ ] Remove unwraps
