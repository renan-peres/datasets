#!/bin/sh

format_size() {
    bytes=$1
    if [ "$bytes" -lt 1024 ]; then
        echo "${bytes}B"
    elif [ "$bytes" -lt 1048576 ]; then
        echo "$(( (bytes + 512) / 1024 ))KB"
    else
        echo "$(( (bytes + 524288) / 1048576 ))MB"
    fi
}

calculate_percentage() {
    size=$1
    total=$2
    echo "$size $total" | awk '{printf "%.2f", ($1/$2)*100}'
}

# Show database size info first
echo "Database Information:"
duckdb data.db -csv -c "PRAGMA database_size;"
echo ""

echo "Table,Size,Percentage,Columns,Rows"

# Calculate total table sizes
total_size=0
tables=$(duckdb data.db -csv -header -c "SELECT name FROM sqlite_master WHERE type='table'" | tail -n +2)

# First pass to calculate total size
for table in $tables; do
    if [ "$table" != "name" ]; then
        size=$(duckdb data.db -csv -c "PRAGMA storage_info('$table');" 2>/dev/null | awk -F',' '{sum+=$3} END {print sum}')
        if [ -n "$size" ] && [ "$size" != "0" ]; then
            total_size=$((total_size + size))
        fi
    fi
done

# Display results with columns and rows
printf "%s\n" "$tables" | while IFS= read -r table; do
    if [ "$table" != "name" ]; then
        size=$(duckdb data.db -csv -c "PRAGMA storage_info('$table');" 2>/dev/null | awk -F',' '{sum+=$3} END {print sum}')
        if [ -n "$size" ] && [ "$size" != "0" ]; then
            percentage=$(calculate_percentage "$size" "$total_size")
            formatted_size=$(format_size "$size")
            
            # Get column count
            cols=$(duckdb data.db -csv -c "SELECT COUNT(*) FROM pragma_table_info('$table');" | tail -n 1)
            
            # Get row count
            rows=$(duckdb data.db -csv -c "SELECT COUNT(*) FROM $table;" 2>/dev/null | tail -n 1)
            
            printf "%s,%s,%.2f%%,%s,%s\n" "$table" "$formatted_size" "$percentage" "$cols" "$rows"
        fi
    fi
done | sort -t',' -k3 -n -r

echo "Total Tables,$(format_size "$total_size"),100%,-,-"