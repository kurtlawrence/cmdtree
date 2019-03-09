set -e	# stop on failure
git pull  # maybe git reset --hard master if Cargo.lock playing up
cargo test
cargo tarpaulin -v -l --out Html
mv tarpaulin-report.html /mnt/c/users/kurt/desktop/tarpaulin-report.html