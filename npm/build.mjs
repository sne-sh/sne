// Builds the npm packages for a published sne release into npm/dist/:
//
//   sne-cli             the launcher (npm/sne), with the packages below as optionalDependencies
//   sne-<os>-<cpu>      one per system, holding the sne program from the release
//
//   node npm/build.mjs 0.1.0-beta.4             build only
//   node npm/build.mjs 0.1.0-beta.4 --publish   build, then publish to npm what is not there yet
//
// The programs come from the release on GitHub and are checked against its SHA256SUMS.txt, so the
// release must be published first. Needs Node.js 18 or newer.

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.join(here, 'dist');
const repo = 'https://github.com/sne-sh/sne';

// npm's names for the system and processor, and the release file built for them.
// The Linux builds are static, so one works on every distribution, glibc or musl.
const systems = [
  { os: 'win32', cpu: 'x64', target: 'x86_64-pc-windows-msvc', label: 'Windows x64' },
  { os: 'win32', cpu: 'ia32', target: 'i686-pc-windows-msvc', label: 'Windows 32-bit' },
  { os: 'win32', cpu: 'arm64', target: 'aarch64-pc-windows-msvc', label: 'Windows ARM64' },
  { os: 'darwin', cpu: 'x64', target: 'x86_64-apple-darwin', label: 'macOS Intel' },
  { os: 'darwin', cpu: 'arm64', target: 'aarch64-apple-darwin', label: 'macOS Apple silicon' },
  { os: 'linux', cpu: 'x64', target: 'x86_64-unknown-linux-musl', label: 'Linux x64' },
  { os: 'linux', cpu: 'arm64', target: 'aarch64-unknown-linux-musl', label: 'Linux ARM64' },
];

const args = process.argv.slice(2);
const version = args.find((arg) => !arg.startsWith('--'));
const publish = args.includes('--publish');
if (!version) {
  console.error('usage: node npm/build.mjs <version> [--publish]');
  process.exit(1);
}

async function download(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}

function writeJson(file, value) {
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n');
}

function unpack(archive, into) {
  if (process.platform === 'win32') {
    // The tar that comes with Windows reads both formats. Git's tar does not understand `C:`.
    const tar = path.join(process.env.SystemRoot || 'C:\\Windows', 'System32', 'tar.exe');
    execFileSync(tar, ['-xf', archive, '-C', into]);
  } else if (archive.endsWith('.zip')) {
    execFileSync('unzip', ['-q', archive, '-d', into]);
  } else {
    execFileSync('tar', ['-xzf', archive, '-C', into]);
  }
}

const downloads = `${repo}/releases/download/${version}`;
const sums = new Map(
  (await download(`${downloads}/SHA256SUMS.txt`))
    .toString()
    .split('\n')
    .map((line) => line.trim().split(/\s+/))
    .filter((parts) => parts.length === 2)
    .map(([hash, name]) => [name.replace(/^\*/, ''), hash.toLowerCase()])
);

fs.rmSync(dist, { recursive: true, force: true });
const work = path.join(dist, '.work');
fs.mkdirSync(work, { recursive: true });

const shared = {
  version,
  homepage: 'https://sne-sh.github.io/sne/',
  bugs: `${repo}/issues`,
  repository: { type: 'git', url: 'git+https://github.com/sne-sh/sne.git' },
  license: 'MIT',
};
const packages = [];
// npm refuses the name "sne", so the launcher is published as sne-cli. Its command is still `sne`.
const launcherName = JSON.parse(fs.readFileSync(path.join(here, 'sne', 'package.json'), 'utf8')).name;

for (const system of systems) {
  const name = `sne-${system.os}-${system.cpu}`;
  const folder = `sne-${version}-${system.target}`;
  const archive = `${folder}.${system.os === 'win32' ? 'zip' : 'tar.gz'}`;
  const expected = sums.get(archive);
  if (!expected) throw new Error(`${archive} is not listed in SHA256SUMS.txt of ${version}`);
  const bytes = await download(`${downloads}/${archive}`);
  const actual = createHash('sha256').update(bytes).digest('hex');
  if (actual !== expected) throw new Error(`checksum mismatch for ${archive}`);

  fs.writeFileSync(path.join(work, archive), bytes);
  unpack(path.join(work, archive), work);

  const program = system.os === 'win32' ? 'sne.exe' : 'sne';
  const dir = path.join(dist, name);
  fs.mkdirSync(dir);
  fs.copyFileSync(path.join(work, folder, program), path.join(dir, program));
  fs.copyFileSync(path.join(work, folder, 'LICENSE'), path.join(dir, 'LICENSE'));
  if (system.os !== 'win32') fs.chmodSync(path.join(dir, program), 0o755);
  fs.writeFileSync(
    path.join(dir, 'README.md'),
    `# ${name}\n\nThe sne program for ${system.label}. Install ` +
      `[${launcherName}](https://www.npmjs.com/package/${launcherName}) ` +
      `instead: it picks this package by itself.\n`
  );
  writeJson(path.join(dir, 'package.json'), {
    name,
    ...shared,
    description: `The sne program for ${system.label}. Install the package "${launcherName}" instead.`,
    // A bin entry makes package managers mark the file as executable, whichever system packed it.
    // Its name differs from "sne", so it never competes with the launcher.
    bin: { [name]: program },
    os: [system.os],
    cpu: [system.cpu],
  });
  packages.push(name);
  console.log(`built ${name}`);
}

fs.rmSync(work, { recursive: true, force: true });

const launcher = path.join(dist, 'sne');
fs.cpSync(path.join(here, 'sne'), launcher, { recursive: true });
fs.copyFileSync(path.join(here, '..', 'LICENSE'), path.join(launcher, 'LICENSE'));
const manifest = JSON.parse(fs.readFileSync(path.join(launcher, 'package.json'), 'utf8'));
manifest.version = version;
manifest.optionalDependencies = Object.fromEntries(packages.map((name) => [name, version]));
writeJson(path.join(launcher, 'package.json'), manifest);
console.log(`built ${launcherName}`);

if (publish) {
  // The system packages go first, so the launcher never points at a package that is missing.
  // A beta is published as "latest" too: while there is no stable release, it is what
  // `npm install -g sne` should give.
  // A package that is already on the registry is skipped, so a run that stopped can be repeated.
  for (const [folder, name] of [...packages.map((p) => [p, p]), ['sne', launcherName]]) {
    const there = await fetch(`https://registry.npmjs.org/${name}/${version}`);
    if (there.ok) {
      console.log(`${name}@${version} is already on npm`);
      continue;
    }
    execFileSync('npm', ['publish', '--access', 'public', '--tag', 'latest'], {
      cwd: path.join(dist, folder),
      stdio: 'inherit',
      shell: process.platform === 'win32',
    });
  }
} else {
  console.log(`\nPackages are in ${dist}. Publish them with: node npm/build.mjs ${version} --publish`);
}
