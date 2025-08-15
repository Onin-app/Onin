/**
 * Unit tests for the event system
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { EventEmitter, createEventEmitter, globalEventEmitter, SystemEvents } from './events';
import type { EventHandler, EventSubscriptionOptions } from './events';

describe('EventEmitter', () => {
  let eventEmitter: EventEmitter;

  beforeEach(() => {
    eventEmitter = new EventEmitter();
  });

  describe('Basic EventAPI functionality', () => {
    it('should register and trigger event listeners', () => {
      const handler = vi.fn();
      const testData = { message: 'test' };

      eventEmitter.on('test-event', handler);
      eventEmitter.emit('test-event', testData);

      expect(handler).toHaveBeenCalledOnce();
      expect(handler).toHaveBeenCalledWith(expect.objectContaining(testData));
    });

    it('should support multiple listeners for the same event', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();
      const testData = { message: 'test' };

      eventEmitter.on('test-event', handler1);
      eventEmitter.on('test-event', handler2);
      eventEmitter.emit('test-event', testData);

      expect(handler1).toHaveBeenCalledOnce();
      expect(handler2).toHaveBeenCalledOnce();
    });

    it('should remove specific event listeners', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      eventEmitter.on('test-event', handler1);
      eventEmitter.on('test-event', handler2);
      eventEmitter.off('test-event', handler1);
      eventEmitter.emit('test-event', { message: 'test' });

      expect(handler1).not.toHaveBeenCalled();
      expect(handler2).toHaveBeenCalledOnce();
    });

    it('should handle removing non-existent listeners gracefully', () => {
      const handler = vi.fn();

      // Should not throw error
      expect(() => {
        eventEmitter.off('non-existent-event', handler);
      }).not.toThrow();

      eventEmitter.on('test-event', handler);
      eventEmitter.off('test-event', vi.fn()); // Different handler
      eventEmitter.emit('test-event', { message: 'test' });

      expect(handler).toHaveBeenCalledOnce();
    });

    it('should not trigger listeners for different events', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      eventEmitter.on('event1', handler1);
      eventEmitter.on('event2', handler2);
      eventEmitter.emit('event1', { message: 'test' });

      expect(handler1).toHaveBeenCalledOnce();
      expect(handler2).not.toHaveBeenCalled();
    });
  });

  describe('Extended EventAPI functionality', () => {
    it('should support subscription with options', () => {
      const handler = vi.fn();
      const options: EventSubscriptionOptions = {
        once: true,
        priority: 10
      };

      const subscription = eventEmitter.subscribe('test-event', handler, options);
      
      expect(subscription).toHaveProperty('event', 'test-event');
      expect(subscription).toHaveProperty('id');
      expect(subscription).toHaveProperty('unsubscribe');
      expect(typeof subscription.unsubscribe).toBe('function');
    });

    it('should handle once option correctly', () => {
      const handler = vi.fn();

      eventEmitter.subscribe('test-event', handler, { once: true });
      eventEmitter.emit('test-event', { message: 'first' });
      eventEmitter.emit('test-event', { message: 'second' });

      expect(handler).toHaveBeenCalledOnce();
      expect(handler).toHaveBeenCalledWith(expect.objectContaining({ message: 'first' }));
    });

    it('should respect priority ordering', () => {
      const callOrder: number[] = [];
      const handler1 = vi.fn(() => callOrder.push(1));
      const handler2 = vi.fn(() => callOrder.push(2));
      const handler3 = vi.fn(() => callOrder.push(3));

      eventEmitter.subscribe('test-event', handler1, { priority: 1 });
      eventEmitter.subscribe('test-event', handler2, { priority: 10 });
      eventEmitter.subscribe('test-event', handler3, { priority: 5 });

      eventEmitter.emit('test-event');

      expect(callOrder).toEqual([2, 3, 1]); // Priority order: 10, 5, 1
    });

    it('should support filter option', () => {
      const handler = vi.fn();
      const filter = (data: any) => data.shouldHandle === true;

      eventEmitter.subscribe('test-event', handler, { filter });

      eventEmitter.emit('test-event', { shouldHandle: false });
      expect(handler).not.toHaveBeenCalled();

      eventEmitter.emit('test-event', { shouldHandle: true });
      expect(handler).toHaveBeenCalledOnce();
    });

    it('should support unsubscribe via subscription object', () => {
      const handler = vi.fn();
      const subscription = eventEmitter.subscribe('test-event', handler);

      eventEmitter.emit('test-event', { message: 'before' });
      expect(handler).toHaveBeenCalledOnce();

      subscription.unsubscribe();
      eventEmitter.emit('test-event', { message: 'after' });
      expect(handler).toHaveBeenCalledOnce(); // Still only called once
    });

    it('should emit events with metadata', () => {
      const handler = vi.fn();
      const testData = { message: 'test' };
      const metadata = { source: 'test-plugin', priority: 5 };

      eventEmitter.on('test-event', handler);
      eventEmitter.emitWithMetadata('test-event', testData, metadata);

      expect(handler).toHaveBeenCalledWith(expect.objectContaining({
        ...testData,
        __metadata: expect.objectContaining({
          ...metadata,
          timestamp: expect.any(Number)
        })
      }));
    });
  });

  describe('Event listener management', () => {
    it('should provide listener information', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      eventEmitter.subscribe('event1', handler1, { priority: 10, once: true });
      eventEmitter.subscribe('event1', handler2, { priority: 5 });
      eventEmitter.subscribe('event2', handler1);

      const listeners = eventEmitter.getListeners();

      expect(listeners).toHaveLength(2);
      
      const event1Listeners = listeners.find(l => l.event === 'event1');
      expect(event1Listeners).toBeDefined();
      expect(event1Listeners!.listenerCount).toBe(2);
      expect(event1Listeners!.listeners).toEqual([
        expect.objectContaining({ priority: 10, once: true }),
        expect.objectContaining({ priority: 5, once: false })
      ]);

      const event2Listeners = listeners.find(l => l.event === 'event2');
      expect(event2Listeners).toBeDefined();
      expect(event2Listeners!.listenerCount).toBe(1);
    });

    it('should remove all listeners for specific event', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      eventEmitter.on('event1', handler1);
      eventEmitter.on('event1', handler2);
      eventEmitter.on('event2', handler1);

      eventEmitter.removeAllListeners('event1');
      eventEmitter.emit('event1');
      eventEmitter.emit('event2');

      expect(handler1).toHaveBeenCalledOnce(); // Only from event2
      expect(handler2).not.toHaveBeenCalled();
    });

    it('should remove all listeners when no event specified', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      eventEmitter.on('event1', handler1);
      eventEmitter.on('event2', handler2);

      eventEmitter.removeAllListeners();
      eventEmitter.emit('event1');
      eventEmitter.emit('event2');

      expect(handler1).not.toHaveBeenCalled();
      expect(handler2).not.toHaveBeenCalled();
    });

    it('should provide listener count', () => {
      const handler = vi.fn();

      expect(eventEmitter.listenerCount('test-event')).toBe(0);

      eventEmitter.on('test-event', handler);
      expect(eventEmitter.listenerCount('test-event')).toBe(1);

      eventEmitter.on('test-event', vi.fn());
      expect(eventEmitter.listenerCount('test-event')).toBe(2);

      eventEmitter.off('test-event', handler);
      expect(eventEmitter.listenerCount('test-event')).toBe(1);
    });

    it('should check if event has listeners', () => {
      const handler = vi.fn();

      expect(eventEmitter.hasListeners('test-event')).toBe(false);

      eventEmitter.on('test-event', handler);
      expect(eventEmitter.hasListeners('test-event')).toBe(true);

      eventEmitter.off('test-event', handler);
      expect(eventEmitter.hasListeners('test-event')).toBe(false);
    });

    it('should provide list of event names', () => {
      const handler = vi.fn();

      expect(eventEmitter.eventNames()).toEqual([]);

      eventEmitter.on('event1', handler);
      eventEmitter.on('event2', handler);
      
      const eventNames = eventEmitter.eventNames();
      expect(eventNames).toContain('event1');
      expect(eventNames).toContain('event2');
      expect(eventNames).toHaveLength(2);
    });
  });

  describe('Error handling', () => {
    it('should handle errors in event handlers gracefully', () => {
      const errorHandler = vi.fn(() => {
        throw new Error('Handler error');
      });
      const normalHandler = vi.fn();
      const systemErrorHandler = vi.fn();

      // Listen for system error events
      eventEmitter.on('system:error', systemErrorHandler);
      
      eventEmitter.on('test-event', errorHandler);
      eventEmitter.on('test-event', normalHandler);

      // Should not throw
      expect(() => {
        eventEmitter.emit('test-event', { message: 'test' });
      }).not.toThrow();

      expect(errorHandler).toHaveBeenCalledOnce();
      expect(normalHandler).toHaveBeenCalledOnce();
      expect(systemErrorHandler).toHaveBeenCalledOnce();
      expect(systemErrorHandler).toHaveBeenCalledWith(expect.objectContaining({
        type: 'event_handler_error',
        event: 'test-event',
        error: 'Handler error'
      }));
    });

    it('should clean up once listeners even after errors', () => {
      const errorHandler = vi.fn(() => {
        throw new Error('Handler error');
      });

      eventEmitter.subscribe('test-event', errorHandler, { once: true });
      eventEmitter.emit('test-event');

      // The once listener should be removed even if it throws an error
      expect(eventEmitter.hasListeners('test-event')).toBe(false);
      expect(eventEmitter.eventNames()).not.toContain('test-event');
    });
  });

  describe('Memory management', () => {
    it('should clean up empty listener arrays', () => {
      const handler = vi.fn();

      eventEmitter.on('test-event', handler);
      expect(eventEmitter.hasListeners('test-event')).toBe(true);

      eventEmitter.off('test-event', handler);
      expect(eventEmitter.hasListeners('test-event')).toBe(false);
      expect(eventEmitter.eventNames()).not.toContain('test-event');
    });

    it('should clean up after once listeners', () => {
      const handler = vi.fn();

      eventEmitter.subscribe('test-event', handler, { once: true });
      expect(eventEmitter.hasListeners('test-event')).toBe(true);

      eventEmitter.emit('test-event');
      expect(eventEmitter.hasListeners('test-event')).toBe(false);
      expect(eventEmitter.eventNames()).not.toContain('test-event');
    });
  });
});

describe('Factory functions', () => {
  it('should create new event emitter instances', () => {
    const emitter1 = createEventEmitter();
    const emitter2 = createEventEmitter();

    expect(emitter1).toBeInstanceOf(EventEmitter);
    expect(emitter2).toBeInstanceOf(EventEmitter);
    expect(emitter1).not.toBe(emitter2);
  });

  it('should provide global event emitter', async () => {
    expect(globalEventEmitter).toBeInstanceOf(EventEmitter);
    
    // Should be singleton
    const { globalEventEmitter: globalEmitter2 } = await import('./events');
    expect(globalEventEmitter).toBe(globalEmitter2);
  });
});

describe('System Events', () => {
  let eventEmitter: EventEmitter;

  beforeEach(() => {
    eventEmitter = new EventEmitter();
  });

  it('should define system event constants', () => {
    expect(SystemEvents.APP_STARTUP).toBe('app:startup');
    expect(SystemEvents.APP_SHUTDOWN).toBe('app:shutdown');
    expect(SystemEvents.PLUGIN_LOADED).toBe('plugin:loaded');
    expect(SystemEvents.PLUGIN_UNLOADED).toBe('plugin:unloaded');
    expect(SystemEvents.PLUGIN_ERROR).toBe('plugin:error');
    expect(SystemEvents.SETTINGS_CHANGED).toBe('settings:changed');
    expect(SystemEvents.THEME_CHANGED).toBe('theme:changed');
  });

  it('should work with system events', () => {
    const handler = vi.fn();
    
    eventEmitter.on(SystemEvents.PLUGIN_LOADED, handler);
    eventEmitter.emit(SystemEvents.PLUGIN_LOADED, {
      pluginName: 'test-plugin',
      version: '1.0.0'
    });

    expect(handler).toHaveBeenCalledWith(expect.objectContaining({
      pluginName: 'test-plugin',
      version: '1.0.0'
    }));
  });
});

describe('Integration scenarios', () => {
  let eventEmitter: EventEmitter;

  beforeEach(() => {
    eventEmitter = new EventEmitter();
  });

  it('should handle complex event flow', () => {
    const results: string[] = [];
    
    // High priority handler that processes data
    eventEmitter.subscribe('data-event', (data) => {
      results.push('high-priority');
      // Simulate processing
      if (data && typeof data === 'object' && data.message) {
        data.processed = true;
      }
    }, { priority: 10 });

    // Medium priority handler that depends on high priority
    eventEmitter.subscribe('data-event', (data) => {
      if (data && data.processed) {
        results.push('medium-priority');
      }
    }, { priority: 5 });

    // Low priority one-time handler
    eventEmitter.subscribe('data-event', () => {
      results.push('low-priority-once');
    }, { priority: 1, once: true });

    // Filtered handler - filter receives original data
    eventEmitter.subscribe('data-event', () => {
      results.push('filtered');
    }, { filter: (data) => data && data.shouldFilter === true });

    // First emission - should trigger high, medium, and low priority handlers
    eventEmitter.emit('data-event', { message: 'test', shouldFilter: false });
    expect(results).toEqual(['high-priority', 'medium-priority', 'low-priority-once']);

    // Second emission - should trigger high, medium, and filtered handlers
    results.length = 0;
    eventEmitter.emit('data-event', { message: 'test2', shouldFilter: true });
    expect(results).toEqual(['high-priority', 'medium-priority', 'filtered']);

    // Third emission - should only trigger high and medium (once handler was removed)
    results.length = 0;
    eventEmitter.emit('data-event', { message: 'test3', shouldFilter: false });
    expect(results).toEqual(['high-priority', 'medium-priority']);
  });

  it('should handle subscription and unsubscription during emission', () => {
    const handler1 = vi.fn();
    const handler2 = vi.fn();
    let subscription: any;

    // Handler that unsubscribes itself
    const selfUnsubscribingHandler = vi.fn(() => {
      subscription.unsubscribe();
    });

    // Handler that adds another handler
    const addingHandler = vi.fn(() => {
      eventEmitter.on('test-event', handler2);
    });

    eventEmitter.on('test-event', handler1);
    subscription = eventEmitter.subscribe('test-event', selfUnsubscribingHandler);
    eventEmitter.on('test-event', addingHandler);

    eventEmitter.emit('test-event');

    expect(handler1).toHaveBeenCalledOnce();
    expect(selfUnsubscribingHandler).toHaveBeenCalledOnce();
    expect(addingHandler).toHaveBeenCalledOnce();
    expect(handler2).not.toHaveBeenCalled(); // Added during emission, shouldn't be called

    // Second emission should call the newly added handler
    eventEmitter.emit('test-event');
    expect(handler2).toHaveBeenCalledOnce();
    expect(selfUnsubscribingHandler).toHaveBeenCalledOnce(); // Still only once
  });
});