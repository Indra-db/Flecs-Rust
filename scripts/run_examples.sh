# Find the project root directory using git
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

cd "$PROJECT_ROOT/flecs_ecs/examples" || {
  echo "Could not find the examples directory"
  exit 1
}

for example in *.rs; do
  name=$(basename "$example" .rs)
  echo "Running example: $name"
  cargo run --example "$name" || exit 1
done
