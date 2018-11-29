create table thing (
  id integer primary key,
  thing text
);

-- annotations (one-to-many) and tags (many-to-many) associated
create table entry (
  id integer primary key,
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
  reading_time integer,
  starred_at text,  -- datetime
  title text,
  uid text,
  updated_at text not null, -- datetime
  url text
);


create table taglink (
  tag_id integer not null,
  entry_id integer not null,
  foreign key (tag_id) references tag (id),
  foreign key (entry_id) references entry (id)
);

create table tag (
  id integer primary key,
  label text not null,
  slug text not null
);

-- has ranges associated (one-to-many)
create table annotation (
  id integer primary key,
  annotator_schema_version text not null,
  created_at text not null, -- datetime
  quote text,
  text not null, -- empty text represented with empty string
  updated_at text not null, -- datetime
  entry_id integer not null,
  foreign key (entry_id) references entry (id)
);


create table range (
  id integer primary key,
  start text,
  end text,
  start_offset integer not null,
  end_offset integer not null
);
