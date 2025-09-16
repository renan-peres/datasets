#!/bin/sh

# Create directory for CSV exports
mkdir -p csv_exports

# Export to CSV format
duckdb data.db -c "EXPORT DATABASE 'csv_exports' (FORMAT CSV, DELIMITER ',');"

# Export table names containing numbers to a text file, without formatting
duckdb data.db -c "
COPY (
    SELECT name
    FROM (SHOW ALL TABLES)
    WHERE regexp_matches(name, '.*[0-9].*')
) TO 'tables_with_numbers.txt' (HEADER FALSE);
"

# First clean up double underscores and trailing underscores
for f in csv_exports/*.csv; do
    new_name=$(echo "$f" | sed -e 's/_\+/_/g' -e 's/_\.csv/.csv/g')
    [ "$f" != "$new_name" ] && mv "$f" "$new_name"
done

# Then rename files based on tables_with_numbers.txt
while IFS= read -r table_name; do
    # Remove any trailing whitespace
    table_name=$(echo "$table_name" | tr -d ' ')
    
    # Find the corresponding CSV file by trying different base name patterns
    # Try removing all numbers and underscores after numbers
    base_name1=$(echo "$table_name" | sed 's/[0-9][0-9_]*//g')
    # Try removing just the numbers
    base_name2=$(echo "$table_name" | sed 's/[0-9]//g' | sed 's/_\+/_/g' | sed 's/_$//')
    # Try splitting at first number
    base_name3=$(echo "$table_name" | sed 's/[0-9].*$//')
    
    # Check all possible original files
    if [ -f "csv_exports/${base_name1}.csv" ]; then
        mv "csv_exports/${base_name1}.csv" "csv_exports/${table_name}.csv"
        echo "Renamed csv_exports/${base_name1}.csv to csv_exports/${table_name}.csv"
    elif [ -f "csv_exports/${base_name2}.csv" ]; then
        mv "csv_exports/${base_name2}.csv" "csv_exports/${table_name}.csv"
        echo "Renamed csv_exports/${base_name2}.csv to csv_exports/${table_name}.csv"
    elif [ -f "csv_exports/${base_name3}.csv" ]; then
        mv "csv_exports/${base_name3}.csv" "csv_exports/${table_name}.csv"
        echo "Renamed csv_exports/${base_name3}.csv to csv_exports/${table_name}.csv"
    else
        echo "Could not find matching file for ${table_name}"
    fi
done < tables_with_numbers.txt

echo "Export complete. Files saved in csv_exports/"