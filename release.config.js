module.exports = {
  branches: ['master'],
  plugins: [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    [
      '@semantic-release/changelog',
      {
        changelogFile: 'CHANGELOG.md',
      },
    ],
    // 重要：这个插件要放在 @semantic-release/git 前面，用来更新 tauri 的版本号
    // 但是它不能修改 package.json，因为 @semantic-release/npm 会做这件事
    [
      "@semantic-release/exec",
      {
        "prepareCmd": "node -e 'let conf = require(\"./src-tauri/tauri.conf.json\"); conf.package.version = \"${nextRelease.version}\"; require(\"fs\").writeFileSync(\"./src-tauri/tauri.conf.json\", JSON.stringify(conf, null, 2));'"
      }
    ],
    // npm 插件会自动更新 package.json 的版本号并生成 package-lock.json
    '@semantic-release/npm',
    [
      '@semantic-release/git',
      {
        assets: ['CHANGELOG.md', 'package.json', 'pnpm-lock.yaml', 'src-tauri/tauri.conf.json'],
        message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}',
      },
    ],
    '@semantic-release/github',
  ],
};
