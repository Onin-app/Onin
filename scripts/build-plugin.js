#!/usr/bin/env node

/**
 * Plugin Build Script
 * 
 * This script builds a plugin by compiling TypeScript and creating a distribution package.
 * It can be used for individual plugins or all plugins in the workspace.
 */

import { execSync } from 'child_process';
import { existsSync, readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'fs';
import { join, dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
import { createPluginPackage, PACKAGE_FORMATS } from './package-utils.js';
import { buildConfig } from './plugin-build.config.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration
const WORKSPACE_ROOT = resolve(__dirname, '..');
const PLUGINS_DIR = join(WORKSPACE_ROOT, 'plugins');

/**
 * Build a single plugin
 * @param {string} pluginPath - Path to the plugin directory
 * @param {object} options - Build options
 */
async function buildPlugin(pluginPath, options = {}) {
  const pluginName = pluginPath.split('/').pop() || pluginPath.split('\\').pop();
  console.log(`🔨 Building plugin: ${pluginName}`);

  try {
    // Check if plugin has package.json
    const packageJsonPath = join(pluginPath, 'package.json');
    if (!existsSync(packageJsonPath)) {
      console.warn(`⚠️  No package.json found in ${pluginName}, skipping...`);
      return false;
    }

    // Check if plugin has TypeScript source
    const srcPath = join(pluginPath, 'src');
    if (!existsSync(srcPath)) {
      console.warn(`⚠️  No src directory found in ${pluginName}, skipping...`);
      return false;
    }

    // Clean previous build
    if (options.clean) {
      const distPath = join(pluginPath, 'dist');
      if (existsSync(distPath)) {
        console.log(`🧹 Cleaning ${pluginName}/dist`);
        execSync(`rimraf "${distPath}"`, { cwd: pluginPath });
      }
    }

    // Install dependencies if needed
    if (options.install) {
      console.log(`📦 Installing dependencies for ${pluginName}`);
      execSync('pnpm install', { cwd: pluginPath, stdio: 'inherit' });
    }

    // Run TypeScript compilation
    console.log(`⚙️  Compiling TypeScript for ${pluginName}`);
    execSync('npx tsc', { cwd: pluginPath, stdio: options.verbose ? 'inherit' : 'pipe' });

    // Validate plugin manifest
    const manifestPath = join(pluginPath, 'plugin.json');
    if (existsSync(manifestPath)) {
      validatePluginManifest(manifestPath, pluginName);
    } else {
      console.warn(`⚠️  No plugin.json manifest found in ${pluginName}`);
    }

    // Copy additional files to dist if needed
    copyAdditionalFiles(pluginPath, options);

    console.log(`✅ Successfully built ${pluginName}`);
    return true;

  } catch (error) {
    console.error(`❌ Failed to build ${pluginName}:`, error.message);
    if (options.verbose) {
      console.error(error.stack);
    }
    return false;
  }
}

/**
 * Validate plugin manifest
 * @param {string} manifestPath - Path to plugin.json
 * @param {string} pluginName - Plugin name for error reporting
 */
function validatePluginManifest(manifestPath, pluginName) {
  try {
    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    
    // Required fields
    const requiredFields = ['name', 'version', 'description', 'author', 'main', 'permissions', 'engines'];
    const missingFields = requiredFields.filter(field => !(field in manifest));
    
    if (missingFields.length > 0) {
      throw new Error(`Missing required fields: ${missingFields.join(', ')}`);
    }

    // Validate field types and formats
    validateManifestFields(manifest);

    // Validate main file exists
    const mainFile = join(dirname(manifestPath), manifest.main);
    if (!existsSync(mainFile)) {
      throw new Error(`Main file not found: ${manifest.main}`);
    }

    // Validate engines.baize
    if (!manifest.engines.baize) {
      throw new Error('Missing engines.baize version requirement');
    }

    // Validate version format (semver)
    if (!isValidSemver(manifest.version)) {
      throw new Error(`Invalid version format: ${manifest.version}. Must be valid semver (e.g., 1.0.0)`);
    }

    // Validate permissions
    if (!Array.isArray(manifest.permissions)) {
      throw new Error('Permissions must be an array');
    }

    console.log(`✅ Plugin manifest validated for ${pluginName}`);
    return manifest;
    
  } catch (error) {
    console.error(`❌ Invalid plugin manifest in ${pluginName}:`, error.message);
    throw error;
  }
}

/**
 * Validate manifest field types and formats
 * @param {object} manifest - Plugin manifest object
 */
function validateManifestFields(manifest) {
  // Validate string fields
  const stringFields = ['name', 'version', 'description', 'author', 'main'];
  for (const field of stringFields) {
    if (typeof manifest[field] !== 'string' || manifest[field].trim() === '') {
      throw new Error(`Field '${field}' must be a non-empty string`);
    }
  }

  // Validate name format (no spaces, lowercase, hyphens allowed)
  if (!/^[a-z0-9-]+$/.test(manifest.name)) {
    throw new Error('Plugin name must contain only lowercase letters, numbers, and hyphens');
  }

  // Validate main file extension
  if (!manifest.main.endsWith('.js')) {
    throw new Error('Main file must be a JavaScript file (.js)');
  }
}

/**
 * Check if version string is valid semver
 * @param {string} version - Version string to validate
 */
function isValidSemver(version) {
  const semverRegex = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
  return semverRegex.test(version);
}

/**
 * Copy additional files to dist directory
 * @param {string} pluginPath - Path to plugin directory
 * @param {object} options - Build options
 */
function copyAdditionalFiles(pluginPath, options) {
  const distPath = join(pluginPath, 'dist');
  
  // Ensure dist directory exists
  if (!existsSync(distPath)) {
    mkdirSync(distPath, { recursive: true });
  }

  // Files to copy
  const filesToCopy = ['plugin.json', 'README.md', 'LICENSE'];
  
  for (const file of filesToCopy) {
    const srcFile = join(pluginPath, file);
    const destFile = join(distPath, file);
    
    if (existsSync(srcFile)) {
      copyFileSync(srcFile, destFile);
      if (options.verbose) {
        console.log(`📄 Copied ${file} to dist`);
      }
    }
  }
}

/**
 * Build all plugins in the workspace
 * @param {object} options - Build options
 */
async function buildAllPlugins(options = {}) {
  console.log('🚀 Building all plugins...');
  
  const { readdirSync, statSync } = await import('fs');
  
  if (!existsSync(PLUGINS_DIR)) {
    console.error('❌ Plugins directory not found');
    process.exit(1);
  }

  const pluginDirs = readdirSync(PLUGINS_DIR)
    .map(name => join(PLUGINS_DIR, name))
    .filter(path => statSync(path).isDirectory());

  let successCount = 0;
  let totalCount = pluginDirs.length;

  for (const pluginPath of pluginDirs) {
    const success = await buildPlugin(pluginPath, options);
    if (success) successCount++;
  }

  console.log(`\n📊 Build Summary: ${successCount}/${totalCount} plugins built successfully`);
  
  if (successCount < totalCount) {
    process.exit(1);
  }
}

/**
 * Create a plugin package for distribution
 * @param {string} pluginPath - Path to plugin directory
 * @param {object} options - Package options
 */
async function packagePlugin(pluginPath, options = {}) {
  const pluginName = pluginPath.split('/').pop() || pluginPath.split('\\').pop();
  console.log(`📦 Packaging plugin: ${pluginName}`);

  try {
    // Ensure plugin is built
    const distPath = join(pluginPath, 'dist');
    if (!existsSync(distPath)) {
      console.log(`🔨 Building ${pluginName} first...`);
      await buildPlugin(pluginPath, { clean: true });
    }

    // Read plugin manifest
    const manifestPath = join(pluginPath, 'plugin.json');
    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    
    // Determine package format
    const format = options.format || buildConfig.package.format || PACKAGE_FORMATS.BPKG;
    
    // Create package using the new packaging system
    const packagePath = await createPluginPackage(pluginPath, {
      format,
      outputDir: buildConfig.package.outputDir,
      includeSource: options.includeSource || false,
      generateChecksums: buildConfig.package.generateChecksums,
      compression: buildConfig.package.compression
    });

    // Generate installation instructions
    const installInstructions = generateInstallInstructions(manifest, options);
    const instructionsPath = join(dirname(packagePath), `${manifest.name}-${manifest.version}-INSTALL.md`);
    writeFileSync(instructionsPath, installInstructions);

    console.log(`✅ Successfully packaged ${pluginName}`);
    return { packagePath, manifest };

  } catch (error) {
    console.error(`❌ Failed to package ${pluginName}:`, error.message);
    return false;
  }
}



/**
 * Generate installation instructions
 * @param {object} manifest - Plugin manifest
 * @param {object} options - Package options
 */
function generateInstallInstructions(manifest, options) {
  return `# Installation Instructions for ${manifest.name}

## Automatic Installation (Recommended)

1. Download the \`${manifest.name}-${manifest.version}.bpkg\` file
2. Open Baize application
3. Go to Settings > Plugins
4. Click "Install Plugin" and select the downloaded .bpkg file
5. Enable the plugin in the plugin list

## Manual Installation

1. Extract the \`${manifest.name}-${manifest.version}.bpkg\` file
2. Copy the extracted folder to your Baize plugins directory:
   - Windows: \`%APPDATA%/Baize/plugins/\`
   - macOS: \`~/Library/Application Support/Baize/plugins/\`
   - Linux: \`~/.config/Baize/plugins/\`
3. Restart Baize
4. Enable the plugin in Settings > Plugins

## Requirements

- Baize version: ${manifest.engines.baize}
- Permissions required: ${manifest.permissions.join(', ')}

## Plugin Information

- **Name:** ${manifest.name}
- **Version:** ${manifest.version}
- **Author:** ${manifest.author}
- **Description:** ${manifest.description}

For more information, see the README.md file included in this package.
`;
}

/**
 * Main CLI function
 */
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  
  const options = {
    clean: args.includes('--clean'),
    install: args.includes('--install'),
    verbose: args.includes('--verbose'),
    archive: args.includes('--archive'),
    watch: args.includes('--watch'),
    includeSource: args.includes('--include-source')
  };

  switch (command) {
    case 'build':
      if (args[1] && args[1] !== '--clean' && !args[1].startsWith('--')) {
        // Build specific plugin
        const pluginPath = join(PLUGINS_DIR, args[1]);
        if (!existsSync(pluginPath)) {
          console.error(`❌ Plugin not found: ${args[1]}`);
          process.exit(1);
        }
        await buildPlugin(pluginPath, options);
      } else {
        // Build all plugins
        await buildAllPlugins(options);
      }
      break;

    case 'package':
      if (args[1] && !args[1].startsWith('--')) {
        // Package specific plugin
        const pluginPath = join(PLUGINS_DIR, args[1]);
        if (!existsSync(pluginPath)) {
          console.error(`❌ Plugin not found: ${args[1]}`);
          process.exit(1);
        }
        
        // Parse format option
        const formatArg = args.find(arg => arg.startsWith('--format='));
        if (formatArg) {
          options.format = formatArg.split('=')[1];
        }
        
        await packagePlugin(pluginPath, options);
      } else {
        console.error('❌ Please specify a plugin name to package');
        process.exit(1);
      }
      break;

    case 'watch':
      console.log('👀 Starting watch mode...');
      // Simple watch implementation
      const chokidar = await import('chokidar');
      const watcher = chokidar.watch(join(PLUGINS_DIR, '*/src/**/*.ts'), {
        ignored: /node_modules/,
        persistent: true
      });

      watcher.on('change', async (path) => {
        const pluginName = path.split(/[/\\]/)[path.split(/[/\\]/).indexOf('plugins') + 1];
        const pluginPath = join(PLUGINS_DIR, pluginName);
        console.log(`🔄 File changed in ${pluginName}, rebuilding...`);
        await buildPlugin(pluginPath, { ...options, clean: false });
      });

      console.log('👀 Watching for changes... Press Ctrl+C to stop');
      break;

    default:
      console.log(`
🔨 Baize Plugin Build Tool

Usage:
  node scripts/build-plugin.js <command> [plugin-name] [options]

Commands:
  build [plugin-name]   Build plugin(s) (all if no name specified)
  package <plugin-name> Create distribution package for plugin
  watch                 Start watch mode for automatic rebuilding

Build Options:
  --clean              Clean dist directory before building
  --install            Install dependencies before building
  --verbose            Show detailed output

Package Options:
  --format=<format>    Package format: bpkg (default), folder, zip, tar.gz
  --include-source     Include source files in package
  --archive            Force create archive (legacy option)

Watch Options:
  --verbose            Show detailed file change information

Examples:
  # Building
  node scripts/build-plugin.js build                           # Build all plugins
  node scripts/build-plugin.js build hello-world              # Build specific plugin
  node scripts/build-plugin.js build --clean --verbose        # Clean build with verbose output
  
  # Packaging
  node scripts/build-plugin.js package hello-world            # Create .bpkg package
  node scripts/build-plugin.js package hello-world --format=zip # Create .zip package
  node scripts/build-plugin.js package hello-world --include-source # Include source code
  
  # Development
  node scripts/build-plugin.js watch                          # Watch for changes
  node scripts/build-plugin.js watch --verbose                # Watch with detailed output

Package Formats:
  bpkg     Baize Plugin Package (recommended, .bpkg file)
  folder   Uncompressed folder structure
  zip      Standard ZIP archive
  tar.gz   Compressed tar archive
      `);
      break;
  }
}

// Run main function
main().catch(error => {
  console.error('❌ Build script failed:', error);
  process.exit(1);
});

export { buildPlugin, buildAllPlugins, packagePlugin };