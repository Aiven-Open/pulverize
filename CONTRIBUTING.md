# Contributing

When contributing please keep this in mind:

- Open an issue to discuss new bigger features.
- Write code consistent with the project style and make sure the tests are passing.
- Stay in touch with us if we have followup questions or requests for further changes.

# Development

## Tests
```shell
RUST_BACKTRACE=1 cargo test
```

## Static checking and Linting
```shell
cargo clippy
```

## Manual testing
```shell
cargo run --release -- --input-file SOME_LARGE_FILE
```

# Opening a PR

- Commit messages should describe the changes, not the filenames.
- Choose a meaningful title for your pull request.
- The pull request description should focus on what changed and why.
- Check that the tests pass (and add test coverage for your changes).