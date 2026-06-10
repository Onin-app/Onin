<script lang="ts">
  import { getContext } from 'svelte';
  import { http } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>HTTP 请求</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.get', () => http.get('https://httpbin.org/get'))}
    >
      <span class="api-name">get</span>
      <span class="api-desc">GET 请求</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.post', () =>
          http.post('https://httpbin.org/post', {
            test: 'data',
            time: Date.now(),
          }),
        )}
    >
      <span class="api-name">post</span>
      <span class="api-desc">POST 请求</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.put', () =>
          http.put('https://httpbin.org/put', { updated: true }),
        )}
    >
      <span class="api-name">put</span>
      <span class="api-desc">PUT 请求</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.patch', () =>
          http.patch('https://httpbin.org/patch', { patched: true }),
        )}
    >
      <span class="api-name">patch</span>
      <span class="api-desc">PATCH 请求</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.delete', () =>
          http.delete('https://httpbin.org/delete'),
        )}
    >
      <span class="api-name">delete</span>
      <span class="api-desc">DELETE 请求</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('http.request', () =>
          http.request({
            url: 'https://httpbin.org/anything',
            method: 'POST',
            headers: { 'X-Custom': 'test' },
            body: { custom: true },
          }),
        )}
    >
      <span class="api-name">request</span>
      <span class="api-desc">通用请求</span>
    </button>
  </div>
</div>
