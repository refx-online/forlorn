create table client_errors 
(
    id bigint unsigned not null auto_increment primary key,

    user_id int not null default 0,
    username varchar(64) not null default 'Offline user',

    feedback text null,
    exception varchar(512) null,
    stacktrace text null,

    created_at timestamp not null default current_timestamp,

    index idx_user_id (user_id),
    index idx_created_at (created_at)
);
