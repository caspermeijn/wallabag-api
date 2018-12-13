
# Wallabag Client Backend

[![wallabag-backend crates.io][cratesio-image]][cratesio]
[![wallabag-backend docs.rs][docsrs-image]][docsrs]

Shared backend client library for [Wallabag][wallabag] client apps.

## About

Manages a local sqlite database, full bidirectional syncing, and abstractions
over the whole thing for clients to easily work with.

This is currently under heavy development.

TODO: should this return failure::Error on error, or implement a custom error
type? I guess it depends on how it will be used?

Goals:

- [X] offline first
- [X] full two way syncing
- [ ] provide a simple, yet extensible api for all possible actions
- [ ] provide many convenience methods for common tasks (starring entries,
  adding urls, etc.)
- [ ] others?


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
[docsrs-image]: https://docs.rs/wallabag-backend/badge.svg
[docsrs]: https://docs.rs/wallabag-backend
[cratesio-image]: https://img.shields.io/crates/v/wallabag-backend.svg
[cratesio]: https://crates.io/crates/wallabag-backend
