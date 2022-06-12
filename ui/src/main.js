/**
 * This is the minimal code that runs in Electron's main thread.
 *
 * It opens a browser window. In addition to the UI code, the native Rust code
 * runs in the renderer thread, not the main thread.
 */

/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const { app, BrowserWindow, ipcMain } = require("electron");
const path = require("path");

const createWindow = () => {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      contextIsolation: false,
      nodeIntegration: true,
      preload: path.join(__dirname, "preload.js"),
    },
  });

  if (app.isPackaged) {
    win.loadFile("dist/index.html");
  } else {
    win.loadURL("http://localhost:1234");
    win.webContents.openDevTools({ mode: "bottom" });
  }
};

ipcMain.on("ondragstart", (event, filePath) => {
  event.sender.startDrag({
    file: filePath,
    icon: path.join(__dirname, "drag_and_drop.png"),
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.whenReady().then(() => {
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});
