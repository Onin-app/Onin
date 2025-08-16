# Plugin Build and Packaging System

This document describes the comprehensive build and packaging system for Baize plugins, including development tools, build processes, and distribution formats.

## Overview

The Baize plugin build system provides:

- **TypeScript compilation** with proper type checking and declaration generation
- **Multiple packaging formats** for different distribution needs
- **Hot reload development server** for rapid plugin development
- **Automated validation** of plugin manifests and code
- **Cross-platform compatibility** for Windows, macOS, and Linux

## Quick Start

### Building a Plugin

```bash
# Build all plugins
pnpm run build:plugins

# Build a specific plugin
pnpm run build:plugin hello-world

# Clean build with verbose output
node scripts/build-plugin.js build hello-world --clean --verbose
```

### Packaging a Plugin

```bash
# Create a .bpkg package (recommended)
pnpm run package:plugin hello-world

# Create different package formats
node scripts/build-plugin.js package hello-world --format=zip
node scripts/build-plugin.js package hello-world --format=folder
node scripts/build-plugin.js package hello-world --format=tar.gz

# Include source code in package
node scripts/build-plugin.js package hello-world --include-source
```

### Development Server

```bash
# Start development server with hot reload
pnpm run dev:plugins

# Start with verbose logging
node scripts/dev-server.js --verbose
```

## Build System Architecture

### Core Components

1. **Build Script** (`scripts/build-plugin.js`)
   - TypeScript compilation
   - Manifest validation
   - File copying and organization
   - Error handling and reporting

2. **Development Server** (`scripts/dev-server.js`)
   - File watching and change detection
   - Automatic rebuilding
   - Hot reload notifications
   - Build status reporting

3. **Package Utilities** (`scripts/package-utils.js`)
   - Multiple package format support
   - Checksum generation
   - Metadata creation
   - Archive creation

4. **Build Configuration** (`scripts/plugin-build.config.js`)
   - Centralized build settings
   - Validation rules
   - Package options
   - Development server configuration

### Plugin Structure

A typical plugin follows this structure:

```
my-plugin/
├── src/
│   ├── index.ts          # Main plugin entry point
│   ├── types.ts          # Plugin-specific types
│   └── utils.ts          # Utility functions
├── dist/                 # Built output (generated)
│   ├── index.js
│   ├── index.d.ts
│   └── index.js.map
├── plugin.json           # Plugin manifest
├── package.json          # NPM package configuration
├── tsconfig.json         # TypeScript configuration
├── README.md             # Plugin documentation
└── LICENSE               # License file
```

## Package Formats

### .bpkg (Baize Plugin Package) - Recommended

The `.bpkg` format is the recommended distribution format for Baize plugins:

- **Compressed**: ZIP-based compression for smaller file sizes
- **Metadata**: Includes comprehensive package information
- **Integrity**: Built-in checksum validation
- **Installation**: Direct installation through Baize UI

```bash
node scripts/build-plugin.js package my-plugin --format=bpkg
```

### Folder Format

Uncompressed folder structure for development and debugging:

```bash
node scripts/build-plugin.js package my-plugin --format=folder
```

### ZIP Format

Standard ZIP archive for general distribution:

```bash
node scripts/build-plugin.js package my-plugin --format=zip
```

### TAR.GZ Format

Compressed tar archive for Unix-like systems:

```bash
node scripts/build-plugin.js package my-plugin --format=tar.gz
```

## Development Workflow

### 1. Create Plugin Structure

Use the plugin scaffold tool to create a new plugin:

```bash
npx @baize/create-plugin my-awesome-plugin
```

### 2. Development

Start the development server for hot reload:

```bash
pnpm run dev:plugins
```

The development server will:
- Watch for file changes in `plugins/*/src/**/*.ts`
- Automatically rebuild changed plugins
- Generate hot reload notifications
- Display build status and errors

### 3. Testing

Run plugin tests:

```bash
cd plugins/my-plugin
pnpm test
```

### 4. Building

Build the plugin for distribution:

```bash
pnpm run build:plugin my-plugin
```

### 5. Packaging

Create distribution packages:

```bash
# Standard package
pnpm run package:plugin my-plugin

# Package with source code
node scripts/build-plugin.js package my-plugin --include-source
```

## Configuration

### Build Configuration

The build system can be configured through `scripts/plugin-build.config.js`:

```javascript
export const buildConfig = {
  build: {
    // TypeScript compiler options
    tsConfig: { /* ... */ },
    
    // Validation rules
    validation: {
      requiredManifestFields: ['name', 'version', /* ... */],
      namePattern: /^[a-z0-9-]+$/,
      maxBundleSize: 5 * 1024 * 1024 // 5MB
    }
  },
  
  package: {
    outputDir: 'packages/dist',
    format: 'bpkg',
    generateChecksums: true
  },
  
  dev: {
    debounceMs: 300,
    hotReload: {
      enabled: true,
      notificationDir: '.kiro/dev-notifications'
    }
  }
};
```

### Plugin Manifest

Each plugin must include a `plugin.json` manifest:

```json
{
  "name": "my-awesome-plugin",
  "version": "1.0.0",
  "description": "An awesome plugin for Baize",
  "author": "Your Name",
  "main": "dist/index.js",
  "permissions": [
    "notifications",
    "storage"
  ],
  "engines": {
    "baize": ">=0.1.0"
  },
  "keywords": ["awesome", "utility"],
  "repository": "https://github.com/user/my-awesome-plugin"
}
```

### TypeScript Configuration

Plugins use a standard TypeScript configuration (`tsconfig.json`):

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "outDir": "./dist",
    "rootDir": "./src",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "strict": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

## Hot Reload System

The development server provides hot reload capabilities:

### How It Works

1. **File Watching**: Monitors TypeScript files, manifests, and package.json
2. **Debounced Rebuilds**: Prevents excessive rebuilds during rapid file changes
3. **Notification System**: Creates notification files that the main app can watch
4. **Error Isolation**: Build errors don't crash the development server

### Notification Format

Hot reload notifications are stored in `.kiro/dev-notifications/`:

```json
{
  "type": "plugin-build",
  "pluginName": "hello-world",
  "status": "success",
  "timestamp": "2024-01-15T10:30:00.000Z",
  "duration": 150
}
```

### Integration with Main App

The main Baize application can watch the notification directory to:
- Reload plugins automatically during development
- Display build status in the UI
- Show error messages to developers

## Validation and Quality Assurance

### Manifest Validation

The build system validates plugin manifests:

- **Required Fields**: Ensures all necessary fields are present
- **Format Validation**: Checks version format (semver), name format, etc.
- **File Existence**: Verifies that referenced files exist
- **Permission Validation**: Checks against allowed permissions list

### Code Validation

- **TypeScript Compilation**: Ensures type safety and catches errors
- **Bundle Size Limits**: Prevents excessively large plugins
- **Dependency Checking**: Validates plugin dependencies

### Package Integrity

- **Checksum Generation**: Creates MD5, SHA1, and SHA256 checksums
- **Integrity Verification**: Validates package contents during installation
- **Metadata Validation**: Ensures package metadata is complete and accurate

## Troubleshooting

### Common Build Issues

1. **TypeScript Errors**
   ```bash
   # Check TypeScript configuration
   npx tsc --noEmit
   
   # Verbose build output
   node scripts/build-plugin.js build my-plugin --verbose
   ```

2. **Missing Dependencies**
   ```bash
   # Install plugin dependencies
   cd plugins/my-plugin
   pnpm install
   ```

3. **Manifest Validation Errors**
   - Check `plugin.json` format
   - Ensure all required fields are present
   - Verify file paths are correct

### Development Server Issues

1. **Hot Reload Not Working**
   - Check file watch patterns in configuration
   - Ensure notification directory is writable
   - Verify TypeScript compilation is successful

2. **Build Performance**
   - Adjust debounce timing in configuration
   - Use `--verbose` flag to identify bottlenecks
   - Consider excluding large files from watch patterns

### Package Creation Issues

1. **Archive Creation Fails**
   - Ensure required system tools are installed (zip, tar)
   - Check file permissions
   - Try different package formats

2. **Checksum Generation Fails**
   - Verify package file exists and is readable
   - Check available disk space
   - Ensure write permissions for output directory

## Best Practices

### Development

1. **Use TypeScript**: Take advantage of type safety and IDE support
2. **Follow Naming Conventions**: Use kebab-case for plugin names
3. **Semantic Versioning**: Follow semver for version numbers
4. **Comprehensive Testing**: Write tests for plugin functionality
5. **Documentation**: Include clear README and API documentation

### Packaging

1. **Minimal Packages**: Only include necessary files in distribution
2. **Version Control**: Tag releases and maintain changelog
3. **Security**: Review permissions and validate user inputs
4. **Performance**: Optimize bundle size and startup time

### Distribution

1. **Multiple Formats**: Provide different package formats for different use cases
2. **Installation Instructions**: Include clear installation and usage instructions
3. **Compatibility**: Test on different platforms and Baize versions
4. **Support**: Provide clear support channels and issue reporting

## Advanced Features

### Custom Build Scripts

Plugins can include custom build steps in their `package.json`:

```json
{
  "scripts": {
    "prebuild": "node custom-prebuild.js",
    "build": "tsc",
    "postbuild": "node custom-postbuild.js"
  }
}
```

### Plugin Dependencies

Plugins can depend on other plugins:

```json
{
  "dependencies": {
    "@baize/plugin-sdk": "workspace:*",
    "other-plugin": "^1.0.0"
  }
}
```

### Environment-Specific Builds

Configure different builds for development and production:

```json
{
  "scripts": {
    "build": "tsc",
    "build:dev": "tsc --sourceMap",
    "build:prod": "tsc --removeComments --declaration false"
  }
}
```

## Future Enhancements

The build system is designed to be extensible. Planned enhancements include:

- **Plugin Signing**: Digital signatures for security
- **Dependency Resolution**: Automatic dependency management
- **Performance Profiling**: Build-time performance analysis
- **Custom Bundlers**: Support for webpack, rollup, etc.
- **Plugin Store Integration**: Direct publishing to plugin store
- **Automated Testing**: Integration with CI/CD systems

## API Reference

For detailed API documentation, see:
- [Plugin SDK API Reference](./api-reference.md)
- [Plugin Development Guide](./plugin-development.md)
- [Build System Configuration](../scripts/plugin-build.config.js)