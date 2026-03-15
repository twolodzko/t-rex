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
    #!/bin/bash
    set -euo pipefail

    awk '# remember previous line to replace SAME in the tests
        // {
            if ($2 == "SAME") {
                $2=prev
            } else {
                prev=$2
            }
        }

        # those lines failed to parse for some reason, so skipping them
        $3 == "" { next }

        # self-references are not supported
        $2 ~ /\\[1-9]/ { next }
        # lazy matching is not supported
        $2 ~ /[+*?]+/ { next }
        # character classes are not supported
        $2 ~ /\[[^]]*\[[:.=]/ { next }
        # special modifiers are not supported
        $2 ~ /\(\?[<>:=!i]/ { next }
        # dont care about empty branches in alternation
        $4 == "ENULL" { next }

        FILENAME ~ /pcre-4/ && FNR >= 19 && FNR <= 22 { next }

        // {
            if ($3 == "NULL") {
                $3 = ""
            }

            expected = 0
            if ($4 == "NOMATCH") {
                expected = 1
            } else if ($4 ~ /E[A-Z]+|BAD/ || $5 ~ /E[A-Z]+|BAD/) {
                expected = 2
            }
        }

        # testing only against the extended regular expressions flavor
        $1 == "E" || $1 == "BE" || $1 == "Ez" {
            status = system("echo '\''" $3 "'\'' | ./trex '\''" $2 "'\'' >/dev/null")
            if (status != expected) {
                failed++
                printf("%s:%s \t FAILED: \t %s \t %s\n", FILENAME, FNR, $2, $3)
            } else {
                passed++
                # printf("%s:%s \t OK: \t %s \t %s\n", FILENAME, FNR, $2, $3)
            }
        }

        $1 == "Ei" || $1 == "Ezi" {
            status = system("echo '\''" $3 "'\'' | ./trex -i '\''" $2 "'\'' >/dev/null")
            if (status != expected) {
                failed++
                printf("%s:%s \t FAILED: \t %s \t %s\n", FILENAME, FNR, $2, $3)
            } else {
                passed++
                # printf("%s:%s \t OK: \t %s \t %s\n", FILENAME, FNR, $2, $3)
            }
        }

        END {
            print("================")
            print("PASSED: ", passed)
            print("FAILED: ", failed)
        }' \
        data/*.dat
