// 跨平台启动 Tauri 构建，并修复 Windows 上 WiX light.exe 打包 MSI 时的
// "文件正由另一进程使用" (System.IO.IOException / LGHT0001) 竞态问题。
//
// 原因：WiX 默认在系统 %TEMP% 下创建临时 CAB 文件，杀软实时扫描 / 系统索引
// 服务等可能在这一瞬间持有该文件句柄，导致 light.exe 独占写入失败。
// 解法：把 Bundler 工具临时目录指向项目 target 内一个不被扫描的独立目录。
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const tmpDir = resolve(root, 'src-tauri', 'target', '.bundler-tmp');
mkdirSync(tmpDir, { recursive: true });

const env = { ...process.env, TAURI_BUNDLER_TOOLS_TEMP_DIR: tmpDir };

const args = ['build', ...process.argv.slice(2)];
const cmd = process.platform === 'win32' ? 'tauri.cmd' : 'tauri';

const child = spawn(cmd, args, { stdio: 'inherit', env, cwd: root, shell: true });
child.on('exit', (code) => process.exit(code ?? 1));
