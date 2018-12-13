
# Wallabag CLI

Command line client for [Wallabag][wallabag].

## About

This aims to be an extensive non-interactive command line application to
interact with a Wallabag server.

Goals:


- [X] offline first
- [X] full two way syncing
- [X] save urls direct from command line
- [-] list and view entries
- [ ] export and save entries in a supported format
- [ ] create/edit/delete entries
- [ ] create/edit/delete annotations
- [ ] create/edit/delete tags
- [ ] search data


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




## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


[wallabag]: https://wallabag.org/
