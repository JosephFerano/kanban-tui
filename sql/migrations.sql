PRAGMA foreign_keys = ON;

create table if not exists task
(
    id integer primary key autoincrement,
    title text not null,
    description text not null,
    sort_order integer not null,
    column_id integer,
    foreign key (column_id) references kb_column(id)
);

create table if not exists kb_column
(
    id integer primary key autoincrement,
    name text not null,
    selected_task integer not null default 0
);


create table if not exists app_state
(
    key text not null primary key,
    value text not null
);

insert into kb_column(name) values ("Todo"),("InProgress"),("Done"),("Ideas");
insert into app_state(key, value) values ("selected_column", "0");
