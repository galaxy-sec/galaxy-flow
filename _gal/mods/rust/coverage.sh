echo "TARGET": $TARGET
grcov -V
export TARGET="target/coverage"
export RUSTFLAGS="-Cinstrument-coverage"
LLVM_PROFILE_FILE="${TARGET}/coverage-%p-%m.profraw" cargo test --all
grcov ${TARGET} --binary-path ./target/debug/ -s . -t lcov --branch --llvm --ignore-not-existing --ignore "*cargo*" --ignore "*target*" -o coverage.info
python3 /usr/local/lib/python3.10/dist-packages/lcov_cobertura/lcov_cobertura.py coverage.info
