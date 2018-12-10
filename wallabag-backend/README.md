
# Wallabag Client Backend

Shared backend client library that clients can use. Manages a local sqlite
database, offline syncing, and abstractions over the api library for common
tasks.

TODO: should this return failure::Error on error, or implement a custom error
type? I guess it depends on how it will be used?
