#!/bin/bash
set -ex

for file in test_files/*
do
    cargo run "$file"
done

for file in expected/*
do
    cmp "$file" $(basename "$file")
done

echo "Success"
