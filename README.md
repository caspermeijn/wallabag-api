# wallabag-rs

Client tools for [Wallabag][wallabag] in Rust.


## About

This repository is a cargo workspace. See READMEs in subdirectories for
information about each package. The tools included are:

### [wallabag-api](wallabag-api/)

This is a client library to work directly with the Wallabag server
[API][api-docs] from Rust in a type-safe manner.

It is currently fully operational and manually tested with the Wallabag
instance running at [Framabag][framabag]. The API might change at any time.


### [wallabag-backend](wallabag-backend/)

A backend library designed to be shared between client software (also to be
developed here). It handles storing entries in a sqlite database, syncing with
a Wallabag server, and aims to provide helpful abstractions over common
actions so client software never has to directly touch the client API or
database.

### [wallabag-cli](wallabag-cli/)

(WIP)

A command line client for Wallabag. Currently a work in progress, but already
has some proof of concept actions developed (including saving a URL to
Wallabag).


### [wallabag-tui](wallabag-tui/)

(Unimplemented)

It is planned to develop a command line interactive client with a TUI. Work
will happen on this once the backend is more stable.


### [wallabag-gtk](wallabag-gtk/)

(Unimplemented)

Ultimately I would like to develop a full suite of client software, including a
GUI client, probably with GTK...


## Documentation

Everything should have extensive documentation, making the most of Rust's
excellent inline docs support. Run `cargo doc` to generate them. (TODO: link to
online docs once uploaded to crates.io)

## Developing

At the moment everything is managed by standard cargo commands - `build`,
`test`, `run`, etc. It is in a workspace, so the binary/crate to build/run
needs to be specified. For example:

```
cargo run --bin wallabag-cli -- sync
```

### Examples

There are some examples (in a crate's `examples/` directory) that can be run
like so:

```
cargo run --example save_url -- [args]
```


### Tests

Currently only a few unit tests have been developed... I'm really not sure how
to automate testing the backend and api. If you know how and willing to
contribute or teach me how, or if you know of resources for integration testing
in Rust, please contact me!! :)



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
[api-docs]: https://doc.wallabag.org/en/developer/api/readme.html
[framabag]: https://framabag.org/
