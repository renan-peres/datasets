#!/bin/sh

# Path to your DuckDB database
DB_PATH="data.db"

# Function to convert camelCase to snake_case
camel_to_snake() {
    echo "$1" | sed -E 's/([a-z0-9])([A-Z])/\1_\2/g' | tr '[:upper:]' '[:lower:]'
}

# Get all tables in the database
all_tables=$(duckdb "$DB_PATH" -csv -noheader "SELECT name FROM (SHOW ALL TABLES);")

echo "============================================"
echo "Starting column rename process..."
echo "============================================"

# Process each table
echo "$all_tables" | while IFS= read -r table; do
    [ -z "$table" ] && continue
    
    echo ""
    echo "Processing table: $table"
    echo "--------------------------------------------"
    
    # Get columns for this table
    columns=$(duckdb "$DB_PATH" -csv -noheader "
        SELECT UNNEST(column_names) 
        FROM (SHOW ALL TABLES) 
        WHERE name = '${table}';
    ")
    
    # Build ALTER TABLE statements
    alter_statements=""
    echo "$columns" | while IFS= read -r col; do
        [ -z "$col" ] && continue
        
        # Convert to snake_case and lowercase
        new_col=$(camel_to_snake "$col")
        
        # Only add ALTER statement if column name actually changes
        if [ "$col" != "$new_col" ]; then
            echo "  Renaming: $col -> $new_col"
            
            # Execute the rename immediately
            duckdb "$DB_PATH" "ALTER TABLE \"${table}\" RENAME COLUMN \"${col}\" TO \"${new_col}\";" 2>&1
            
            if [ $? -eq 0 ]; then
                echo "    ✓ Successfully renamed"
            else
                echo "    ✗ Failed to rename"
            fi
        fi
    done
    
    echo "  Completed processing table: $table"
done

echo ""
echo "============================================"
echo "Column rename process completed!"
echo "============================================"

echo ""
echo "============================================"
echo "Starting date column conversion process..."
echo "============================================"

# Process each table to convert date columns from TIMESTAMP to DATE
echo "$all_tables" | while IFS= read -r table; do
    [ -z "$table" ] && continue
    
    # Get columns with their types
    date_columns=$(duckdb "$DB_PATH" -csv -noheader "
        SELECT column_name 
        FROM information_schema.columns 
        WHERE table_name = '${table}' 
        AND column_name = 'date' 
        AND data_type LIKE '%TIMESTAMP%';
    ")
    
    # Check if there are any date columns to convert
    if [ -n "$date_columns" ]; then
        echo ""
        echo "Processing table: $table"
        echo "--------------------------------------------"
        
        echo "$date_columns" | while IFS= read -r col; do
            [ -z "$col" ] && continue
            
            echo "  Converting column '$col' from TIMESTAMP to DATE"
            
            # Use ALTER TABLE to change the column type
            duckdb "$DB_PATH" "ALTER TABLE \"${table}\" ALTER COLUMN \"${col}\" TYPE DATE;" 2>&1
            
            if [ $? -eq 0 ]; then
                echo "    ✓ Successfully converted to DATE"
            else
                echo "    ✗ Failed to convert"
            fi
        done
    fi
done

echo ""
echo "============================================"
echo "Date column conversion completed!"
echo "============================================"

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