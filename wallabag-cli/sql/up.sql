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
  headers text,
  user_email text not null,
  user_id integer not null,
  user_name text not null
);


create table taglinks (
  tag_id integer not null,
  entry_id integer not null,
  foreign key (tag_id) references tags (id),
  foreign key (entry_id) references entries (id),
  primary key (tag_id, entry_id)
);

-- has entries associated (many-to-many)
create table tags (
  id integer primary key not null,
  label text not null,
  slug text not null
);

create table annotations (
  id integer primary key not null,
  annotator_schema_version text not null,
  created_at text not null, -- datetime
  quote text,
  ranges text not null, -- json array
  text text not null, -- empty text represented with empty string
  updated_at text not null, -- datetime
  entry_id integer not null,
  user text,
  foreign key (entry_id) references entries (id)
);

commit;
