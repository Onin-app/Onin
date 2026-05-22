import { strict as assert } from 'node:assert';
import test from 'node:test';

import {
  isNodeError,
  isValidPackageName,
  isValidPluginId,
  slugify,
  toTitleCase,
} from './validators.js';

test('toTitleCase converts strings to title case', () => {
  assert.equal(toTitleCase('hello-world'), 'Hello World');
  assert.equal(toTitleCase('foo_bar_baz'), 'Foo Bar Baz');
  assert.equal(toTitleCase('some.plugin.name'), 'Some Plugin Name');
  assert.equal(toTitleCase('already Title Case'), 'Already Title Case');
  assert.equal(toTitleCase('  multiple   spaces  '), 'Multiple Spaces');
  assert.equal(toTitleCase(''), '');
});

test('slugify cleans and formats strings to package/plugin slugs', () => {
  assert.equal(slugify('My Plugin Name'), 'my-plugin-name');
  assert.equal(slugify('  Trim Space  '), 'trim-space');
  assert.equal(slugify('special@#$%characters'), 'special-characters');
  assert.equal(slugify('multiple---hyphens'), 'multiple-hyphens');
  assert.equal(slugify('-leading-and-trailing-'), 'leading-and-trailing');
  assert.equal(
    slugify('dots.and-hyphens.are.kept'),
    'dots.and-hyphens.are.kept',
  );
  assert.equal(slugify(''), '');
});

test('isValidPluginId validates plugin IDs correctly', () => {
  // Valid plugin IDs
  assert.ok(isValidPluginId('com.example.my-plugin'));
  assert.ok(isValidPluginId('plugin-id'));
  assert.ok(isValidPluginId('a.b.c'));
  assert.ok(isValidPluginId('a-b-c'));

  // Invalid plugin IDs
  assert.ok(!isValidPluginId('com.example..plugin')); // Double dots
  assert.ok(!isValidPluginId('.leading.dot'));
  assert.ok(!isValidPluginId('trailing.dot.'));
  assert.ok(!isValidPluginId('-leading.hyphen'));
  assert.ok(!isValidPluginId('trailing.hyphen-'));
  assert.ok(!isValidPluginId('Upper.Case'));
  assert.ok(!isValidPluginId('special#char'));
  assert.ok(!isValidPluginId(''));
});

test('isValidPackageName validates package names correctly', () => {
  // Valid package names
  assert.ok(isValidPackageName('com.example.my-plugin'));
  assert.ok(isValidPackageName('my-plugin'));
  assert.ok(isValidPackageName('plugin'));

  // Invalid package names
  assert.ok(!isValidPackageName('.leading.dot'));
  assert.ok(!isValidPackageName('trailing.dot.'));
  assert.ok(!isValidPackageName('-leading.hyphen'));
  assert.ok(!isValidPackageName('trailing.hyphen-'));
  assert.ok(!isValidPackageName('Upper.Case'));
  assert.ok(!isValidPackageName(''));
});

test('isNodeError identifies NodeJS ErrnoException', () => {
  // Node error object
  const nodeError = new Error('File not found') as NodeJS.ErrnoException;
  nodeError.code = 'ENOENT';
  assert.ok(isNodeError(nodeError));

  // Normal error
  const normalError = new Error('Some error');
  assert.ok(!isNodeError(normalError));

  // Plain objects
  assert.ok(isNodeError({ code: 'SOME_CODE' }));
  assert.ok(!isNodeError({ code: 123 })); // code is not string
  assert.ok(!isNodeError({ message: 'some message' })); // missing code
  assert.ok(!isNodeError(null));
  assert.ok(!isNodeError(undefined));
  assert.ok(!isNodeError('string error'));
  assert.ok(!isNodeError(456));
});
