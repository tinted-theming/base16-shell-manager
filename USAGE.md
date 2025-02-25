# Usage

There are many specific usage situations which this document will cover.
For more general usage, look at the [Usage section] in [README.md].

## Table of contents

- [Shell completions](#shell-completions)
- [How it works](#how-it-works)
- [Sourcing scripts that set environment variables](#sourcing-scripts-that-set-environment-variables)
- [Use your own schemes](#use-your-own-schemes)
- [Scripting](#scripting)
- [shell](#shell)
- [Vim or Neovim](#vim-or-neovim)
- [tmux](#tmux)
- [fzf](#fzf)
- [bat](#bat)
- [qutebrowser](#qutebrowser)
- [rofi](#rofi)
- [dunst](#dunst)
- [delta](#delta)

## Shell completions

You can generate shell completions with the `generate-completion`
subcommand, source the generated file in your shell startup file (`*rc`)
and completions will exist for `tinty`. Have a look at the [README CLI
section] for more information about the command usage.

```sh
tinty generate-completion zsh > path/to/tinty-zsh-completion.sh
```

In your startup file (`*rc`) add the following:

```sh
source path/to/tinty-zsh-completion.sh
```

Completion will not appear to be working if you've aliased `tinty` as explained
[here](#sourcing-scripts-that-set-environment-variables). In order to verify the functionality, be sure to run the actual `tinty` binary itself which is usually `~/.cargo/bin/tinty`

### Completions in the repo

A shell completion generation via `tinty` doesn't include any dynamic
values, meaning scheme names (such as `base16-ocean`) won't be completed
typing `tinty apply base`. We've created modified completion script
files for this reason so it can also generate the scheme names.
Currently this is only supported for the `bash` completion file, but we
plan to include the other shells too. You can find these completion
files in [contrib/completion].

## How Tinty Works

There are some concepts which some of the following instructions will
make use of.

### current_scheme

`~/.local/share/tinted-theming/tinty/current_scheme` is a file which
contains the name of the system prefix and name of the scheme
(`<system>-<scheme_name>`), eg: `base16-mocha`. Whenever a scheme is
applied through Tinty, this file is updated.

### What does `tinty apply` do?

1. `tinty apply` sets `current_scheme`.
1. It then runs through the `[[items]]` in your `config.toml
1. For each `[[items]]`, or theme template, it copies the relevant theme
   to `~/.local/shared/tinted-theming/tinty` and executes the `hook`
   property of the `[[items]]`. `%f` is a template variable that can be
   used in the hook, e.g., `hook = "cp -f %f
   ~/.config/alacritty/colors.yml"`.

`tinty apply` can also be used without a theme template. The
`config.hooks` property will execute the array of hooks regardless of
template. This can be useful for when an application is using base16 (or
another supported system) and you just want to write `tinty current` to
a file.

Once you understand the functionality and the lifecycle, you can do a
lot with it.

## Sourcing scripts that set environment variables

General `config.toml` hooks can be used to source and execute scripts,
but due to the way shell sub-processes work, the scripts sourced by
Tinty can't set your current shell session's environment variables.
There is a workaround for this specific issue.

1. Create a function which executes `tinty` with all the same arguments
2. Check for any `*.sh` files in the active Tinty themes directory
3. Source any matching files

The following script does that. Add it to your shell startup file (`*rc`):

```sh
# Tinty isn't able to apply environment variables to your shell due to
# the way shell sub-processes work. This is a work around by running
# Tinty through a function and then executing the shell scripts.
tinty_source_shell_theme() {
  newer_file=$(mktemp)
  tinty $@
  subcommand="$1"

  if [ "$subcommand" = "apply" ] || [ "$subcommand" = "init" ]; then
    tinty_data_dir="${XDG_DATA_HOME:-$HOME/.local/share}/tinted-theming/tinty"

    while read -r script; do
      # shellcheck disable=SC1090
      . "$script"
    done < <(find "$tinty_data_dir" -maxdepth 1 -type f -name "*.sh" -newer "$newer_file")

    unset tinty_data_dir
  fi

  unset subcommand
}

if [ -n "$(command -v 'tinty')" ]; then
  tinty_source_shell_theme "init" > /dev/null

  alias tinty=tinty_source_shell_theme
fi
```

**Note:** Make sure to swap out `$tinty_data_dir` with the path to your custom data
directory if you don't use the default of Tinty.
Tinty stores themes to `$XDG_DATA_HOME` based on [XDG Base Directory specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) by default.

## Use your own schemes

To use your own schemes, create a
`custom-schemes/<scheme_system>/your-scheme-name.yaml` file in your
data-dir (Run `tinty config --data-dir-path` to get the path to your
data-dir) - where `<scheme_system>` is the system you use. Currently we
support scheme_system `base16` and `base24`. After you've added your
scheme, make sure it exists correctly by running `tinty list
--custom-schemes`. If you do not see it listed, something is wrong and
Tinty will not apply it.

If everything works as expected, `tinty apply
base16-your-scheme-name.yaml` should apply your scheme.

```sh
mkdir "$(tinty config --data-dir-path)/custom-schemes/base16"
cp path/to/your/base16-your-scheme.yaml "$(tinty config --data-dir-path)/custom-schemes/base16/your-scheme.yaml"
tinty list --custom-schemes # Should show your scheme
tinty apply base16-your-scheme
```

## Terminals

See the [tinted-terminal](https://github.com/tinted-theming/tinted-terminal) repo for a list of supported terminals and their setups.

## Scripting

The `tinty list --json` option outputs a list of all available schemes in JSON format. It provides extensive information
about each scheme, like its human-friendly name, its variant (`light` vs `dark`), its scheme (`base16` vs `base24`),
as well as its color palette in multiple formats. See [this sample object entry](./fixtures/gruvbox-material-dark-hard.json) to see the information available.

Installing [jq] to parse & process the output is recommended.

### Examples

Pretty print:

```sh
tinty list --json | jq
```

List light themes only:

```sh
tinty list --json | jq '.[] | select(.variant == "light") | .id' -r
```

List all themes but grouping light themes and dark themes together:

```sh
tinty list --json | jq 'sort_by(.variant) | reverse' -r
```

Sort themes by background color, from darkest to lightest:

```sh
tinty list --json | jq 'sort_by(.lightness.background)' -r
```

## Shell

When Tinty does not have any `[[items]]` set up in `config.toml`, Tinty
automatically uses [tinted-shell] as a default `[[items]]`. If you have
added anything to `[[items]]`, you must also add [tinted-shell] there
too if you want it to be part of the templates you apply.

Add the following to your `config.toml`:

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-shell"
name = "tinted-shell"
themes-dir = "scripts"
hook = ". %f"
```

[tinted-shell] does set some environment variables in the script, but
it's not necessary for shell styling. If you still want access to these
variables, you will need to execute the [tinted-shell] theme script in
your current shell session. Have a look at [Sourcing scripts that set environment variables]

## Vim or Neovim

There are two different ways you could have vim hooked up to Tiny:

1. Have base16-vim installed in your vim setup and run
   `:colorscheme <THEME_NAME>` when Tinty applies a scheme
2. Have vim source the `.vim` theme file when Tinty applies a scheme

### With base16-vim setup in Vim/Neovim

This doesn't require any setup in your Tinty `config.toml`.

1. Follow the [base16-vim] setup installation instructions.
2. Have vim read the `current_scheme` file and set the vim colorscheme
   with `:colorscheme <DATA_IN_CURRENT_SCHEME>` by adding the following
   to your vim setup. The following Lua or VimScript reads the
   `current_scheme` file when you set your focus to vim and if the theme
   is different to the one already set, it sets it:

**Neovim (Lua)**

```lua
local default_theme = "base16-oceanicnext"

local function get_tinty_theme()
  local theme_name = vim.fn.system("tinty current &> /dev/null && tinty current")

  if vim.v.shell_error ~= 0 then
    return default_theme
  else
    return vim.trim(theme_name)
  end
end


local function handle_focus_gained()
  local new_theme_name = get_tinty_theme()
  local current_theme_name = vim.g.colors_name

  if current_theme_name ~= new_theme_name then
    vim.cmd("colorscheme " .. new_theme_name)
  end
end

local function main()
  vim.o.termguicolors = true
  vim.g.tinted_colorspace = 256
  local current_theme_name = get_tinty_theme()

  vim.cmd("colorscheme " .. current_theme_name)

  vim.api.nvim_create_autocmd("FocusGained", {
    callback = handle_focus_gained,
  })
end

main()
```

**Vim**

```vim
let g:default_theme = "base16-oceanicnext"

function! GetTintyTheme()
  let l:theme_name = system("tinty current &> /dev/null && tinty current")

  if v:shell_error != 0
    return g:default_theme
  else
    return trim(l:theme_name)
  endif
endfunction

function! HandleFocusGained()
  let l:new_theme_name = GetTintyTheme()
  let l:current_theme_name = g:colors_name

  if l:current_theme_name != l:new_theme_name
    execute "colorscheme " . l:new_theme_name
  endif
endfunction

function! Main()
  set termguicolors
  let g:tinted_colorspace = 256
  let l:current_theme_name = GetTintyTheme()

  execute "colorscheme " . l:current_theme_name

  augroup TintyThemeChange
    autocmd!
    autocmd FocusGained * call HandleFocusGained()
  augroup END
endfunction

call Main()
```

### Without base16-vim setup in Vim/Neovim

1. Add base16-vim to Tinty `config.toml`
2. Have vim source the `.vim` colorscheme file when you focus
   Vim/Neovim.

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-vim"
name = "base16-vim"
themes-dir = "colors"
```

**Neovim (Lua)**

```lua
local theme_script_path = vim.fn.expand("~/.local/share/tinted-theming/tinty/base16-vim-colors-file.vim")

local function file_exists(file_path)
  return vim.fn.filereadable(file_path) == 1 and true or false
end

local function handle_focus_gained()
  if file_exists(theme_script_path) then
      vim.cmd("source " .. theme_script_path)
  end
end

if file_exists(theme_script_path) then
  vim.o.termguicolors = true
  vim.g.tinted_colorspace = 256

  vim.cmd("source " .. theme_script_path)

  vim.api.nvim_create_autocmd("FocusGained", {
    callback = handle_focus_gained,
  })
end
```

**Vim**

```vim
let theme_script_path = expand("~/.local/share/tinted-theming/tinty/base16-vim-colors-file.vim")

function! FileExists(file_path)
  return filereadable(a:file_path) == 1
endfunction

function! HandleFocusGained()
  if FileExists(g:theme_script_path)
    execute 'source ' . g:theme_script_path
  endif
endfunction

if FileExists(theme_script_path)
  set termguicolors
  let g:tinted_colorspace = 256
  execute 'source ' . theme_script_path
  autocmd FocusGained * call HandleFocusGained()
endif
```

## tmux

### Add to Tinty config.toml

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-tmux"
name = "tinted-tmux"
# Check if tmux is in use and if it is, reload the config file
hook = "tmux run 2> /dev/null && tmux source-file %f"
themes-dir = "colors"
```

### Without Tinty template setup

If you're using [tinted-tmux] as a [tmux tpm] plugin, you can add add the
following to your `tmux.conf`:

```tmux
run-shell "tmux set-option -g @tinted-color $(tinty current)"
```

And add the following to Tinty `config.toml`:

```toml
hooks = ["tmux source-file /path/to/tmux.conf"]
```

## fzf

### Using shell ANSI colors

There is a special fzf theme file in [tinted-fzf] created for using the
shell's ANSI colors to style fzf. If you are using [tinted-shell] 

### Using theme

Due to the way shell sub-processes work, Tinty isn't able to set shell
environment variables in your session, which is how fzf themes are
applied, so a workaround is needed. 

**1. Add the following to your `config.toml`:**

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-fzf"
name = "tinted-fzf"
themes-dir = "sh"
# Or for fish shell
# themes-dir = "fish"
```

**2. Source the fzf theme script files in your shell**

Have a look at [Sourcing scripts that set environment variables] section. Once you've
implemented that, your fzf theme should be updating correctly when you
run `tinty init` or `tinty apply base16-mocha` or some other theme name.

### Add to Tinty config.toml

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-fzf"
name = "tinted-fzf"
hook = ". %f"
themes-dir = "sh"
# Or for fish shell
# themes-dir = "fish"
```

## Iterm2

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-iterm2"
name = "tinted-iterm2"
hook = "sh %f"
themes-dir = "scripts"
supported-systems = ["base16", "base24"]
```

## bat

[bat has an integration] with [tinted-shell] and another option to allow
ANSI colors to be used. The available `bat` theme names for this are
`base16-256` and `ansi`.

- `bat --theme="base16-256"` if you're using the default Tinty or
  [tinted-shell] with Tinty.
- `bat --theme="ansi"` if you're using another shell template theme with
  Tinty.

Set the alias in your `.*rc` file to make sure this is run by default
whenever `bat` is executed.

```sh
alias bat="bat --theme='base16-256'"
```

## Qutebrowser

To add [base16-qutebrowser] support, add the following to Tinty
config.toml:

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-qutebrowser"
name = "base16-qutebrowser"
themes-dir = "themes/default" # Or "themes/minimal"
hook = "cp -f %f ~/.config/qutebrowser/config.d/colorscheme.py"
theme-file-extension = ".config.py"
```

## Rofi

Add the following to `~/.config/tinted-theming/tinty/config.toml`:

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-rofi"
name = "tinted-rofi"
themes-dir = "colors"
hook = "cp -f %f ~/.config/rofi/base16-theme.rasi"
```

`~/.config/rofi/config.rasi` should contain this line:

```
@theme "~/.config/rofi/base16-theme.rasi"
```

## Dunst
Add the following to `~/.config/tinted-theming/tinty/config.toml`:

```toml
[[items]]
path = "https://github.com/tinted-theming/base16-dunst"
name = "base16-dunst"
themes-dir = "themes"
hook = "cp -f %f ~/.config/dunst/dunstrc && systemctl --user restart dunst"
```

The above `hook` assumes `dunst` is being managed as a service. If that is not the case, you will need to handle the restart for your system accordingly.

The above workflow is an all or nothing ordeal as the `dunstrc` configuration file does not appear to support importing or including additional files.

However, limited testing has shown `dunst` will not complain if its configuration file contains multiple `[global]` sections. This means we can persist our tinty-agnostic settings (fonts, etc) in a separate file and then use our `hook` to concatenate them like so:

```toml
hook = "cat ~/.config/dunst/dunstrc.local %f > ~/.config/dunst/dunstrc && systemctl --user restart dunst"
```

## Delta

Add the following to `~/.config/tinted-theming/tinty/config.toml`:

```toml
[[items]]
path = "https://github.com/tinted-theming/tinted-delta"
name = "tinted-delta"
themes-dir = "configs"
supported-systems = ["base16", "base24"]
```

Configure [delta] as your Git pager and/or difftool under the name `delta`, like this:

```gitconfig
[core]
	pager = delta

[interactive]
	diffFilter = delta --color-only

# Delta configuration not related to theming:
[delta]
	navigate = true
	line-numbers = true
	hyperlinks = true
	tabs = 4

# Include the config file generated by the tinty hook, which contains theme-related configuration:
[include]
	path = ~/.local/share/tinted-theming/tinty/tinted-delta-configs-file.gitconfig
```
[Usage section]: https://github.com/tinted-theming/tinty?tab=readme-ov-file#usage
[README.md]: https://github.com/tinted-theming/tinty/blob/main/README.md
[bat]: https://github.com/sharkdp/bat
[bat has an integration]: https://github.com/sharkdp/bat?tab=readme-ov-file#highlighting-theme
[tinted-fzf]: https://github.com/tinted-theming/tinted-fzf
[tinted-shell]: https://github.com/tinted-theming/tinted-shell
[tinted-tmux]: https://github.com/tinted-theming/tinted-tmux
[tmux tpm]: https://github.com/tmux-plugins/tpm
[XDG Base Directory specification]: https://wiki.archlinux.org/title/XDG_Base_Directory
[Sourcing scripts that set environment variables]: #sourcing-scripts-that-set-environment-variables
[README CLI section]: README.md#cli
[contrib/completion]: contrib/completion
[base16-qutebrowser]: https://github.com/tinted-theming/base16-qutebrowser
[delta]: https://github.com/dandavison/delta
[jq]: https://jqlang.github.io/jq/
