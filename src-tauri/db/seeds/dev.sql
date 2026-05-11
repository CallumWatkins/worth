INSERT INTO
  institutions (id, name)
VALUES
  (1, 'Nationwide'),
  (2, 'Monzo'),
  (3, 'Trading 212'),
  (4, 'Aviva'),
  (5, 'Starling'),
  (6, 'HSBC'),
  (7, 'American Express');

INSERT INTO
  accounts (
    id,
    name,
    institution_id,
    type_id,
    currency_code,
    account_classification,
    opened_date,
    closed_date
  )
VALUES
  (
    1,
    'Everyday Current',
    1,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'current'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    2,
    'Bills Pot',
    2,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'current'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    3,
    'Rainy Day Savings',
    1,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'savings'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    4,
    'Emergency Fund',
    1,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'savings'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    5,
    'Stocks & Shares ISA',
    3,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'isa'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    6,
    'General Investment Account',
    3,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'investment'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    7,
    'Workplace Pension',
    4,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'pension'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    8,
    'Holiday Savings',
    5,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'savings'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    9,
    'Cash ISA (Legacy)',
    6,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'isa'
    ),
    'GBP',
    'asset',
    NULL,
    NULL
  ),
  (
    10,
    'Everyday Credit Card',
    7,
    (
      SELECT
        id
      FROM
        account_types
      WHERE
        name = 'credit_card'
    ),
    'GBP',
    'liability',
    NULL,
    NULL
  );

-- Balance snapshots (irregular) for ~5 years.
-- Notes:
-- - `balance_date` is stored as ISO `YYYY-MM-DD`.
-- - Missing days mean balance unchanged since last stored value.
-- - Each account's latest snapshot is pinned exactly to its desired latest balance.
--   Not all accounts necessarily end on today's date.
WITH
  seed_params (
    account_id,
    latest_balance_minor,
    account_classification
  ) AS (
    VALUES
      (1, 243512, 'asset'),
      (2, 0, 'asset'),
      (3, 1325000, 'asset'),
      (4, 800000, 'asset'),
      (5, 4589042, 'asset'),
      (6, 1211070, 'asset'),
      (7, 9802533, 'asset'),
      (8, 142000, 'asset'),
      (9, 0, 'asset'),
      (10, -54321, 'liability')
  ),
  days (n) AS (
    SELECT
      0
    UNION ALL
    SELECT
      n + 1
    FROM
      days
    WHERE
      n < 1825
  ),
  raw AS (
    SELECT
      p.account_id,
      p.latest_balance_minor,
      p.account_classification,
      d.n,
      DATE('now', PRINTF('-%d days', 1825 - d.n)) AS balance_date,
      -- Per-account span so not all series start/end on the same dates.
      (1825 - (ABS(p.account_id * 97 + 4242) % 31)) AS end_n,
      CASE
        WHEN (1825 - (ABS(p.account_id * 97 + 4242) % 31)) - (900 + (ABS(p.account_id * 193 + 777) % 900)) < 0 THEN 0
        ELSE (1825 - (ABS(p.account_id * 97 + 4242) % 31)) - (900 + (ABS(p.account_id * 193 + 777) % 900))
      END AS start_n,
      CASE
        WHEN p.latest_balance_minor = 0 THEN 0
        ELSE (
          CASE
            WHEN p.account_classification = 'asset' THEN p.latest_balance_minor * (0.85 + 0.15 * (d.n / 1825.0))
            ELSE p.latest_balance_minor * (1.2 - 0.2 * (d.n / 1825.0))
          END
        )
      END AS base_minor,
      -- Deterministic random-walk step (correlated noise after cum-summing).
      CASE
        WHEN p.latest_balance_minor = 0 THEN 0
        ELSE (
          (
            (
              (
                CAST(
                  ABS(
                    (p.account_id * 1664525 + d.n * 1013904223 + 17) * (d.n + 3) + p.account_id * 97
                  ) % 1000003 AS REAL
                ) / 1000003.0
              ) * 2.0
            ) - 1.0
          ) * (ABS(p.latest_balance_minor) / 800.0 + 500.0)
        )
      END AS step_minor,
      -- Day-pick score used to create gaps (local maxima in a moving window).
      (
        ABS(
          (p.account_id * 1103515245 + d.n * 12345 + 4242) * (d.n + 1) + p.account_id * 77
        ) % 1000003
      ) AS pick_score
    FROM
      seed_params p
      CROSS JOIN days d
  ),
  walk1 AS (
    SELECT
      raw.*,
      SUM(step_minor) OVER (
        PARTITION BY
          account_id
        ORDER BY
          n ROWS BETWEEN UNBOUNDED PRECEDING
          AND CURRENT ROW
      ) AS cum_step
    FROM
      raw
  ),
  walk2 AS (
    SELECT
      walk1.*,
      MAX(
        CASE
          WHEN n = end_n THEN cum_step
          ELSE NULL
        END
      ) OVER (
        PARTITION BY
          account_id
      ) AS cum_end
    FROM
      walk1
  ),
  marked AS (
    SELECT
      walk2.*,
      MAX(pick_score) OVER (
        PARTITION BY
          account_id
        ORDER BY
          n ROWS BETWEEN CURRENT ROW
          AND 5 FOLLOWING
      ) AS pick_win_max
    FROM
      walk2
  )
INSERT INTO
  account_balance_snapshots (account_id, balance_date, balance_minor)
SELECT
  m.account_id,
  m.balance_date,
  CAST(
    ROUND(
      CASE
        WHEN m.latest_balance_minor = 0 THEN 0
        WHEN m.n = m.end_n THEN m.latest_balance_minor
        ELSE (
          m.base_minor + (
            m.cum_step - (m.n / CAST(m.end_n AS REAL)) * m.cum_end
          )
        )
      END
    ) AS INTEGER
  ) AS balance_minor
FROM
  marked m
WHERE
  -- Only include dates within each account's span.
  m.n >= m.start_n
  AND m.n <= m.end_n
  AND (
    -- Always include the first and last day in-span.
    m.n = m.start_n
    OR m.n = m.end_n
    -- Plus a sparse set of other days to create gaps.
    OR m.pick_score = m.pick_win_max
  );
