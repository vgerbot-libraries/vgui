import { spawnSync } from 'node:child_process';
import { cp, mkdir, rm, stat } from 'node:fs/promises';
import { basename, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const websiteRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = resolve(websiteRoot, '..');
const rustdocOutput = resolve(repositoryRoot, 'target/doc');
const siteApiOutput = resolve(websiteRoot, 'dist/api');

const cargo = process.platform === 'win32' ? 'cargo.cmd' : 'cargo';
const result = spawnSync(cargo, ['doc', '--workspace', '--no-deps', '--locked'], {
  cwd: repositoryRoot,
  stdio: 'inherit',
});

if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);

await stat(resolve(rustdocOutput, 'vgui/index.html'));
await rm(siteApiOutput, { recursive: true, force: true });
await mkdir(siteApiOutput, { recursive: true });
await cp(rustdocOutput, siteApiOutput, {
  recursive: true,
  filter(source) {
    return !['.cargo-ok', '.lock'].includes(basename(source));
  },
});
await stat(resolve(siteApiOutput, 'vgui/index.html'));

console.log('Rustdoc added to website/dist/api (outside the Pagefind index).');
