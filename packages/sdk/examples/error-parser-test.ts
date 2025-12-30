import { 
  parseHttpError, 
  parseFsError, 
  parseClipboardError, 
  parseDialogError 
} from '../src/utils/error-parser';
import { errorCode, errorUtils } from '../src/types/errors';

/**
 * 错误解析器测试示例
 * 展示改进后的错误解析器如何处理各种错误情况
 */

// 测试HTTP错误解析的优先级
export function testHttpErrorParsing() {
  console.log('=== HTTP 错误解析测试 ===');

  // 测试具体的HTTP错误（应该优先匹配）
  const forbiddenError = parseHttpError('HTTP 403 Forbidden', { url: 'https://api.example.com' });
  console.log('403错误:', forbiddenError.code === errorCode.http.FORBIDDEN ? '✅' : '❌');

  // 测试通用权限错误（应该降级匹配）
  const permissionError = parseHttpError('Permission denied for API access', { url: 'https://api.example.com' });
  console.log('权限错误:', permissionError.code === errorCode.common.PERMISSION_DENIED ? '✅' : '❌');

  // 测试超时错误
  const timeoutError = parseHttpError('Request timed out after 30000ms', { url: 'https://api.example.com', timeout: 30000 });
  console.log('超时错误:', timeoutError.code === errorCode.http.TIMEOUT ? '✅' : '❌');

  // 测试网络错误
  const networkError = parseHttpError('Network error: connection refused', { url: 'https://api.example.com' });
  console.log('网络错误:', networkError.code === errorCode.http.NETWORK_ERROR ? '✅' : '❌');

  // 测试未知错误
  const unknownError = parseHttpError('Some random error message', { url: 'https://api.example.com' });
  console.log('未知错误:', unknownError.code === errorCode.common.UNKNOWN ? '✅' : '❌');
}

// 测试文件系统错误解析的优先级
export function testFsErrorParsing() {
  console.log('\n=== 文件系统错误解析测试 ===');

  // 测试文件未找到
  const fileNotFoundError = parseFsError('No such file or directory', { path: '/path/to/file.txt' });
  console.log('文件未找到:', fileNotFoundError.code === errorCode.fs.FILE_NOT_FOUND ? '✅' : '❌');

  // 测试文件访问被拒绝（应该优先于通用权限错误）
  const fileAccessError = parseFsError('Access denied to file', { path: '/path/to/file.txt' });
  console.log('文件访问被拒绝:', fileAccessError.code === errorCode.fs.FILE_ACCESS_DENIED ? '✅' : '❌');

  // 测试通用权限错误
  const permissionError = parseFsError('Permission denied for filesystem operation', { path: '/path/to/file.txt' });
  console.log('通用权限错误:', permissionError.code === errorCode.common.PERMISSION_DENIED ? '✅' : '❌');

  // 测试磁盘空间不足
  const diskFullError = parseFsError('No space left on device', { path: '/path/to/file.txt' });
  console.log('磁盘空间不足:', diskFullError.code === errorCode.fs.DISK_FULL ? '✅' : '❌');

  // 测试文件已存在
  const fileExistsError = parseFsError('File already exists', { path: '/path/to/file.txt' });
  console.log('文件已存在:', fileExistsError.code === errorCode.fs.FILE_ALREADY_EXISTS ? '✅' : '❌');
}

// 测试剪贴板错误解析的优先级
export function testClipboardErrorParsing() {
  console.log('\n=== 剪贴板错误解析测试 ===');

  // 测试格式不支持
  const formatError = parseClipboardError('Format not supported: image/svg', { format: 'image/svg' });
  console.log('格式不支持:', formatError.code === errorCode.clipboard.FORMAT_UNSUPPORTED ? '✅' : '❌');

  // 测试剪贴板为空
  const emptyError = parseClipboardError('Clipboard is empty', {});
  console.log('剪贴板为空:', emptyError.code === errorCode.clipboard.EMPTY ? '✅' : '❌');

  // 测试剪贴板访问被拒绝（应该优先于通用权限错误）
  const clipboardAccessError = parseClipboardError('Clipboard access denied', {});
  console.log('剪贴板访问被拒绝:', clipboardAccessError.code === errorCode.clipboard.ACCESS_DENIED ? '✅' : '❌');

  // 测试剪贴板不可用
  const unavailableError = parseClipboardError('Clipboard not available', {});
  console.log('剪贴板不可用:', unavailableError.code === errorCode.clipboard.UNAVAILABLE ? '✅' : '❌');

  // 测试通用权限错误
  const permissionError = parseClipboardError('Permission denied for clipboard operation', {});
  console.log('通用权限错误:', permissionError.code === errorCode.common.PERMISSION_DENIED ? '✅' : '❌');
}

// 测试对话框错误解析
export function testDialogErrorParsing() {
  console.log('\n=== 对话框错误解析测试 ===');

  // 测试对话框被取消
  const cancelledError = parseDialogError('Dialog was cancelled by user', {});
  console.log('对话框被取消:', cancelledError.code === errorCode.dialog.CANCELLED ? '✅' : '❌');

  // 测试对话框不可用
  const unavailableError = parseDialogError('Dialog not available in headless mode', {});
  console.log('对话框不可用:', unavailableError.code === errorCode.dialog.UNAVAILABLE ? '✅' : '❌');

  // 测试无效选项
  const invalidOptionsError = parseDialogError('Invalid options provided', { reason: 'missing title' });
  console.log('无效选项:', invalidOptionsError.code === errorCode.dialog.INVALID_OPTIONS ? '✅' : '❌');
}

// 测试类型安全
export function testTypeSafety() {
  console.log('\n=== 类型安全测试 ===');

  // 测试各种输入类型
  const stringError = parseHttpError('Network error');
  const objectError = parseHttpError({ message: 'Network error' });
  const errorObjectError = parseHttpError(new Error('Network error'));
  const nullError = parseHttpError(null);
  const undefinedError = parseHttpError(undefined);

  console.log('字符串错误:', errorUtils.isPluginError(stringError) ? '✅' : '❌');
  console.log('对象错误:', errorUtils.isPluginError(objectError) ? '✅' : '❌');
  console.log('Error对象错误:', errorUtils.isPluginError(errorObjectError) ? '✅' : '❌');
  console.log('null错误:', errorUtils.isPluginError(nullError) ? '✅' : '❌');
  console.log('undefined错误:', errorUtils.isPluginError(undefinedError) ? '✅' : '❌');
}

// 测试上下文信息
export function testContextInformation() {
  console.log('\n=== 上下文信息测试 ===');

  const error = parseHttpError('Network error', {
    url: 'https://api.example.com/users',
    method: 'GET',
    timeout: 5000,
    customData: 'test'
  });

  console.log('错误码:', error.code);
  console.log('错误消息:', error.message);
  console.log('上下文信息:', error.context);
  
  // 验证上下文信息是否正确保存
  const hasCorrectContext = 
    error.context?.url === 'https://api.example.com/users' &&
    error.context?.method === 'GET' &&
    error.context?.timeout === 5000 &&
    error.context?.customData === 'test' &&
    error.context?.errorType === 'http' &&
    error.context?.originalError === 'Network error';

  console.log('上下文信息完整性:', hasCorrectContext ? '✅' : '❌');
}

// 运行所有测试
export function runAllTests() {
  console.log('🧪 开始错误解析器测试...\n');
  
  testHttpErrorParsing();
  testFsErrorParsing();
  testClipboardErrorParsing();
  testDialogErrorParsing();
  testTypeSafety();
  testContextInformation();
  
  console.log('\n✨ 错误解析器测试完成！');
}

// 如果直接运行此文件，执行测试
if (typeof window === 'undefined' && typeof process !== 'undefined') {
  runAllTests();
}