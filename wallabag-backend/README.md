
# Wallabag Client Backend

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
