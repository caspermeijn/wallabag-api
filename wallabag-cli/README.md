
# Wallabag Cli

Command line client for Wallabag.

- offline first
- syncing
- save urls direct from command line
- list and view entries
- etc. [WIP]


## Usage

Note: to run without installing, you can use `cargo run --bin <bin name>`. It
will need `--` between this and any args meant for the target program. Eg.
`cargo run --bin wallabag-cli -- entry list`.

First, sync everything for use. All (well, most) commands that operate on the
data work solely on the local versions saved in the database.

```
wallabag-cli sync
```

For some things, a full sync is required (eg. remotely deleted entries):

```
wallabag-cli sync --full
```

List entries:

```
wallabag-cli entry list
```

Show an entry with ID (IDs are shown in `entry list`). This dumps the html
output - pipe through something that can display the html for easy reading:

```
cargo run --bin wallabag-cli -- entry show 1798248 | w3m -dump -T text/html
```



## Developing


