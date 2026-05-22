import { describe, it, expect } from 'vitest';

describe('adapters barrel exports', () => {
  it('exports BaseAdapter', async () => {
    const { BaseAdapter } = await import('../index');
    expect(BaseAdapter).toBeDefined();
    expect(typeof BaseAdapter).toBe('function');
  });

  it('exports WindowModeAdapter', async () => {
    const { WindowModeAdapter } = await import('../index');
    expect(WindowModeAdapter).toBeDefined();
    expect(typeof WindowModeAdapter).toBe('function');
  });

  it('exports LifecycleMessageAdapter', async () => {
    const { LifecycleMessageAdapter } = await import('../index');
    expect(LifecycleMessageAdapter).toBeDefined();
    expect(typeof LifecycleMessageAdapter).toBe('function');
  });
});
