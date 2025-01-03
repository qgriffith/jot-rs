# Jot
[<img alt="crates.io" src="https://img.shields.io/crates/v/strava-client-rs.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/jot-rs)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-strava-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/jot-rs)
## About
A CLI to quickly create notes in the **markdown** format. It can create new notes,
or open exiting ones to edit. It can also search for a keyword and display all 
your notes. It has a *scratch* pad feature which allows you to quickly add something to a
long lived file, to later organize into a new note or existing note. 

### Example Use
```shell
jot -h
A ClI for jotting down notes

Usage: jot [OPTIONS] <COMMAND>

Commands:
  new      jot something new down
  open     Running jot doc that appends text to the same file
  scratch  Used as a scratch pad
  search   Search documents for a string of text
  list     List all files in your jot dir
  help     Print this message or the help of the given subcommand(s)

Options:
  -p, --jot-path <JOT_PATH>  [env: JOT_PATH=]
  -h, --help                 Print help
  -V, --version              Print version
```
> Jot dir defaults to ***/userhome/jot*** Can be set by passing -p or setting an env 
> var of JOT_PATH

## Versions
* [Release Notes](https://github.com/qgriffith/jot-rs/releases)
