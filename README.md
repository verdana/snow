# S(n)ow

S(n)ow is a clear and concise replacement for GNU stow.

S(n)ow is not intended to be a complete replacement for stow, but to provide a more concise alternative.

# INSTALL

**Install from script**

```bash
curl -fsSL https://example.net/snow/install.sh | sh -
```

**Install from source**

```bash
git clone https://github.com/verdana/snow.git

cd snow
cargo install --path .

# The binary executable will be installed in the $HOME/.cargo/bin directory.
```

# USAGE

Before we start, let’s explain the 3 terms we need to know to work with STOW, which are: **Package**, **Stow Directory** and **Target Directory**.

-   **Package**: It’s a collection of files that must be “installed” together in the same target directory
-   **Stow directory**: It’s the repository where all our packages will be
-   **Target directory**: It’s the directory where we want to install those configurations that are in our packages

This is the help information for snow:

```txt
Usage: snow [OPTIONS] [COMMAND]

Commands:
  link    Link packages to the target dir
  unlink  Unlink the specified packages
  list    List the linked packages
  prune   Delete all symlinks from target dir

Options:
  -t, --target <DIR>  The dir to link the packages to [default: ~]
  -h, --help          Print help
  -V, --version       Print version
```

Now let’s assume we have a dir called `dotfiles` that contains our configuration files, and we want to link these configuration files to our $HOME.

```txt
~/dotfiles
❯ tree .
.
├── git
│   └── .gitconfig
├── kitty
│   └── .config
|       └── kitty
|           ├── kitty.conf
|           └── theme-nord.conf
```

`dotfiles` directory contains two packages `git` and `kitty`.
When we use the `snow link` command, it will link the files in these two packages to our target directory.

If no target directory is specified, `snow` will link the files to the $HOME directory.

```bash
# Create a symlink ~/.gitconfig pointing to ~/dotfiles/git/.gitconfig
snow link git

# Create the following symlinks:
# ~/.gitconfig    -> ~/dotfiles/git/.gitconfig
# ~/.config/kitty -> ~/dotfiles/kitty/.config/kitty
snow link git kitty

# Equivalent to: snow link git kitty
snow link *

# Remove symblink ~/.gitconfig
snow unlink git

# Remove symblinks:
#   ~/.gitconfig
#   ~/.config/kitty
snow unlink git kitty

# Remove all symlinks created by snow
snow prune
```
