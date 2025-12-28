import { lifecycle, storage, notification, clipboard, http } from 'sdk';
import './style.css';

const output = document.getElementById('output') as HTMLPreElement;
const status = document.getElementById('status') as HTMLDivElement;

function log(message: string) {
  const timestamp = new Date().toLocaleTimeString();
  output.textContent += `[${timestamp}] ${message}\n`;
  console.log(message);
}

// Initialize
status.textContent = 'SDK loaded successfully!';
log('Baize SDK Demo initialized');
log('Available APIs: lifecycle, storage, notification, clipboard, http, fs, dialog, command, settings, scheduler');

// Test Lifecycle API
document.getElementById('test-lifecycle')?.addEventListener('click', async () => {
  log('\n=== Testing Lifecycle API ===');
  try {
    // Register lifecycle callbacks
    lifecycle.onLoad(() => {
      log('✓ onLoad callback registered');
    });
    
    lifecycle.onUnload(() => {
      log('✓ onUnload callback registered');
    });
    
    lifecycle.onWindowShow(() => {
      log('✓ onWindowShow callback registered');
    });
    
    lifecycle.onWindowHide(() => {
      log('✓ onWindowHide callback registered');
    });
    
    log('✓ All lifecycle callbacks registered successfully');
  } catch (error) {
    log(`✗ Error: ${error}`);
  }
});

// Test Storage API
document.getElementById('test-storage')?.addEventListener('click', async () => {
  log('\n=== Testing Storage API ===');
  try {
    const testData = { value: 'Hello from demo!', timestamp: Date.now() };
    
    await storage.setItem('test-key', testData);
    log('✓ storage.setItem() called');
    
    const value = await storage.getItem('test-key');
    log(`✓ storage.getItem() returned: ${JSON.stringify(value)}`);
    
    const keys = await storage.keys();
    log(`✓ storage.keys() returned: ${JSON.stringify(keys)}`);
    
    await storage.removeItem('test-key');
    log('✓ storage.removeItem() called');
    
    log('✓ Storage API test completed');
  } catch (error) {
    log(`✗ Error: ${error}`);
  }
});

// Test Notification API
document.getElementById('test-notification')?.addEventListener('click', async () => {
  log('\n=== Testing Notification API ===');
  try {
    await notification.show({
      title: 'Test Notification',
      body: 'This is a test notification from Baize SDK Demo'
    });
    log('✓ notification.show() called');
    log('✓ Check your system notifications');
  } catch (error) {
    log(`✗ Error: ${error}`);
  }
});

// Test Clipboard API
document.getElementById('test-clipboard')?.addEventListener('click', async () => {
  log('\n=== Testing Clipboard API ===');
  try {
    const testText = 'Hello from Baize SDK Demo!';
    
    await clipboard.writeText(testText);
    log(`✓ clipboard.writeText() called with: "${testText}"`);
    
    const text = await clipboard.readText();
    log(`✓ clipboard.readText() returned: "${text}"`);
    
    const hasText = await clipboard.hasText();
    log(`✓ clipboard.hasText() returned: ${hasText}`);
    
    log('✓ Clipboard API test completed');
  } catch (error) {
    log(`✗ Error: ${error}`);
  }
});

// Test HTTP API
document.getElementById('test-http')?.addEventListener('click', async () => {
  log('\n=== Testing HTTP API ===');
  try {
    log('Sending GET request to httpbin.org...');
    
    const response = await http.get('https://httpbin.org/get?test=demo');
    
    log(`✓ http.get() status: ${response.status}`);
    log(`✓ Response body: ${JSON.stringify(response.body, null, 2)}`);
    log('✓ HTTP API test completed');
  } catch (error) {
    log(`✗ Error: ${error}`);
  }
});

log('\nClick buttons to test SDK APIs');
