green := "\\033[32m"
yellow := "\\033[33m"
reset := "\\033[0m"

# generate builtins
gen:
    echo 'builtins'

# catchall lints
lint:
    typos
    just --fmt --check
    cargo clippy --all-targets --all-features -- -W clippy::unused_async -W clippy::uninlined_format_args -W clippy::unnecessary_mut_passed

# formats rust and justfile code
fmt:
    cargo fmt
    just --fmt --unstable

# updates cargo and dist dependencies
update:
    cargo upgrade && cargo update
    dist init -y

# updates node package.json to latest available
outdated:
    @printf '{{ yellow }}={{ reset }}Cargo{{ yellow }}={{ reset }}\n'
    cargo outdated -d 1
    @printf '{{ yellow }}======={{ reset }}\n'
