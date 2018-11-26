
# Wallabag client API

Provides types and functions for interacting with a Wallabag server.


## supported endpoints:


- [X] DELETE `/api/annotations/{annotation}.{_format}`
      Removes an annotation.
- [X] PUT `/api/annotations/{annotation}.{_format}`
      Updates an annotation.
- [X] GET `/api/annotations/{entry}.{_format}`
      Retrieve annotations for an entry.
- [X] POST `/api/annotations/{entry}.{_format}`
      Creates a new annotation.
- [-] GET `/api/entries.{_format}`
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
- [ ] DELETE `/api/entries/{entry}.{_format}`
      Delete permanently an entry.
- [X] GET `/api/entries/{entry}.{_format}`
      Retrieve a single entry.
- [ ] PATCH `/api/entries/{entry}.{_format}`
      Change several properties of an entry.
- [ ] GET `/api/entries/{entry}/export.{_format}`
      Retrieve a single entry as a predefined format.
- [ ] PATCH `/api/entries/{entry}/reload.{_format}`
      Reload an entry.
- [ ] GET `/api/entries/{entry}/tags.{_format}`
      Retrieve all tags for an entry.
- [ ] POST `/api/entries/{entry}/tags.{_format}`
      Add one or more tags to an entry.
- [ ] DELETE `/api/entries/{entry}/tags/{tag}.{_format}`
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
