// Header scroll shadow is in inline FOUC script (_base.html <head>)

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
