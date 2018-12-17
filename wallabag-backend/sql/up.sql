begin;

-- annotations (one-to-many) and tags (many-to-many) associated
create table entries (
  id integer primary key not null,
  content text,
  created_at text not null, -- datetime
  domain_name text,
  http_status text,
  is_archived boolean not null,
  is_public boolean not null,
  is_starred boolean not null,
  language text,
  mimetype text,
  origin_url text,
  preview_picture text,
  published_at text, -- datetime
  published_by text,  -- json array
  reading_time integer,
  starred_at text,  -- datetime
  title text,
  uid text,
  updated_at text not null, -- datetime
  url text,
  headers text,  -- json array
  tags text not null,  -- json array
  user_email text not null,
  user_id integer not null,
  user_name text not null
);

create unique index index_entries_id on entries (id);

create table taglinks (
  tag_id integer not null,
  entry_id integer not null,
  foreign key (tag_id) references tags (id) on delete cascade,
  foreign key (entry_id) references entries (id) on delete cascade,
  primary key (tag_id, entry_id)
);

create unique index index_taglinks_id on taglinks (tag_id, entry_id);

-- has entries associated (many-to-many)
create table tags (
  id integer primary key not null,
  label text not null,
  slug text not null
);

create unique index index_tags_id on tags (id);

create table annotations (
  id integer primary key not null,
  annotator_schema_version text not null,
  created_at text not null, -- datetime
  ranges text not null, -- json array
  text text not null, -- empty text represented with empty string
  updated_at text not null, -- datetime
  quote text,
  user text,
  entry_id integer not null,
  foreign key (entry_id) references entries (id) on delete cascade
);

create unique index index_annotations_entry_id on annotations (entry_id);

-- tables to hold temporary ids of locally deleted entries; should be deleted
-- after sync
create table deleted_entries (
  id integer primary key not null
);

create table deleted_annotations (
  id integer primary key not null
);

create table deleted_tags (
  id integer primary key not null
);

-- to manage adding new things offline; items in new_* should be deleted after synced
create table new_urls (
  id integer primary key not null,
  url text not null
);

create table new_annotations (
  id integer primary key not null,
  quote text not null, -- also should not be empty string or failure
  text text not null,
  ranges text not null, -- json array
  entry_id integer not null,
  foreign key (entry_id) references entries (id) on delete cascade
);


-- used to hold a single row with columns for each saved config value
-- (this is config that isn't set by the user)
create table config (
  id integer primary key not null,
  last_sync text not null -- datetime
);

-- initial config
insert into config values (
  1,
  "1970-01-01T00:00:00+00:00"
);

commit;
