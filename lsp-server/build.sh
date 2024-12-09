docker build -f lsp-server/Dockerfile -t lsp-server . &&
(docker rm temp-container || true) &&
docker create --name temp-container lsp-server &&
docker cp temp-container:/usr/src/app/output/lsp-server lsp-server/vscode_extension/vscode-server