if [ "$1" = "clean" ]; then
    cargo clean --doc
fi

cargo doc --no-deps --package lexer

touch target/doc/index.html
echo "<meta http-equiv=\"refresh\" content=\"0; url=lexer\">" > target/doc/index.html

gh-pages -d target/doc/ --remote gh-pages