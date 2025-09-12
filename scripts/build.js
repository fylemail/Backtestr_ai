const { spawn } = require('child_process');
const path = require('path');

console.log('🚀 Building BackTestr AI...');

// Platform-specific script
const isWindows = process.platform === 'win32';
const scriptExt = isWindows ? '.bat' : '.sh';
const scriptPath = path.join(__dirname, `build${scriptExt}`);

const buildProcess = spawn(isWindows ? 'cmd' : 'bash', 
  isWindows ? ['/c', scriptPath] : [scriptPath], 
  { stdio: 'inherit', shell: true }
);

buildProcess.on('error', (error) => {
  console.error('Failed to start build script:', error);
  process.exit(1);
});

buildProcess.on('exit', (code) => {
  if (code !== 0) {
    console.error(`Build failed with exit code ${code}`);
  } else {
    console.log('✅ Build completed successfully!');
  }
  process.exit(code);
});