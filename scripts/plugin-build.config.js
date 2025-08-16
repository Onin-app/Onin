/**
 * Plugin Build Configuration
 * 
 * This file contains configuration options for plugin building and packaging.
 */

export const buildConfig = {
  // Build settings
  build: {
    // TypeScript compiler options override
    tsConfig: {
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "bundler",
      declaration: true,
      declarationMap: true,
      sourceMap: true,
      strict: true
    },
    
    // Files to include in build output
    includeFiles: [
      'plugin.json',
      'README.md',
      'LICENSE',
      'CHANGELOG.md'
    ],
    
    // Files to exclude from build
    excludePatterns: [
      '**/*.test.ts',
      '**/*.spec.ts',
      '**/node_modules/**',
      '**/.git/**',
      '**/dist/**'
    ],
    
    // Build optimization
    minify: false,
    sourceMaps: true,
    
    // Validation rules
    validation: {
      // Require specific fields in plugin.json
      requiredManifestFields: [
        'name', 'version', 'description', 'author', 
        'main', 'permissions', 'engines'
      ],
      
      // Validate plugin name format
      namePattern: /^[a-z0-9-]+$/,
      
      // Validate version format (semver)
      versionPattern: /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/,
      
      // Maximum file size for main bundle (in bytes)
      maxBundleSize: 5 * 1024 * 1024, // 5MB
      
      // Allowed permissions
      allowedPermissions: [
        'notifications',
        'storage',
        'filesystem',
        'network',
        'clipboard',
        'system-info'
      ]
    }
  },
  
  // Package settings
  package: {
    // Output directory for packages
    outputDir: 'packages/dist',
    
    // Package format
    format: 'bpkg', // Baize Plugin Package
    
    // Compression level (0-9)
    compression: 6,
    
    // Include development files in package
    includeDev: false,
    
    // Generate checksums
    generateChecksums: true,
    
    // Package metadata
    metadata: {
      packager: 'Baize Plugin Build System',
      includeSystemInfo: true,
      includeBuildInfo: true
    }
  },
  
  // Development server settings
  dev: {
    // Watch patterns
    watchPatterns: [
      'plugins/*/src/**/*.ts',
      'plugins/*/plugin.json',
      'plugins/*/package.json'
    ],
    
    // Ignore patterns
    ignorePatterns: [
      '**/node_modules/**',
      '**/dist/**',
      '**/.git/**',
      '**/*.test.ts'
    ],
    
    // Debounce time for file changes (ms)
    debounceMs: 300,
    
    // Hot reload settings
    hotReload: {
      enabled: true,
      notificationDir: '.kiro/dev-notifications',
      maxNotifications: 10
    },
    
    // Build on startup
    buildOnStart: true,
    
    // Clear console on rebuild
    clearConsole: false
  },
  
  // Distribution settings
  distribution: {
    // Create installation instructions
    generateInstallInstructions: true,
    
    // Include example usage
    includeExamples: true,
    
    // Generate API documentation
    generateDocs: false,
    
    // Sign packages (future feature)
    signPackages: false
  }
};

export default buildConfig;