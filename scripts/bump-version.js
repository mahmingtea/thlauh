import fs from 'fs';
import path from 'path';

// Get the new version from the command-line arguments
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Error: Please provide a version number (e.g. bun run bump 0.0.3)');
  process.exit(1);
}

// Validate semver format (X.Y.Z)
if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error(`Error: "${newVersion}" is not a valid semver version (X.Y.Z)`);
  process.exit(1);
}

const rootDir = path.resolve(import.meta.dirname, '..');

// 1. Update package.json
const packageJsonPath = path.join(rootDir, 'package.json');
try {
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  packageJson.version = newVersion;
  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n', 'utf8');
  console.log(`✓ Updated package.json version to ${newVersion}`);
} catch (err) {
  console.error('Failed to update package.json:', err.message);
}

// 2. Update src-tauri/tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
try {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = newVersion;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  console.log(`✓ Updated tauri.conf.json version to ${newVersion}`);
} catch (err) {
  console.error('Failed to update tauri.conf.json:', err.message);
}

// 3. Update src-tauri/Cargo.toml (using regex to preserve comments and layout)
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
try {
  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
  if (!/^(version\s*=\s*")[^"]+(")/m.test(cargoToml)) {
    throw new Error('Could not find version field under [package]');
  }
  cargoToml = cargoToml.replace(/^(version\s*=\s*")[^"]+(")/m, `$1${newVersion}$2`);
  
  fs.writeFileSync(cargoTomlPath, cargoToml, 'utf8');
  console.log(`✓ Updated Cargo.toml version to ${newVersion}`);
} catch (err) {
  console.error('Failed to update Cargo.toml:', err.message);
}

// 4. Update updater/latest.json
const latestJsonPath = path.join(rootDir, 'updater', 'latest.json');
try {
  if (fs.existsSync(latestJsonPath)) {
    const latestJson = JSON.parse(fs.readFileSync(latestJsonPath, 'utf8'));
    const oldVersion = latestJson.version.replace(/^v/, '');
    latestJson.version = `v${newVersion}`;
    
    // Dynamically replace old version occurrences in the URLs
    if (latestJson.platforms) {
      for (const platform in latestJson.platforms) {
        let url = latestJson.platforms[platform].url;
        if (url) {
          // Replace both vX.Y.Z and X.Y.Z in the URL
          url = url.replace(new RegExp(`v${oldVersion}`, 'g'), `v${newVersion}`);
          url = url.replace(new RegExp(`_${oldVersion}_`, 'g'), `_${newVersion}_`);
          latestJson.platforms[platform].url = url;
        }
      }
    }
    
    // Update pub_date to current time
    latestJson.pub_date = new Date().toISOString();
    
    fs.writeFileSync(latestJsonPath, JSON.stringify(latestJson, null, 2) + '\n', 'utf8');
    console.log(`✓ Updated updater/latest.json version to v${newVersion} and updated release URLs`);
  }
} catch (err) {
  console.error('Failed to update updater/latest.json:', err.message);
}

console.log('\nAll versions bumped successfully! Run a build or cargo check to sync Cargo.lock.');
