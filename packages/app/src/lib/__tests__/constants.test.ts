import { describe, it, expect } from 'vitest';
import { UPDATE_CONFIG } from '../constants';

describe('UPDATE_CONFIG', () => {
  it('has correct GitHub owner and repo', () => {
    expect(UPDATE_CONFIG.GITHUB_OWNER).toBe('b-yp');
    expect(UPDATE_CONFIG.GITHUB_REPO).toBe('baize');
  });

  it('generates correct LATEST_RELEASE_URL', () => {
    expect(UPDATE_CONFIG.LATEST_RELEASE_URL).toBe(
      'https://api.github.com/repos/b-yp/baize/releases/latest',
    );
  });
});
