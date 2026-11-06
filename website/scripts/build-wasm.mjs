import { spawnSync } from 'node:child_process';
import { readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const websiteRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = resolve(websiteRoot, '..');
const output = resolve(websiteRoot, 'public/wasm/counter');
const wasm = resolve(repositoryRoot, 'target/wasm32-unknown-unknown/release/examples/counter.wasm');
const cargo = process.platform === 'win32' ? 'cargo.cmd' : 'cargo';

function run(command, args) {
  const result = spawnSync(command, args, { cwd: repositoryRoot, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const lockfile = await readFile(resolve(repositoryRoot, 'Cargo.lock'), 'utf8');
const bindgenVersion = lockfile.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/)?.[1];
if (!bindgenVersion) throw new Error('Cannot determine wasm-bindgen version from Cargo.lock.');

run(cargo, ['+nightly', 'build', '--target', 'wasm32-unknown-unknown', '-p', 'vgui', '--example', 'counter', '--release', '--locked']);
const cliResult = spawnSync('wasm-bindgen', ['--version'], { encoding: 'utf8' });
if (cliResult.error) throw cliResult.error;
if (cliResult.status !== 0) process.exit(cliResult.status ?? 1);
const cliVersion = cliResult.stdout.trim().split(/\s+/)[1];
if (cliVersion !== bindgenVersion) {
  throw new Error(`wasm-bindgen CLI ${cliVersion} does not match Cargo.lock ${bindgenVersion}. Install it with: cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked`);
}
run('wasm-bindgen', ['--target', 'web', '--out-dir', output, '--out-name', 'counter', '--no-typescript', wasm]);

await stat(resolve(output, 'counter.js'));
await stat(resolve(output, 'counter_bg.wasm'));
console.log('WASM counter added to website/public/wasm/counter.');
