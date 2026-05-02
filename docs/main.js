// Smooth reveal on scroll
const observer = new IntersectionObserver(
  (entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add("visible");
        observer.unobserve(entry.target);
      }
    });
  },
  { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
);

document.querySelectorAll(
  ".feature-card, .step, .download-card, .section-header"
).forEach((el) => {
  el.style.opacity = "0";
  el.style.transform = "translateY(20px)";
  el.style.transition = "opacity 0.5s ease, transform 0.5s ease";
  observer.observe(el);
});

const style = document.createElement("style");
style.textContent = `.visible { opacity: 1 !important; transform: translateY(0) !important; }`;
document.head.appendChild(style);

document.querySelectorAll(".feature-card").forEach((card, i) => {
  card.style.transitionDelay = `${i * 0.08}s`;
});

// Nav background on scroll
const nav = document.querySelector("nav");
window.addEventListener("scroll", () => {
  nav.style.background = window.scrollY > 60
    ? "rgba(10, 10, 12, 0.92)"
    : "rgba(10, 10, 12, 0.7)";
}, { passive: true });

// Fetch latest release and update download link
fetch("https://api.github.com/repos/dzearing/zecho/releases/latest")
  .then(r => r.json())
  .then(release => {
    const dmg = release.assets && release.assets.find(a => a.name.endsWith(".dmg"));
    if (dmg) {
      const link = document.getElementById("download-link");
      if (link) link.href = dmg.browser_download_url;
    }
    const note = document.getElementById("download-note");
    if (note && release.tag_name) {
      note.insertAdjacentText("afterbegin", release.tag_name + " · ");
    }
  })
  .catch(() => {});

// Platform detection — highlight the user's OS download card
const platform = navigator.platform.toLowerCase();
const isMac = platform.includes("mac");
const isWin = platform.includes("win");
const cards = document.querySelectorAll(".download-card");
cards.forEach((card) => {
  const label = card.querySelector(".download-info strong");
  if (!label) return;
  const os = label.textContent.trim().toLowerCase();
  if ((isMac && os === "macos") || (isWin && os === "windows")) {
    card.style.borderColor = "rgba(108, 92, 231, 0.4)";
    card.style.background = "rgba(108, 92, 231, 0.06)";
  }
});
