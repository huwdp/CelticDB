#!/bin/bash

# Array of files
files=(
    "create_statement/create1.sql"
    "alter_statement/alter1.sql"
    "insert_statement/insert1.sql"
    "drop_statement/drop1.sql"
    "select_statement/select1.sql"
    "select_statement/select_distinct.sql"
    "truncate_statement/truncate1.sql"
    "create_statement//create_with_columns.sql"
    "alter_statement/alter2.sql"
)

# Loop through each file in the array
for file in "${files[@]}"
do
    echo "Running test $file"
    # Run the file through the ./main executable and pipe the output to the output file
    ./main "tests/$file" > "tests/$file.out"
    diff "tests/$file.out" "tests/$file.exp"
done