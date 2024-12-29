#!/bin/sh

# Path to your DuckDB database
DB_PATH="data.db"

# Get the list of problematic tables
tables=$(duckdb "$DB_PATH" -csv -noheader "
WITH problematic_tables AS (
    SELECT DISTINCT name as table_name
    FROM (
        SELECT name, UNNEST(column_names) as col_name 
        FROM (SHOW ALL TABLES)
    ) t 
    WHERE col_name LIKE 'column%'
)
SELECT table_name FROM problematic_tables;
")

# Show problematic tables info
duckdb "$DB_PATH" << EOF
-- First create a view of problematic tables
CREATE OR REPLACE TEMP VIEW problematic_tables AS
SELECT DISTINCT name as table_name
FROM (
    SELECT name, UNNEST(column_names) as col_name 
    FROM (SHOW ALL TABLES)
) t 
WHERE col_name LIKE 'column%';

-- Show the problematic tables
SELECT 'Tables with "column" in their column names:' as message;
SELECT * FROM problematic_tables;

-- Execute sample queries for each table
WITH RECURSIVE sample_queries AS (
    SELECT ROW_NUMBER() OVER () as id,
           'SELECT * FROM ' || table_name || ' LIMIT 5' as query
    FROM problematic_tables
)
SELECT query || ';'
FROM sample_queries
ORDER BY id;

EOF

# For each table, show sample data
echo "$tables" | while IFS= read -r table; do
    [ -z "$table" ] && continue
    
    echo ""
    duckdb "$DB_PATH" << EOF
SELECT 'Sample data from ${table}:' as message;
SELECT * FROM "${table}" LIMIT 5;
EOF
done