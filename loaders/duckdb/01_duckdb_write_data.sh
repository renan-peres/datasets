#!/bin/sh
DB_FILE="data.db"
LANDING_ZONE="landing_zone"

# Create file to track imports
: > temp_imported_files.txt

# Process all files in landing_zone directory
for file in "$LANDING_ZONE"/[!._]*.csv "$LANDING_ZONE"/[!._]*.parquet "$LANDING_ZONE"/[!._]*.json "$LANDING_ZONE"/[!._]*.jsonl; do
    [ -f "$file" ] || continue
    
    # Get extension and table name (remove directory prefix)
    ext="${file##*.}"
    base_file=$(basename "$file")
    table_name=$(echo "$base_file" | sed "s/\.$ext$//" | tr '-' '_')
    
    # Set read command based on extension
    case "$ext" in
        csv)     
            cmd="read_csv_auto"
            echo "Created table $table_name from CSV file: $base_file"
            ;;
        parquet) 
            cmd="read_parquet"
            echo "Created table $table_name from Parquet file: $base_file"
            ;;
        json) 
            cmd="read_json_auto"
            echo "Created table $table_name from JSON file: $base_file"
            ;;
        jsonl) 
            cmd="read_json_auto"
            echo "Created table $table_name from JSONL file: $base_file"
            ;;
        *) continue ;;
    esac
    
    duckdb "$DB_FILE" "CREATE OR REPLACE TABLE $table_name AS (FROM $cmd('$file'));"
    echo "$file" >> temp_imported_files.txt
done

# Handle deletion if files were imported
if [ -s temp_imported_files.txt ]; then
    echo "\nImported files:"
    cat temp_imported_files.txt
    printf "Delete these files? (Y/N): "
    read response
    case $response in
        [Yy]*) xargs rm < temp_imported_files.txt && echo "Files deleted" ;;
        *) echo "Files preserved" ;;
    esac
fi

rm temp_imported_files.txt