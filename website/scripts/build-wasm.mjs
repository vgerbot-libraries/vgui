import { spawnSync } from 'node:child_process';
import { readFile, stat } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const websiteRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = resolve(websiteRoot, '..');
const cargo = process.platform === 'win32' ? 'cargo.cmd' : 'cargo';
const examples = [
  { package: 'vgui', example: 'counter' },
  { package: 'vgui-shadcn', example: 'themed_view' },
  { package: 'vgui-shadcn', example: 'click_button' },
  { package: 'vgui-shadcn', example: 'profile_form' },
  { package: 'vgui-shadcn', example: 'language_select' },
  { package: 'vgui-shadcn', example: 'project_card' },
  { package: 'vgui-shadcn', example: 'package_details' },
  { package: 'vgui-shadcn', example: 'team_table' },
  { package: 'vgui-shadcn', example: 'breadcrumb' },
  { package: 'vgui-shadcn', example: 'pagination' },
  { package: 'vgui-shadcn', example: 'tabs' },
  { package: 'vgui-shadcn', example: 'h_resizable' },
  { package: 'vgui-shadcn', example: 'dialog' },
  { package: 'vgui-shadcn', example: 'popover' },
  { package: 'vgui-shadcn', example: 'alert' },
  { package: 'vgui-shadcn', example: 'progress' },
];

function run(command, args) {
  const result = spawnSync(command, args, { cwd: repositoryRoot, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const lockfile = await readFile(resolve(repositoryRoot, 'Cargo.lock'), 'utf8');
const bindgenVersion = lockfile.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/)?.[1];
if (!bindgenVersion) throw new Error('Cannot determine wasm-bindgen version from Cargo.lock.');

const cliResult = spawnSync('wasm-bindgen', ['--version'], { encoding: 'utf8' });
if (cliResult.error) throw cliResult.error;
if (cliResult.status !== 0) process.exit(cliResult.status ?? 1);
const cliVersion = cliResult.stdout.trim().split(/\s+/)[1];
if (cliVersion !== bindgenVersion) {
  throw new Error(`wasm-bindgen CLI ${cliVersion} does not match Cargo.lock ${bindgenVersion}. Install it with: cargo install wasm-bindgen-cli --version ${bindgenVersion} --locked`);
}

for (const { package: packageName, example } of examples) {
  const output = resolve(websiteRoot, `public/wasm/${example}`);
  const wasm = resolve(
    repositoryRoot,
    `target/wasm32-unknown-unknown/release/examples/${example}.wasm`,
  );

  run(cargo, [
    '+nightly',
    'build',
    '--target',
    'wasm32-unknown-unknown',
    '-p',
    packageName,
    '--example',
    example,
    '--release',
    '--locked',
  ]);
  run('wasm-bindgen', [
    '--target',
    'web',
    '--out-dir',
    output,
    '--out-name',
    example,
    '--no-typescript',
    wasm,
  ]);

  await stat(resolve(output, `${example}.js`));
  await stat(resolve(output, `${example}_bg.wasm`));
  console.log(`WASM ${example} added to website/public/wasm/${example}.`);
}
