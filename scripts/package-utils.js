/**
 * Plugin Package Utilities
 * 
 * Utilities for creating different plugin package formats and handling distribution.
 */

import { join, dirname, basename } from 'path';
import { existsSync, readFileSync, writeFileSync, mkdirSync, copyFileSync, statSync } from 'fs';
import { execSync } from 'child_process';
import { createHash } from 'crypto';

/**
 * Package formats supported by the build system
 */
export const PACKAGE_FORMATS = {
  BPKG: 'bpkg',     // Baize Plugin Package (zip-based)
  FOLDER: 'folder',  // Simple folder structure
  TAR: 'tar.gz',    // Compressed tar archive
  ZIP: 'zip'        // Standard zip archive
};

/**
 * Create a plugin package in the specified format
 * @param {string} pluginPath - Path to plugin directory
 * @param {object} options - Package options
 */
export async function createPluginPackage(pluginPath, options = {}) {
  const {
    format = PACKAGE_FORMATS.BPKG,
    outputDir = 'packages/dist',
    includeSource = false,
    generateChecksums = true,
    compression = 6
  } = options;

  const pluginName = basename(pluginPath);
  console.log(`📦 Creating ${format} package for ${pluginName}...`);

  try {
    // Read plugin manifest
    const manifestPath = join(pluginPath, 'plugin.json');
    if (!existsSync(manifestPath)) {
      throw new Error('Plugin manifest (plugin.json) not found');
    }

    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    
    // Ensure output directory exists
    const fullOutputDir = join(process.cwd(), outputDir);
    mkdirSync(fullOutputDir, { recursive: true });

    // Create package based on format
    let packagePath;
    switch (format) {
      case PACKAGE_FORMATS.BPKG:
        packagePath = await createBpkgPackage(pluginPath, manifest, fullOutputDir, options);
        break;
      case PACKAGE_FORMATS.FOLDER:
        packagePath = await createFolderPackage(pluginPath, manifest, fullOutputDir, options);
        break;
      case PACKAGE_FORMATS.TAR:
        packagePath = await createTarPackage(pluginPath, manifest, fullOutputDir, options);
        break;
      case PACKAGE_FORMATS.ZIP:
        packagePath = await createZipPackage(pluginPath, manifest, fullOutputDir, options);
        break;
      default:
        throw new Error(`Unsupported package format: ${format}`);
    }

    // Generate checksums if requested
    if (generateChecksums) {
      await generatePackageChecksums(packagePath);
    }

    console.log(`✅ Package created: ${basename(packagePath)}`);
    return packagePath;

  } catch (error) {
    console.error(`❌ Failed to create package for ${pluginName}:`, error.message);
    throw error;
  }
}

/**
 * Create a .bpkg (Baize Plugin Package) format
 * @param {string} pluginPath - Plugin source path
 * @param {object} manifest - Plugin manifest
 * @param {string} outputDir - Output directory
 * @param {object} options - Package options
 */
async function createBpkgPackage(pluginPath, manifest, outputDir, options) {
  const packageName = `${manifest.name}-${manifest.version}.bpkg`;
  const packagePath = join(outputDir, packageName);
  
  // Create temporary staging directory
  const stagingDir = join(outputDir, `staging-${manifest.name}-${Date.now()}`);
  mkdirSync(stagingDir, { recursive: true });

  try {
    // Copy plugin files to staging
    await copyPluginFiles(pluginPath, stagingDir, options);
    
    // Add package metadata
    const packageInfo = createPackageMetadata(manifest, options);
    writeFileSync(
      join(stagingDir, '.bpkg-info.json'),
      JSON.stringify(packageInfo, null, 2)
    );

    // Create the archive
    await createArchive(stagingDir, packagePath, 'zip');
    
    return packagePath;
  } finally {
    // Clean up staging directory
    try {
      execSync(`rimraf "${stagingDir}"`, { stdio: 'ignore' });
    } catch (error) {
      // Ignore cleanup errors
    }
  }
}

/**
 * Create a folder-based package
 * @param {string} pluginPath - Plugin source path
 * @param {object} manifest - Plugin manifest
 * @param {string} outputDir - Output directory
 * @param {object} options - Package options
 */
async function createFolderPackage(pluginPath, manifest, outputDir, options) {
  const packageName = `${manifest.name}-${manifest.version}`;
  const packagePath = join(outputDir, packageName);
  
  // Create package directory
  mkdirSync(packagePath, { recursive: true });
  
  // Copy plugin files
  await copyPluginFiles(pluginPath, packagePath, options);
  
  // Add package metadata
  const packageInfo = createPackageMetadata(manifest, options);
  writeFileSync(
    join(packagePath, 'package-info.json'),
    JSON.stringify(packageInfo, null, 2)
  );

  return packagePath;
}

/**
 * Create a tar.gz package
 * @param {string} pluginPath - Plugin source path
 * @param {object} manifest - Plugin manifest
 * @param {string} outputDir - Output directory
 * @param {object} options - Package options
 */
async function createTarPackage(pluginPath, manifest, outputDir, options) {
  const packageName = `${manifest.name}-${manifest.version}.tar.gz`;
  const packagePath = join(outputDir, packageName);
  
  // Create temporary staging directory
  const stagingDir = join(outputDir, `staging-${manifest.name}-${Date.now()}`);
  mkdirSync(stagingDir, { recursive: true });

  try {
    // Copy plugin files to staging
    await copyPluginFiles(pluginPath, stagingDir, options);
    
    // Add package metadata
    const packageInfo = createPackageMetadata(manifest, options);
    writeFileSync(
      join(stagingDir, 'package-info.json'),
      JSON.stringify(packageInfo, null, 2)
    );

    // Create the archive
    await createArchive(stagingDir, packagePath, 'tar');
    
    return packagePath;
  } finally {
    // Clean up staging directory
    try {
      execSync(`rimraf "${stagingDir}"`, { stdio: 'ignore' });
    } catch (error) {
      // Ignore cleanup errors
    }
  }
}

/**
 * Create a zip package
 * @param {string} pluginPath - Plugin source path
 * @param {object} manifest - Plugin manifest
 * @param {string} outputDir - Output directory
 * @param {object} options - Package options
 */
async function createZipPackage(pluginPath, manifest, outputDir, options) {
  const packageName = `${manifest.name}-${manifest.version}.zip`;
  const packagePath = join(outputDir, packageName);
  
  // Create temporary staging directory
  const stagingDir = join(outputDir, `staging-${manifest.name}-${Date.now()}`);
  mkdirSync(stagingDir, { recursive: true });

  try {
    // Copy plugin files to staging
    await copyPluginFiles(pluginPath, stagingDir, options);
    
    // Add package metadata
    const packageInfo = createPackageMetadata(manifest, options);
    writeFileSync(
      join(stagingDir, 'package-info.json'),
      JSON.stringify(packageInfo, null, 2)
    );

    // Create the archive
    await createArchive(stagingDir, packagePath, 'zip');
    
    return packagePath;
  } finally {
    // Clean up staging directory
    try {
      execSync(`rimraf "${stagingDir}"`, { stdio: 'ignore' });
    } catch (error) {
      // Ignore cleanup errors
    }
  }
}

/**
 * Copy plugin files to destination
 * @param {string} sourcePath - Source plugin path
 * @param {string} destPath - Destination path
 * @param {object} options - Copy options
 */
async function copyPluginFiles(sourcePath, destPath, options = {}) {
  const { includeSource = false } = options;
  
  // Use Node.js built-in fs methods for copying
  const { cpSync } = await import('fs');
  
  // Always copy dist directory (built plugin)
  const distPath = join(sourcePath, 'dist');
  if (existsSync(distPath)) {
    cpSync(distPath, destPath, { recursive: true });
  }

  // Copy additional files
  const additionalFiles = [
    'plugin.json',
    'README.md',
    'LICENSE',
    'CHANGELOG.md',
    'package.json'
  ];

  for (const file of additionalFiles) {
    const srcFile = join(sourcePath, file);
    if (existsSync(srcFile)) {
      copyFileSync(srcFile, join(destPath, file));
    }
  }

  // Copy source files if requested
  if (includeSource) {
    const srcPath = join(sourcePath, 'src');
    if (existsSync(srcPath)) {
      cpSync(srcPath, join(destPath, 'src'), { recursive: true });
    }
  }
}

/**
 * Create package metadata
 * @param {object} manifest - Plugin manifest
 * @param {object} options - Package options
 */
function createPackageMetadata(manifest, options = {}) {
  return {
    ...manifest,
    packageInfo: {
      format: options.format || 'bpkg',
      packagedAt: new Date().toISOString(),
      packagedBy: 'Baize Plugin Build System',
      version: '1.0.0',
      includesSource: options.includeSource || false
    },
    buildInfo: {
      nodeVersion: process.version,
      platform: process.platform,
      arch: process.arch,
      buildTool: 'baize-plugin-builder'
    }
  };
}

/**
 * Create archive file
 * @param {string} sourcePath - Source directory
 * @param {string} archivePath - Output archive path
 * @param {string} format - Archive format (zip/tar)
 */
async function createArchive(sourcePath, archivePath, format) {
  try {
    if (format === 'zip') {
      // Use PowerShell on Windows, zip command on Unix
      if (process.platform === 'win32') {
        // PowerShell Compress-Archive only supports .zip extension
        // Create with .zip extension first, then rename if needed
        const tempZipPath = archivePath.endsWith('.zip') ? archivePath : archivePath + '.temp.zip';
        
        execSync(
          `powershell Compress-Archive -Path "${sourcePath}\\*" -DestinationPath "${tempZipPath}" -Force`,
          { shell: true }
        );
        
        // Rename if the target has a different extension
        if (tempZipPath !== archivePath) {
          const { renameSync } = await import('fs');
          renameSync(tempZipPath, archivePath);
        }
      } else {
        execSync(`cd "${sourcePath}" && zip -r "${archivePath}" .`, { shell: true });
      }
    } else if (format === 'tar') {
      execSync(`cd "${sourcePath}" && tar -czf "${archivePath}" .`, { shell: true });
    }
  } catch (error) {
    // Fallback to Node.js implementation for zip
    if (format === 'zip') {
      await createZipWithNodejs(sourcePath, archivePath);
    } else {
      throw error;
    }
  }
}

/**
 * Create zip archive using Node.js (fallback)
 * @param {string} sourcePath - Source directory
 * @param {string} archivePath - Output archive path
 */
async function createZipWithNodejs(sourcePath, archivePath) {
  try {
    const archiver = await import('archiver');
    const { createWriteStream } = await import('fs');
    
    return new Promise((resolve, reject) => {
      const output = createWriteStream(archivePath);
      const archive = archiver.default('zip', { zlib: { level: 6 } });
      
      output.on('close', resolve);
      archive.on('error', reject);
      
      archive.pipe(output);
      archive.directory(sourcePath, false);
      archive.finalize();
    });
  } catch (error) {
    throw new Error(`Failed to create zip archive: ${error.message}`);
  }
}

/**
 * Generate checksums for package
 * @param {string} packagePath - Path to package file or directory
 */
async function generatePackageChecksums(packagePath) {
  // Skip checksum generation for directories
  if (statSync(packagePath).isDirectory()) {
    console.log(`📋 Skipping checksums for directory package`);
    return {};
  }
  
  const algorithms = ['md5', 'sha1', 'sha256'];
  const checksums = {};
  
  const fileContent = readFileSync(packagePath);
  
  for (const algorithm of algorithms) {
    const hash = createHash(algorithm);
    hash.update(fileContent);
    checksums[algorithm] = hash.digest('hex');
  }
  
  // Write checksums file
  const checksumPath = `${packagePath}.checksums`;
  const checksumContent = Object.entries(checksums)
    .map(([algo, hash]) => `${hash}  ${basename(packagePath)} (${algo})`)
    .join('\n');
  
  writeFileSync(checksumPath, checksumContent);
  
  console.log(`📋 Generated checksums: ${basename(checksumPath)}`);
  return checksums;
}

/**
 * Verify package integrity
 * @param {string} packagePath - Path to package file
 * @param {string} expectedChecksum - Expected checksum
 * @param {string} algorithm - Hash algorithm (default: sha256)
 */
export function verifyPackageIntegrity(packagePath, expectedChecksum, algorithm = 'sha256') {
  if (!existsSync(packagePath)) {
    throw new Error(`Package file not found: ${packagePath}`);
  }
  
  const fileContent = readFileSync(packagePath);
  const hash = createHash(algorithm);
  hash.update(fileContent);
  const actualChecksum = hash.digest('hex');
  
  if (actualChecksum !== expectedChecksum) {
    throw new Error(`Package integrity check failed. Expected: ${expectedChecksum}, Got: ${actualChecksum}`);
  }
  
  return true;
}

export { createPluginPackage as default };