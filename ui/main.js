const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let isRecording = false;
let historyVisible = false;
let waveformInterval = null;

const $ = (sel) => document.querySelector(sel);

function setState(state) {
  const pill = $("#pill");
  const states = ["idle", "recording", "processing", "done"];

  states.forEach((s) => {
    const el = $(`#state-${s}`);
    if (el) el.classList.toggle("hidden", s !== state);
  });

  pill.className = "";
  if (state !== "idle") {
    pill.classList.add(state);
  }
}

function startWaveform() {
  const canvas = $("#waveform");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const w = canvas.width;
  const h = canvas.height;
  const bars = 16;
  const barW = 3;
  const totalWidth = bars * barW + (bars - 1) * 2;
  const offsetX = (w - totalWidth) / 2;

  waveformInterval = setInterval(() => {
    ctx.clearRect(0, 0, w, h);
    for (let i = 0; i < bars; i++) {
      const amplitude = 0.2 + Math.random() * 0.8;
      const barH = amplitude * h * 0.75;
      const x = offsetX + i * (barW + 2);
      const y = (h - barH) / 2;
      ctx.fillStyle = "rgba(255, 255, 255, 0.7)";
      ctx.beginPath();
      ctx.roundRect(x, y, barW, barH, 1.5);
      ctx.fill();
    }
  }, 70);
}

function stopWaveform() {
  if (waveformInterval) {
    clearInterval(waveformInterval);
    waveformInterval = null;
  }
}

async function startRecording() {
  if (isRecording) return;
  try {
    await invoke("start_recording");
    isRecording = true;
    setState("recording");
    startWaveform();
  } catch (err) {
    console.error("Start error:", err);
  }
}

async function stopRecording() {
  if (!isRecording) return;
  isRecording = false;
  stopWaveform();
  setState("processing");
  try {
    await invoke("stop_recording");
    setState("done");
    setTimeout(() => setState("idle"), 1200);
  } catch (err) {
    console.error("Stop error:", err);
    setState("idle");
  }
}

async function cancelRecording() {
  if (!isRecording) return;
  isRecording = false;
  stopWaveform();
  try {
    await invoke("cancel_recording");
  } catch (err) {
    console.error("Cancel error:", err);
  }
  setState("idle");
}

function formatTime(isoString) {
  const d = new Date(isoString);
  const now = new Date();
  const diff = now - d;
  if (diff < 60000) return "Just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

async function renderHistory() {
  const list = $("#history-list");
  const empty = $("#history-empty");

  try {
    const items = await invoke("get_history");
    if (items.length === 0) {
      list.innerHTML = "";
      empty.classList.remove("hidden");
      return;
    }

    empty.classList.add("hidden");
    list.innerHTML = items
      .map(
        (item) => `
      <div class="history-item" data-id="${item.id}">
        <div class="history-item-content" title="${item.text.replace(/"/g, "&quot;")}">
          <div class="history-text">${escapeHtml(item.text)}</div>
          <div class="history-time">${formatTime(item.created_at)}</div>
        </div>
        <button class="history-delete" data-delete-id="${item.id}" title="Delete">&times;</button>
      </div>`
      )
      .join("");

    list.querySelectorAll(".history-item-content").forEach((el) => {
      el.addEventListener("click", async () => {
        const id = el.parentElement.dataset.id;
        try {
          await invoke("copy_history_item", { id });
          el.querySelector(".history-text").textContent = "Copied!";
          setTimeout(() => renderHistory(), 800);
        } catch (err) {
          console.error("Copy error:", err);
        }
      });
    });

    list.querySelectorAll(".history-delete").forEach((btn) => {
      btn.addEventListener("click", async (e) => {
        e.stopPropagation();
        try {
          await invoke("delete_history_item", { id: btn.dataset.deleteId });
          renderHistory();
        } catch (err) {
          console.error("Delete error:", err);
        }
      });
    });
  } catch (err) {
    console.error("History error:", err);
  }
}

function toggleHistory() {
  historyVisible = !historyVisible;
  const panel = $("#history-panel");
  if (historyVisible) {
    panel.classList.remove("hidden");
    renderHistory();
  } else {
    panel.classList.add("hidden");
  }
}

// ── Event listeners ──

// Idle: click dot area or hover to reveal, then click history/settings
$("#btn-history").addEventListener("click", (e) => {
  e.stopPropagation();
  toggleHistory();
});

$("#history-close").addEventListener("click", () => {
  if (historyVisible) toggleHistory();
});

$("#btn-settings").addEventListener("click", async (e) => {
  e.stopPropagation();
  try {
    await invoke("open_settings");
  } catch (err) {
    console.error("Settings error:", err);
  }
});

// Recording: cancel and stop buttons
$("#btn-cancel").addEventListener("click", (e) => {
  e.stopPropagation();
  cancelRecording();
});

$("#btn-stop").addEventListener("click", (e) => {
  e.stopPropagation();
  stopRecording();
});

// Keyboard: Escape to cancel while recording
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && isRecording) {
    cancelRecording();
  }
});

// Backend events (triggered by global hotkey)
listen("start-recording", () => startRecording());
listen("stop-recording", () => stopRecording());
listen("cancel-recording", () => cancelRecording());
listen("toggle-recording", () => {
  if (isRecording) {
    stopRecording();
  } else {
    startRecording();
  }
});

setState("idle");
