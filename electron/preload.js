const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  getVersion: () => ipcRenderer.invoke('get-version'),
  getPlatform: () => ipcRenderer.invoke('get-platform'),
  
  // Message passing to Rust engine
  sendToEngine: (channel, data) => {
    const validChannels = ['tick-data', 'algorithm-execute', 'backtest-start', 'backtest-stop'];
    if (validChannels.includes(channel)) {
      ipcRenderer.send(channel, data);
    }
  },
  
  onEngineMessage: (channel, callback) => {
    const validChannels = ['tick-update', 'bar-update', 'position-update', 'backtest-result'];
    if (validChannels.includes(channel)) {
      ipcRenderer.on(channel, callback);
    }
  },
  
  removeEngineListener: (channel, callback) => {
    ipcRenderer.removeListener(channel, callback);
  }
});