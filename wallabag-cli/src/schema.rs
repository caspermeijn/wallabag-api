table! {
    annotations (id) {
        id -> Integer,
        annotator_schema_version -> Text,
        created_at -> Text,
        quote -> Nullable<Text>,
        text -> Text,
        updated_at -> Text,
        entry_id -> Integer,
    }
}

table! {
    entries (id) {
        id -> Integer,
        content -> Nullable<Text>,
        created_at -> Text,
        domain_name -> Nullable<Text>,
        http_status -> Nullable<Text>,
        is_archived -> Bool,
        is_public -> Bool,
        is_starred -> Bool,
        language -> Nullable<Text>,
        mimetype -> Nullable<Text>,
        origin_url -> Nullable<Text>,
        preview_picture -> Nullable<Text>,
        published_at -> Nullable<Text>,
        reading_time -> Nullable<Integer>,
        starred_at -> Nullable<Text>,
        title -> Nullable<Text>,
        uid -> Nullable<Text>,
        updated_at -> Text,
        url -> Nullable<Text>,
    }
}

table! {
    ranges (id) {
        id -> Integer,
        start -> Nullable<Text>,
        end -> Nullable<Text>,
        start_offset -> Integer,
        end_offset -> Integer,
        annotation_id -> Integer,
    }
}

table! {
    taglinks (tag_id, entry_id) {
        tag_id -> Integer,
        entry_id -> Integer,
    }
}

table! {
    tags (id) {
        id -> Integer,
        label -> Text,
        slug -> Text,
    }
}

joinable!(annotations -> entries (entry_id));
joinable!(ranges -> annotations (annotation_id));
joinable!(taglinks -> entries (entry_id));
joinable!(taglinks -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    annotations,
    entries,
    ranges,
    taglinks,
    tags,
);
