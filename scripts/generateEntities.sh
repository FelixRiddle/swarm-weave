#!/bin/bash

# Run the command and store the output in a variable
OUTPUT=$(cargo run print --mysql-connection-string)

# Run the sea-orm-cli command using the stored output
sea-orm-cli generate entity -u "$OUTPUT" -o entity/src
