# This is a template for my Bevy projects, which have to be build with VSCode on Windows.

## **Crates**

1. **Bevy**
2. **bevy_embedded_assets**: embed assets folder inside binary files (including web).
3. **bevy_asset_loader**: load assets in a specific state and automatically transition to next state. 

## Details

Project contains ball.png asset file and main.rs file with code for scene setup.

_.vscode_ contains *tasks.json* that is used for executing rust code with shortcut: <code>CTRL + SHIFT + B</code>.

_.cargo_ contains code for dynamic linking and faster compiling.

_rust-toolchain.toml_ is used for turning on the rust nightly compiler.

## WASM Build

**Run the following to build for web:** 

TODO: add variable into cli to replace bevy_project_template name in commands.

1. Install or update _**wasm-bindgen**_ , _**wasm-bindgen-cli**_ , **_wasm-opt_** dependencies:
   - <code>cargo install wasm-bindgen</code>
   - <code>cargo install wasm-bindgen-cli</code>
   - <code>cargo install wasm-opt</code>
3. Comment dynamic linking in **cargo.toml** and execute <code>cargo build --release --target wasm32-unknown-unknown</code>.
4. Make javascript bindings using <code> wasm-bindgen --target web --out-dir ./wasm_out/ --out-name "bevy_project_template" ./target/wasm32-unknown-unknown/release/bevy_project_template.wasm </code>
5. Optimise the wasm build <code>wasm-opt -Oz out/bevy_project_template_bg.wasm --output out/bevy_project_template_bg.wasm</code>
6. Inside wasm_out folder add index.html file. The contents can be any, but the file must include this: TODO.
7. Optional (for Itch.io): zip produced files into an archive <code> zip -FSj wasm_out/bevy_project_template.zip wasm_out/* </code> TODO

### Run locally

Reference: https://bevy-cheatbook.github.io/platforms/wasm.html

1. Update nightly toolchain: <code> rustup update </code>
2. Install wasm-server: <code> cargo install wasm-server-runner </code>
3. Add target to Cargo.toml file:
   - <code> [target.wasm32-unknown-unknown] </code>
   - <code> runner = "wasm-server-runner" </code>
3. <code> cargo run --target wasm32-unknown-unknown </code>
