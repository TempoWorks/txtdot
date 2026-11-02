const fs = require('node:fs');
const path = require('node:path');
const CleanCSS = require('clean-css');

fs.mkdirSync('dist/static', { recursive: true });

for (const file of fs.globSync('static/*.css')) {
  const output = path.join('dist/static', path.basename(file));
  const result = new CleanCSS({ rebaseTo: path.dirname(output) }).minify([
    file,
  ]);
  if (result.errors.length) {
    throw new Error(result.errors.join('\n'));
  }
  fs.writeFileSync(output, result.styles);
}

for (const file of fs.globSync('templates/**/*.ejs')) {
  const output = path.join('dist', file);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.copyFileSync(file, output);
}

fs.cpSync('static/third-party', 'dist/static/third-party', { recursive: true });

if (fs.existsSync('.env')) {
  fs.copyFileSync('.env', 'dist/.env');
}
