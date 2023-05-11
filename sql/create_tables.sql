create table if not exists monitors (
    -- sqlite stores datetime as integers
    timestamp integer,
    domain text,
    power_watts integer not null,
    constraint pk primary key (timestamp, domain),
    check (power_watts > 0)
);

create table if not exists trials (
    id integer primary key autoincrement,
    start_time integer,
    end_time integer,
    cap_request_time integer,
    cap_did_complete boolean,
    cap_complete_time_millis integer,
    load_pct integer,
    load_period integer,
    n_threads integer,
    capping_order text,
    capping_operation text,
    check (load_pct between 1 and 100),
    check (load_period >= 100),
    check (n_threads > 0),
    check (capping_order in ('LevelBeforeActivate', 'LevelAfterActivate')),
    check (capping_operation in ('Activate', 'Deactivate'))
);


-- load the data:
--      sqlite> .mode csv <table>  --- I don't think this is needed.
--      sqlite> .import <filename> <table>
--

-- Create indexes after data load
-- create index mon_time_idx on monitors (timestamp);
-- create index mon_domain_idx on monitors (domain);
