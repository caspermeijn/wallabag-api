
# Wallabag client API

[![wallabag-api crates.io][cratesio-image]][cratesio]
[![wallabag-api docs.rs][docsrs-image]][docsrs]

Provides types and functions for interacting with a [Wallabag][wallabag] server API.


## About

See the [documentation][docsrs] for usage information.

## supported API endpoints:

All API endpoints are implemented except for the `/api/*/list{,s}`. I don't
plan on implementing support for those unless there is a good reason to.

- [X] DELETE `/api/annotations/{annotation}.{_format}`
      Removes an annotation.
- [X] PUT `/api/annotations/{annotation}.{_format}`
      Updates an annotation.
- [X] GET `/api/annotations/{entry}.{_format}`
      Retrieve annotations for an entry.
- [X] POST `/api/annotations/{entry}.{_format}`
      Creates a new annotation.
- [X] GET `/api/entries.{_format}`
      Retrieve all entries. It could be filtered by many options.
- [X] POST `/api/entries.{_format}`
      Create an entry.
- [X] GET `/api/entries/exists.{_format}`
      Check if an entry exist by url.
- [ ] DELETE `/api/entries/list.{_format}`
      Handles an entries list and delete URL.
- [ ] POST `/api/entries/lists.{_format}`
      Handles an entries list and create URL.
- [ ] DELETE `/api/entries/tags/list.{_format}`
      Handles an entries list delete tags from them.
- [ ] POST `/api/entries/tags/lists.{_format}`
      Handles an entries list and add tags to them.
- [X] DELETE `/api/entries/{entry}.{_format}`
      Delete permanently an entry.
- [X] GET `/api/entries/{entry}.{_format}`
      Retrieve a single entry.
- [X] PATCH `/api/entries/{entry}.{_format}`
      Change several properties of an entry.
- [X] GET `/api/entries/{entry}/export.{_format}`
      Retrieve a single entry as a predefined format.
- [X] PATCH `/api/entries/{entry}/reload.{_format}`
      Reload an entry.
- [X] GET `/api/entries/{entry}/tags.{_format}`
      Retrieve all tags for an entry.
- [X] POST `/api/entries/{entry}/tags.{_format}`
      Add one or more tags to an entry.
- [X] DELETE `/api/entries/{entry}/tags/{tag}.{_format}`
      Permanently remove one tag for an entry.
- [X] DELETE `/api/tag/label.{_format}`
      Permanently remove one tag from every entry by passing the Tag label.
- [X] GET `/api/tags.{_format}`
      Retrieve all tags.
- [X] DELETE `/api/tags/label.{_format}`
      Permanently remove some tags from every entry.
- [X] DELETE `/api/tags/{tag}.{_format}`
      Permanently remove one tag from every entry by passing the Tag ID.
- [X] GET `/api/user.{_format}`
      Retrieve current logged in user informations.
- [X] PUT `/api/user.{_format}`
      Register an user and create a client.
- [X] GET `/api/version.{_format}`
      Retrieve version number.


## Examples

A few small examples are provided. To use these, the following environment
variables must be set (for authentication). For example:

```sh
export WALLABAG_CLIENT_ID="client_id"
export WALLABAG_CLIENT_SECRET="client_secret"
export WALLABAG_USERNAME="username"
export WALLABAG_PASSWORD="password"
export WALLABAG_URL="https://framabag.org" # must not end with trailing slash
```

The examples include:

- [check_exists](examples/check_exists.rs): check if there is an entry
  corresponding to the url provided.
- [example_sandbox](examples/example_sandbox.rs): a bunch of (mostly) commented
  out small examples, used for manual testing... have fun experimenting!
- [get_entries](examples/get_entries.rs): simply retrieve and debug print all
  entries. See the source code for filtering options.
- [save_url](examples/save_url.rs): save a url to the server, printing the
  created entry on success.

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
[docsrs-image]: https://docs.rs/wallabag-api/badge.svg
[docsrs]: https://docs.rs/wallabag-api
[cratesio-image]: https://img.shields.io/crates/v/wallabag-api.svg
[cratesio]: https://crates.io/crates/wallabag-api
