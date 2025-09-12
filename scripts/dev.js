const { spawn } = require('child_process');
const path = require('path');

console.log('🚀 Starting BackTestr AI development environment...');

// Platform-specific script
const isWindows = process.platform === 'win32';
const scriptExt = isWindows ? '.bat' : '.sh';
const scriptPath = path.join(__dirname, `dev${scriptExt}`);

const devProcess = spawn(isWindows ? 'cmd' : 'bash', 
  isWindows ? ['/c', scriptPath] : [scriptPath], 
  { stdio: 'inherit', shell: true }
);

devProcess.on('error', (error) => {
  console.error('Failed to start development script:', error);
  process.exit(1);
});

devProcess.on('exit', (code) => {
  process.exit(code);
});