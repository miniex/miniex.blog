// Header scroll shadow is in inline FOUC script (_base.html <head>)

// Global toast notification
window.showToast = function (msg, type) {
  type = type || "info";
  var icons = {
    info: "ph-info",
    success: "ph-check-circle",
    error: "ph-warning-circle",
    warn: "ph-warning",
  };
  var colors = {
    info: "bg-base-200 border-primary/30 text-base-content",
    success: "bg-success/10 border-success/30 text-success",
    error: "bg-error/10 border-error/30 text-error",
    warn: "bg-warning/10 border-warning/30 text-warning",
  };
  var container = document.getElementById("toast-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "toast-container";
    container.className =
      "fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2 pointer-events-none";
    document.body.appendChild(container);
  }
  var toast = document.createElement("div");
  toast.className =
    "toast-slide-in pointer-events-auto flex items-center gap-2.5 px-5 py-3 rounded-2xl border shadow-lg backdrop-blur-sm text-sm font-medium " +
    (colors[type] || colors.info);
  var icon = document.createElement("i");
  icon.className = "ph " + (icons[type] || icons.info) + " text-lg";
  toast.appendChild(icon);
  var text = document.createElement("span");
  text.textContent = msg;
  toast.appendChild(text);
  container.appendChild(toast);
  setTimeout(function () {
    toast.classList.add("toast-slide-out");
    setTimeout(function () {
      toast.remove();
    }, 400);
  }, 3000);
};

// Highlight active nav link
(function () {
  var path = window.location.pathname;
  document.querySelectorAll(".nav-link").forEach(function (link) {
    var href = link.getAttribute("href");
    if (href && href !== "/" && path.startsWith(href)) {
      link.classList.add("active");
    }
  });
})();

// Theme toggle functionality
document.addEventListener("DOMContentLoaded", function () {
  var themeToggle = document.getElementById("theme-toggle");
  var themeToggleMobile = document.getElementById("theme-toggle-mobile");
  var html = document.documentElement;

  var savedTheme = localStorage.getItem("theme");
  var systemDarkMode = window.matchMedia(
    "(prefers-color-scheme: dark)",
  ).matches;

  if (
    savedTheme === "dark" ||
    savedTheme === "pastel-dark" ||
    (!savedTheme && systemDarkMode)
  ) {
    applyTheme("pastel-dark");
  } else {
    applyTheme("pastel");
  }

  themeToggle.addEventListener("click", function () {
    var currentTheme = html.getAttribute("data-theme");
    applyTheme(currentTheme === "pastel-dark" ? "pastel" : "pastel-dark");
  });

  themeToggleMobile.addEventListener("click", function () {
    var currentTheme = html.getAttribute("data-theme");
    applyTheme(currentTheme === "pastel-dark" ? "pastel" : "pastel-dark");
  });

  function applyTheme(theme) {
    html.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);

    var lightElements = document.querySelectorAll(".theme-light");
    var darkElements = document.querySelectorAll(".theme-dark");

    if (theme === "pastel-dark") {
      lightElements.forEach(function (el) {
        el.classList.add("hidden");
      });
      darkElements.forEach(function (el) {
        el.classList.remove("hidden");
      });
    } else {
      lightElements.forEach(function (el) {
        el.classList.remove("hidden");
      });
      darkElements.forEach(function (el) {
        el.classList.add("hidden");
      });
    }
  }
});
