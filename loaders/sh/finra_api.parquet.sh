#!/bin/bash

# Credentials & paths
CLIENT_ID="445db239b8054bbb9542"
CLIENT_SECRET="kbj.uxc5TCQ-vah9cqr"
CODE="finra"

# Get token
AUTH_STRING=$(echo -n "$CLIENT_ID:$CLIENT_SECRET" | base64)
TOKEN_RESPONSE=$(curl -s -X POST \
  "https://ews.fip.finra.org/fip/rest/ews/oauth2/access_token?grant_type=client_credentials" \
  -H "Authorization: Basic $AUTH_STRING")
TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

# Get data and convert to CSV
curl -s \
  "https://api.finra.org/data/group/fixedIncomeMarket/name/treasuryMonthlyAggregates?limit=1000" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" | \
  jq -r '(.[0] | keys_unsorted) as $keys | $keys, map([.[ $keys[] ]])[] | @csv' > "$CODE.csv"

# Convert CSV to Parquet
duckdb << EOF
COPY (
    SELECT *
    FROM read_csv('$CODE.csv')
) TO STDOUT (FORMAT 'parquet', COMPRESSION 'gzip');
  -- TO '$CODE.parquet' (FORMAT 'parquet', COMPRESSION 'gzip'); -- Write the File to Directory
EOF

