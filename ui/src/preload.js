/* eslint-env node, browser */
/* eslint-disable @typescript-eslint/no-var-requires */

const { ipcRenderer } = require("electron");

window.oxygen = require("oxygen-core");
window.startDragOut = (tmpPath) => {
  ipcRenderer.send("ondragstart", tmpPath);
};
