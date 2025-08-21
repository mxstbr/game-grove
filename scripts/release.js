#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { createInterface } from 'readline';

const rl = createInterface({
  input: process.stdin,
  output: process.stdout
});

function question(query) {
  return new Promise(resolve => rl.question(query, resolve));
}

function incrementVersion(version, type) {
  const parts = version.split('.').map(Number);
  
  switch (type) {
    case 'major':
      parts[0]++;
      parts[1] = 0;
      parts[2] = 0;
      break;
    case 'minor':
      parts[1]++;
      parts[2] = 0;
      break;
    case 'patch':
      parts[2]++;
      break;
    default:
      throw new Error('Invalid version type. Must be major, minor, or patch.');
  }
  
  return parts.join('.');
}

function updatePackageJson(newVersion) {
  const packagePath = './package.json';
  const packageJson = JSON.parse(readFileSync(packagePath, 'utf8'));
  packageJson.version = newVersion;
  writeFileSync(packagePath, JSON.stringify(packageJson, null, 2) + '\n');
  console.log(`✅ Updated package.json to version ${newVersion}`);
}

function updateCargoToml(newVersion) {
  const cargoPath = './src-tauri/Cargo.toml';
  const cargoContent = readFileSync(cargoPath, 'utf8');
  const updatedContent = cargoContent.replace(
    /^version = ".*"$/m,
    `version = "${newVersion}"`
  );
  writeFileSync(cargoPath, updatedContent);
  console.log(`✅ Updated Cargo.toml to version ${newVersion}`);
}

function runCommand(command) {
  try {
    execSync(command, { stdio: 'inherit' });
    return true;
  } catch (error) {
    console.error(`❌ Command failed: ${command}`);
    console.error(error.message);
    return false;
  }
}

async function main() {
  try {
    // Check if we're in a clean git state
    try {
      const status = execSync('git status --porcelain', { encoding: 'utf8' });
      if (status.trim()) {
        console.error('❌ Working directory is not clean. Please commit or stash your changes first.');
        process.exit(1);
      }
    } catch (error) {
      console.error('❌ Failed to check git status:', error.message);
      process.exit(1);
    }

    // Read current version
    const packageJson = JSON.parse(readFileSync('./package.json', 'utf8'));
    const currentVersion = packageJson.version;
    
    console.log(`Current version: ${currentVersion}`);
    console.log('');
    
    // Ask for version bump type
    const bumpType = await question('Select version bump type (patch/minor/major): ');
    
    if (!['patch', 'minor', 'major'].includes(bumpType)) {
      console.error('❌ Invalid selection. Please choose patch, minor, or major.');
      process.exit(1);
    }
    
    // Calculate new version
    const newVersion = incrementVersion(currentVersion, bumpType);
    console.log(`New version will be: ${newVersion}`);
    console.log('');
    
    // Confirm
    const confirm = await question(`Proceed with release ${newVersion}? (y/N): `);
    if (confirm.toLowerCase() !== 'y' && confirm.toLowerCase() !== 'yes') {
      console.log('❌ Release cancelled.');
      process.exit(0);
    }
    
    console.log('');
    console.log('🚀 Starting release process...');
    console.log('');
    
    // Update version files
    updatePackageJson(newVersion);
    updateCargoToml(newVersion);
    
    // Stage changes
    if (!runCommand('git add package.json src-tauri/Cargo.toml')) {
      process.exit(1);
    }
    console.log('✅ Staged version changes');
    
    // Commit changes
    if (!runCommand(`git commit -m "chore: bump version to ${newVersion}"`)) {
      process.exit(1);
    }
    console.log('✅ Committed version changes');
    
    // Create tag
    if (!runCommand(`git tag -a v${newVersion} -m "Release v${newVersion}"`)) {
      process.exit(1);
    }
    console.log(`✅ Created tag v${newVersion}`);
    
    // Push changes and tag
    if (!runCommand('git push origin main')) {
      process.exit(1);
    }
    console.log('✅ Pushed changes to main');
    
    if (!runCommand(`git push origin v${newVersion}`)) {
      process.exit(1);
    }
    console.log(`✅ Pushed tag v${newVersion}`);
    
    console.log('');
    console.log(`🎉 Successfully released version ${newVersion}!`);
    
  } catch (error) {
    console.error('❌ Release failed:', error.message);
    process.exit(1);
  } finally {
    rl.close();
  }
}

main();
