# remember previous line to replace SAME in the tests
// {
    FS = " "
    if ($2 == "SAME") {
        $2=prev
    } else {
        prev=$2
    }
}

# character encoding issues (non utf-8)
FILENAME ~ /pcre-4/ { next }
FILENAME ~ /rxposix|pcre-/ { FS = "\t" }
# those lines failed to parse for some reason, so skipping them
$3 == "" { next }
# self-references etc are not supported
$2 ~ /\\[1-9QEAZGUzZ]/ { next }
# lazy matching is not supported
$2 ~ /[+*?]+/ { next }
# character classes are not supported
$2 ~ /\[[^]]*\[[:.=]/ { next }
# special modifiers are not supported
$2 ~ /\(\?[<>:=!i]/ { next }
# word boundaries of this form are not supported
$2 ~ /\\<|\\>/ { next }
# dont care about empty branches in alternation
$4 == "ENULL" || $4 == "BADESC" { next }

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
}
