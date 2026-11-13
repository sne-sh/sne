#!/usr/bin/env node
// Starts the sne program that npm, yarn, pnpm or bun installed for this system.
// The program itself is in a separate package, one per system, listed in optionalDependencies.
// Written for old Node.js versions too: sne may switch to any of them.
'use strict';

var fs = require('fs');
var path = require('path');
var spawnSync = require('child_process').spawnSync;

var name = 'sne-' + process.platform + '-' + process.arch;
var binary;
try {
  binary = path.join(
    path.dirname(require.resolve(name + '/package.json')),
    process.platform === 'win32' ? 'sne.exe' : 'sne'
  );
} catch (err) {
  console.error(
    'sne: the package ' + name + ' is not installed.\n' +
    'Either sne has no build for this system (' + process.platform + ' ' + process.arch + '),\n' +
    'or optional dependencies were skipped. Install again without --no-optional or --omit=optional,\n' +
    'or use another way to install: https://github.com/sne-sh/sne#install'
  );
  process.exit(1);
}

if (process.platform !== 'win32') {
  // Some tools unpack files without the executable bit.
  try {
    fs.accessSync(binary, fs.constants.X_OK);
  } catch (err) {
    try {
      fs.chmodSync(binary, 493); // 0755
    } catch (ignored) {}
  }
}

var result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' });
if (result.error) {
  console.error('sne: could not run ' + binary + ': ' + result.error.message);
  process.exit(1);
}
process.exit(result.status === null ? 1 : result.status);
