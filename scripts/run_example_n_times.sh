if [ "$#" -ne 2 ]; then
    echo "Usage: $0 example_name times"
    exit 1
fi

EXAMPLE_NAME="$1"

TIMES="$2"

if ! [[ "$TIMES" =~ ^[0-9]+$ ]]; then
    echo "Error: 'times' must be a positive integer."
    exit 1
fi

for ((i=1; i<=TIMES; i++))
do
    echo "Running $EXAMPLE_NAME for the $i time..."
    cargo run --example "$EXAMPLE_NAME"
    EXIT_STATUS=$?
    
    if [ $EXIT_STATUS -ne 0 ]; then
        echo "Error: $EXAMPLE_NAME encountered an assert or crashed on attempt $i."
        exit $EXIT_STATUS
    fi
done

echo "Completed running $EXAMPLE_NAME $TIMES times."
