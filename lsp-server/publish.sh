read -s -p "Enter password: " SSHPASS
echo ""
export SSHPASS

source lsp-server/build.sh &&

sshpass -e ssh root@195.133.21.99 "rm -rf /usr/src/vscode_extension" &&
sshpass -e rsync -avz --exclude 'node_modules' --exclude 'out' --exclude 'package-lock.json' lsp-server/vscode_extension root@195.133.21.99:/usr/src/ &&
sshpass -e ssh root@195.133.21.99 "cd /usr/src/vscode_extension && ./vscode-server/build_and_run.sh --restart"