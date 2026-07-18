//! Query flags discovered & set during query creation.
use super::*;
create_pre_registered_component!(
    MatchPrefab,
    ECS_QUERY_MATCH_PREFAB,
    "Query must match prefabs."
);
create_pre_registered_component!(
    MatchDisabled,
    ECS_QUERY_MATCH_DISABLED,
    "Query must match disabled entities."
);
create_pre_registered_component!(
    MatchEmptyTables,
    ECS_QUERY_MATCH_EMPTY_TABLES,
    "Query must match empty tables."
);

create_pre_registered_component!(
    AllowUnresolvedByName,
    ECS_QUERY_ALLOW_UNRESOLVED_BY_NAME,
    "Query may have unresolved entity identifiers."
);
create_pre_registered_component!(
    DetectChanges,
    ECS_QUERY_DETECT_CHANGES,
    "Query detects changes to matched components."
);
create_pre_registered_component!(
    GroupByOrdered,
    ECS_QUERY_GROUP_BY_ORDERED,
    "Iterate query groups in ascending group id order."
);
create_pre_registered_component!(
    GroupByDesc,
    ECS_QUERY_GROUP_BY_DESC,
    "Iterate query groups in descending group id order."
);
create_pre_registered_component!(
    TableOnly,
    ECS_QUERY_TABLE_ONLY,
    "Query only returns whole tables (ignores toggle/member fields)."
);
