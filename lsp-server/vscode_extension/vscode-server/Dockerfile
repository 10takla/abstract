FROM ubuntu:20.04

RUN apt-get update && apt-get install -y curl build-essential

RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs

RUN curl -fsSL https://code-server.dev/install.sh | sh

COPY vscode-server/settings.json /root/.local/share/code-server/User/settings.json
COPY vscode-server/darcula-dark-color-theme.json /root/.local/share/code-server/User/darcula-dark-color-theme.json

RUN apt-get install -y wget unzip fontconfig && \
    wget https://download.jetbrains.com/fonts/JetBrainsMono-2.304.zip -P /tmp/ && \
    unzip /tmp/JetBrainsMono-2.304.zip -d /usr/share/fonts/jetbrains-mono && \
    fc-cache -fv

WORKDIR /usr/src/app

COPY . .

ENV LSP_SERVER_PATH="/usr/src/app/vscode-server/lsp-server"

RUN npm install
RUN npm run compile
RUN npm install -g vsce --unsafe-perm --loglevel verbose && vsce package
RUN code-server --install-extension *.vsix

EXPOSE 80

CMD ["code-server", "--bind-addr", "0.0.0.0:80", "--auth", "none", "--extensions-dir", "/root/.local/share/code-server/extensions", "examples"]