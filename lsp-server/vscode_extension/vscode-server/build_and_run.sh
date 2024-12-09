docker build -f vscode-server/Dockerfile -t lsp-code-server . &&
docker stop lsp-code-server
docker rm lsp-code-server

if [ "$1" == "--restart" ]; then
    docker run --restart always -d -p 80:80 --name lsp-code-server lsp-code-server
else
    docker run -d -p 80:80 --name lsp-code-server lsp-code-server
fi