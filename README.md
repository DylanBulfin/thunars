# Thuna.rs - Simple TUI File Browser in Rust
- Name shamelessly stolen from the GUI file browser [Thunar](https://wiki.archlinux.org/title/Thunar)
- Spiritual successor to a [previous project of mine](https://github.com/DylanBulfin/rust_practice/tree/main/filebrowser), a command line file browser

## Features
- Basic folder navigation (`n e` for down and up, `Return` to select)
- Also supports selecting files, for now always opens them via `code $file`
- Search mode: `/` opens a recursive directory search via `ignore::Walk`
    - Zoxide mode: `z` opens interactive `zoxide` search
    - For either mode exit via `Esc` or select an entry via `Return`
- Hint mode: `f` opens hint mode. A 1-2 character string will be assigned to each entry on screen, enter the string to jump your selection to this file
    - Similar to `hop.nvim`/`leap.nvim`, or the browser extension `Vimium (C)`

## Goals
- Fully configurable keybindings (in all modes)
- Configurable file associations (e.g. open some file types with specific programs)
- Basic file manipulation support
    - ~~Cut/Copy files one by one~~
    - Rename files
    - Create directory/empty file
    - Delete file/directory
- File details panel (On hold till I figure out a good layout)
    - Maybe just a 5 line or something thing on the side with the preview panel
- ~~File previews~~ Basic done

### Stretch Goals
- Some way to customize layout
- Support for populating finder list with arbitrary command instead of just `fzf`/`zoxide`
- Hints reorder based on selection's proximity to line in question (e.g. shorter hints assigned to closer entries)