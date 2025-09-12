const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { spawn } = require('child_process');

let mainWindow;
let rustProcess;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1920,
    height: 1080,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    },
    icon: path.join(__dirname, 'assets', 'icon.ico'),
    title: 'BackTestr AI'
  });

  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:3000');
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, 'renderer', 'dist', 'index.html'));
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

function startRustEngine() {
  const rustBinary = process.env.NODE_ENV === 'development'
    ? path.join(__dirname, '..', 'target', 'debug', 'backtestr_ai.exe')
    : path.join(process.resourcesPath, 'backtestr_ai.exe');

  rustProcess = spawn(rustBinary, [], {
    env: { ...process.env, RUST_LOG: 'debug' }
  });

  rustProcess.stdout.on('data', (data) => {
    console.log(`Rust Engine: ${data}`);
  });

  rustProcess.stderr.on('data', (data) => {
    console.error(`Rust Engine Error: ${data}`);
  });

  rustProcess.on('close', (code) => {
    console.log(`Rust Engine exited with code ${code}`);
  });
}

app.whenReady().then(() => {
  startRustEngine();
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (rustProcess) {
    rustProcess.kill();
  }
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// IPC handlers
ipcMain.handle('get-version', () => {
  return app.getVersion();
});

ipcMain.handle('get-platform', () => {
  return process.platform;
});