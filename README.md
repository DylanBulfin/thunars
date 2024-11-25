# Thuna.rs - Simple TUI File Browser in Rust
- Name shamelessly stolen from the GUI file browser [Thunar](https://wiki.archlinux.org/title/Thunar)
- Spiritual successor to a [previous project of mine](https://github.com/DylanBulfin/rust_practice/tree/main/filebrowser), a command line file browser

## Video Demo
[A video demo showing off some of the major features](https://youtu.be/0Z4GG511xQg)

## Status
I will not be making further updates to this project except to fix any issues I notice.

## Features
- Basic folder navigation (`n e` for down and up, `Return` to select)
- Also supports selecting files, for now always opens them via `code $file`
- Search mode: `/` opens a recursive directory search via `ignore::Walk`
    - Zoxide mode: `z` opens interactive `zoxide` search
    - For either mode exit via `Esc` or select an entry via `Return`
- Hint mode: `f` opens hint mode. A 1-2 character string will be assigned to each entry on screen, enter the string to jump your selection to this file
    - Similar to `hop.nvim`/`leap.nvim`, or the browser extension `Vimium (C)`
