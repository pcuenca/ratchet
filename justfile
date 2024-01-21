line-count:
    cd ./crates/ratchet-core && scc -irs --exclude-file kernels
install-pyo3:
    env PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install --verbose 3.10.6
    echo "Please PYO3_PYTHON to your .bashrc or .zshrc"
wasm CRATE:
    RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web -d `pwd`/target/pkg/{{CRATE}} --out-name {{CRATE}} ./crates/{{CRATE}} --release 
wasm-test CRATE:
  RUSTFLAGS="--cfg=web_sys_unstable_apis -Z threads=8" wasm-pack test --chrome `pwd`/crates/{{CRATE}}
wasm-test-headless CRATE:
  RUSTFLAGS="--cfg=web_sys_unstable_apis -Z threads=8" wasm-pack test --chrome --headless `pwd`/crates/{{CRATE}}
