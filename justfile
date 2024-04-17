build: build-wasm build-web build-raycast build-extension

build-wasm:
    rm -rf ./pkg
    wasm-pack build -t web --no-default-features --features wasm

build-web:
    rm -rf ./web/dist
    pnpm run -C web build

build-raycast:
    rm -rf ./raycast/dist
    cp ./pkg/orgwise_bg.wasm ./raycast/assets/orgwise_bg.wasm
    pnpm run -C raycast build

build-extension:
    rm -rf ./vscode/dist
    mkdir -p ./vscode/dist
    cp -r ./web/dist ./vscode/dist/web
    cp ./pkg/orgwise_bg.wasm ./vscode/dist/orgwise_bg.wasm
    pnpm run -C vscode build
    pnpm run -C vscode package --no-dependencies

install-extension:
    code --install-extension ./vscode/orgwise.vsix --force

install-raycast:
    pnpm run -C raycast install-local

dev-raycast:
    pnpm run -C raycast dev

dev-web:
    pnpm run -C web dev
