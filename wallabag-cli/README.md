
# Wallabag Cli

Command line client for wallabag.


(WIP)

- offline first
- syncing
- save urls direct from command line
- interact and non-interactive flows
- etc.


## Developing

To update the db schema:

1. edit `./migrations/create_tables/up.sql` with the new schema
2. delete the old database
3. run `diesel migrations run --database-url db.sqlite3` (this will update the
   schema.rs file automatically.
