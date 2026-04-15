test: fmt
    cargo clippy
    cargo test

build:
    cargo build --release
    cp target/release/t-rex ./trex

# autoformat the code
fmt:
    cargo fmt

benchmark: build
    #!/bin/bash
    set -euo pipefail

    bench() {
        if ! diff -q <(eval "$2") <(eval "$3") ; then
            echo "outputs of $2 and $3 differ"
            exit 1
        fi

        hyperfine -r "$1" --shell=none -w 5 "$2" "$3"
    }

    bench 10000 \
        "./trex 'fn [a-z]+\(' src/main.rs" \
        "grep -E 'fn [a-z]+\(' src/main.rs"

stress-test: build
    #!/bin/bash
    set -euo pipefail

    reps=200
    input="$(for ((i=0; i<=$reps; i++)); do echo -n "a"; done)"
    regex="$(for ((i=0; i<=$reps; i++)); do echo -n "a?"; done)$input"

    echo "$input" | ./trex "$regex"

integration-test: build
    awk -f test.awk data/*.dat
