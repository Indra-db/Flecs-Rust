#!/bin/bash

# Check if the --allow-panic argument is provided
allow_panic=false
if [[ "$1" == "--allow-panic" ]]; then
  allow_panic=true
fi

# Navigate to the project root directory
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
cd "$PROJECT_ROOT/flecs_ecs/examples" || {
  echo "Could not find the examples directory"
  exit 1
}

for example in *.rs; do
  # Extract the base name of the example
  name=$(basename "$example" .rs)
  echo "Running example: $name"

  # Run the example using Cargo
  if ! cargo run --example "$name"; then
    if [ "$allow_panic" = true ]; then
      echo "Example $name failed to run successfully, continuing..."
    else
      echo "Example $name failed to run, exiting..."
      exit 1
    fi
  fi
done

echo "All examples have been processed."
