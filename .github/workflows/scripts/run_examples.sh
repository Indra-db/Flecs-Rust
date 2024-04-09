cd ../../../flecs_ecs/examples || exit

for example in *.rs; do
    name=$(basename "$example" .rs)
    echo "Running example: $name"
    cargo run --example "$name" || exit 1
done