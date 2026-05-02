const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let isRecording = false;
let recordingLocked = false;
let historyVisible = false;
let waveformInterval = null;
let fnHoldTimer = null;
let lastFnDown = 0;

const DOUBLE_TAP_MS = 400;

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

let barLevels = new Array(16).fill(0);

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

  waveformInterval = setInterval(async () => {
    let level = 0;
    try {
      level = await invoke("get_audio_level");
    } catch (_) {}

    // Aggressive normalization — make speech visually prominent
    const normalized = Math.min(1, Math.pow(level * 15, 0.7));

    barLevels.shift();
    barLevels.push(normalized);

    ctx.clearRect(0, 0, w, h);
    for (let i = 0; i < bars; i++) {
      const amp = 0.08 + barLevels[i] * 0.92;
      const barH = amp * h * 0.9;
      const x = offsetX + i * (barW + 2);
      const y = (h - barH) / 2;
      const alpha = 0.5 + barLevels[i] * 0.5;
      // Tint bars slightly purple when active
      const r = Math.round(200 + barLevels[i] * 55);
      const g = Math.round(200 + barLevels[i] * 30);
      const b = 255;
      ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${alpha})`;
      ctx.beginPath();
      ctx.roundRect(x, y, barW, barH, 1.5);
      ctx.fill();
    }
  }, 50);
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
  recordingLocked = false;
  stopWaveform();
  setState("processing");
  try {
    await invoke("stop_recording");
    // UI stays in "processing" — transcription-complete event will trigger "done"
  } catch (err) {
    console.error("Stop error:", err);
    setState("idle");
  }
}

async function cancelRecording() {
  if (!isRecording) return;
  isRecording = false;
  recordingLocked = false;
  stopWaveform();
  try {
    await invoke("cancel_recording");
  } catch (err) {
    console.error("Cancel error:", err);
  }
  setState("idle");
}

// ── FN key: hold-to-record + double-tap-to-lock ──

function handleFnDown() {
  const now = Date.now();

  if (isRecording && recordingLocked) {
    // FN pressed while locked recording — stop it
    stopRecording();
    return;
  }

  if (!isRecording) {
    // Check for double-tap
    if (now - lastFnDown < DOUBLE_TAP_MS) {
      // Double-tap: start and lock
      recordingLocked = true;
      startRecording();
    } else {
      // Single press: start (will stop on release unless locked)
      recordingLocked = false;
      startRecording();
    }
  }

  lastFnDown = now;
}

function handleFnUp() {
  if (isRecording && !recordingLocked) {
    // Release after hold — stop recording
    stopRecording();
  }
}

// ── Utility ──

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
        (item) => {
          const hasTimings = item.transcribe_ms > 0 || item.cleanup_ms > 0;
          const timingHtml = hasTimings
            ? `<span class="history-timing">STT ${(item.transcribe_ms / 1000).toFixed(1)}s + AI ${(item.cleanup_ms / 1000).toFixed(1)}s</span>`
            : "";
          return `
      <div class="history-item" data-id="${item.id}">
        <div class="history-item-content">
          <div class="history-text">${escapeHtml(item.text)}</div>
          <div class="history-meta">
            <span class="history-time">${formatTime(item.created_at)}</span>
            ${timingHtml}
          </div>
        </div>
        <button class="history-info" data-info-id="${item.id}">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="6.5" stroke="currentColor" stroke-width="1.2"/>
            <path d="M8 7v4M8 5.5v.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <div class="history-detail hidden" id="detail-${item.id}">
        <div class="detail-col">
          <div class="detail-label">Original</div>
          <div class="detail-text">${escapeHtml(item.raw_text)}</div>
        </div>
        <div class="detail-col">
          <div class="detail-label">Cleaned</div>
          <div class="detail-text">${escapeHtml(item.text)}</div>
        </div>
        ${hasTimings ? `<div class="detail-timing">STT ${(item.transcribe_ms / 1000).toFixed(1)}s &middot; AI ${(item.cleanup_ms / 1000).toFixed(1)}s &middot; Total ${((item.transcribe_ms + item.cleanup_ms) / 1000).toFixed(1)}s</div>` : ""}
      </div>`;
        }
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

    list.querySelectorAll(".history-info").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const id = btn.dataset.infoId;
        const detail = document.getElementById(`detail-${id}`);
        if (detail) {
          detail.classList.toggle("hidden");
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

$("#btn-history").addEventListener("click", (e) => {
  e.stopPropagation();
  toggleHistory();
});

$("#history-close").addEventListener("click", () => {
  if (historyVisible) toggleHistory();
});

$("#history-clear").addEventListener("click", async () => {
  try {
    await invoke("clear_history");
    renderHistory();
  } catch (err) {
    console.error("Clear error:", err);
  }
});

$("#btn-settings").addEventListener("click", async (e) => {
  e.stopPropagation();
  try {
    await invoke("open_settings");
  } catch (err) {
    console.error("Settings error:", err);
  }
});

$("#btn-cancel").addEventListener("click", (e) => {
  e.stopPropagation();
  cancelRecording();
});

$("#btn-stop").addEventListener("click", (e) => {
  e.stopPropagation();
  stopRecording();
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && isRecording) {
    cancelRecording();
  }
});

// Backend events
listen("fn-key-down", () => handleFnDown());
listen("fn-key-up", () => handleFnUp());
listen("toggle-recording", () => {
  if (isRecording) {
    stopRecording();
  } else {
    startRecording();
  }
});
listen("transcription-complete", () => {
  setState("done");
  setTimeout(() => setState("idle"), 1200);
  if (historyVisible) renderHistory();
});
listen("transcription-error", (event) => {
  console.error("Transcription error:", event.payload);
  setState("idle");
});

// ── Accessibility check ──

async function checkAccessibility() {
  try {
    const hasAccess = await invoke("check_accessibility");
    if (!hasAccess) {
      $("#accessibility-prompt").classList.remove("hidden");
    }
  } catch (err) {
    console.error("Accessibility check error:", err);
  }
}

$("#btn-grant-access").addEventListener("click", async () => {
  try {
    await invoke("open_accessibility_settings");
  } catch (err) {
    console.error(err);
  }
});

$("#btn-dismiss-access").addEventListener("click", () => {
  $("#accessibility-prompt").classList.add("hidden");
});

// ── Drag ──

$("#pill").addEventListener("mousedown", async (e) => {
  if (e.target.closest("button") || e.target.closest("canvas")) return;
  try {
    await invoke("start_drag");
  } catch (_) {}
});

// ── Dynamic window sizing ──
// Keep the window small (just the pill) when idle. Expand when history opens.
// This avoids transparent area mouse issues — the window IS the pill.


setState("idle");
checkAccessibility();
