read -s -p "Enter password: " SSHPASS
echo ""
export SSHPASS

source lsp-server/build.sh &&

server_ip=5.39.249.52 &&

sshpass -e ssh root@$server_ip "rm -rf /usr/src/vscode_extension" &&
sshpass -e rsync -avz --exclude 'node_modules' --exclude 'out' --exclude 'package-lock.json' lsp-server/vscode_extension root@$server_ip:/usr/src/ &&
sshpass -e ssh root@$server_ip "cd /usr/src/vscode_extension && ./vscode-server/build_and_run.sh --restart"