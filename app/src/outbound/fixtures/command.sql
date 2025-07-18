INSERT INTO "command" (
    uuid,
    fermentation_step_id,
    status,
    status_date,
    value,
    value_reached_at,
    value_holding_duration,
    session_id,
    execution_order
)
VALUES (
    '23bc0b04-05a4-4d28-a82d-2cc640fb3042',
    1,
    'Planned',
    NOW(),
    20.4,
    null,
    1,
    1,
    0
);

INSERT INTO "command" (
    uuid,
    fermentation_step_id,
    status,
    status_date,
    value,
    value_reached_at,
    value_holding_duration,
    session_id,
    execution_order
)
VALUES (
    'b51a3a1b-9e4c-4e6d-ab96-3f0972afbd9c',
    1,
    'Running',
    NOW(),
    20.4,
    null,
    1,
    1,
    1
);
