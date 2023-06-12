PRAGMA foreign_keys = ON;

create table if not exists task
(
    id integer primary key autoincrement,
    title text not null,
    description text not null,
    sort_order integer not null default 0,
    column_id integer,
    foreign key (column_id) references kb_column(id)
);

create table if not exists kb_column
(
    id integer primary key autoincrement,
    name text not null,
    selected_task integer not null default 0
);


create table if not exists setting
(
    id integer primary key autoincrement,
    name text not null,
    value text not null
);
