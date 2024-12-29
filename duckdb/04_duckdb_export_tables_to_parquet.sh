#!/bin/sh

mkdir -p parquet_exports
duckdb data.db -c "EXPORT DATABASE 'parquet_exports' (FORMAT PARQUET);"

# Export table names containing numbers to a text file, without formatting
duckdb data.db -c "
COPY (
    SELECT name
    FROM (SHOW ALL TABLES)
    WHERE regexp_matches(name, '.*[0-9].*')
) TO 'tables_with_numbers.txt' (HEADER FALSE);
"

# First clean up double underscores and trailing underscores
for f in parquet_exports/*.parquet; do
    new_name=$(echo "$f" | sed -e 's/_\+/_/g' -e 's/_\.parquet/.parquet/g')
    [ "$f" != "$new_name" ] && mv "$f" "$new_name"
done

# Then rename files based on tables_with_numbers.txt
while IFS= read -r table_name; do
    # Remove any trailing whitespace
    table_name=$(echo "$table_name" | tr -d ' ')
    
    # Find the corresponding parquet file by trying different base name patterns
    # Try removing all numbers and underscores after numbers
    base_name1=$(echo "$table_name" | sed 's/[0-9][0-9_]*//g')
    # Try removing just the numbers
    base_name2=$(echo "$table_name" | sed 's/[0-9]//g' | sed 's/_\+/_/g' | sed 's/_$//')
    # Try splitting at first number
    base_name3=$(echo "$table_name" | sed 's/[0-9].*$//')
    
    # Check all possible original files
    if [ -f "parquet_exports/${base_name1}.parquet" ]; then
        mv "parquet_exports/${base_name1}.parquet" "parquet_exports/${table_name}.parquet"
        echo "Renamed parquet_exports/${base_name1}.parquet to parquet_exports/${table_name}.parquet"
    elif [ -f "parquet_exports/${base_name2}.parquet" ]; then
        mv "parquet_exports/${base_name2}.parquet" "parquet_exports/${table_name}.parquet"
        echo "Renamed parquet_exports/${base_name2}.parquet to parquet_exports/${table_name}.parquet"
    elif [ -f "parquet_exports/${base_name3}.parquet" ]; then
        mv "parquet_exports/${base_name3}.parquet" "parquet_exports/${table_name}.parquet"
        echo "Renamed parquet_exports/${base_name3}.parquet to parquet_exports/${table_name}.parquet"
    else
        echo "Could not find matching file for ${table_name}"
    fi
done < tables_with_numbers.txt

echo "Export complete. Files saved in parquet_exports/"