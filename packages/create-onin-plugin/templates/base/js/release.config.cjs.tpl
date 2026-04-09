const pluginName = require('./package.json').name

module.exports = {
  branches: 'master',
  plugins: [
    [
      '@semantic-release/commit-analyzer',
      {
        releaseRules: [
          { type: 'refactor', release: 'patch' }
        ]
      }
    ],
    '@semantic-release/release-notes-generator',
    '@semantic-release/changelog',
    [
      '@semantic-release/npm',
      { npmPublish: false }
    ],
    [
      '@semantic-release/git',
      {
        assets: ['CHANGELOG.md', 'package.json', 'manifest.json'],
        message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}'
      }
    ],
    [
      '@semantic-release/exec',
      {
        prepareCmd: "node -e \"const fs=require('fs'); const m=JSON.parse(fs.readFileSync('manifest.json', 'utf8')); m.version='${nextRelease.version}'; fs.writeFileSync('manifest.json', JSON.stringify(m, null, 2) + '\\n');\" && zip -qq -r " + pluginName + "-${nextRelease.version}.zip package.json manifest.json icon.png dist/"
      }
    ],
    [
      '@semantic-release/github',
      {
        assets: [`${pluginName}-*.zip`]
      }
    ]
  ]
}
