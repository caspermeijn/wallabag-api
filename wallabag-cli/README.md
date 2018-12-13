
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

```
MIT License

Copyright (c) 2018 Samuel Walladge

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```


[wallabag]: https://wallabag.org/
