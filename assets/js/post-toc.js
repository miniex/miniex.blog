document.addEventListener("DOMContentLoaded", function () {
  var tocLinks = document.querySelectorAll(".toc-link");
  var tocLinksMobile = document.querySelectorAll(".toc-link-mobile");
  var tocDrawerCheckbox = document.getElementById("toc-drawer");
  var headerOffset = 120;

  // Smooth scroll with header offset compensation
  function handleTocClick(e) {
    e.preventDefault();
    var targetId = this.getAttribute("data-target");
    var target = document.getElementById(targetId);
    if (!target) return;

    var top =
      target.getBoundingClientRect().top + window.pageYOffset - headerOffset;
    window.scrollTo({ top: top, behavior: "smooth" });
  }

  tocLinks.forEach(function (link) {
    link.addEventListener("click", handleTocClick);
  });

  // Mobile links: also close drawer after click
  tocLinksMobile.forEach(function (link) {
    link.addEventListener("click", function (e) {
      handleTocClick.call(this, e);
      if (tocDrawerCheckbox) {
        tocDrawerCheckbox.checked = false;
      }
    });
  });

  // Active section highlighting based on scroll position
  if (tocLinks.length === 0) return;

  var headingIds = [];
  tocLinks.forEach(function (link) {
    headingIds.push(link.getAttribute("data-target"));
  });

  var ticking = false;
  window.addEventListener("scroll", function () {
    if (!ticking) {
      window.requestAnimationFrame(function () {
        highlightCurrentSection();
        ticking = false;
      });
      ticking = true;
    }
  });

  function highlightCurrentSection() {
    var scrollPos = window.pageYOffset + headerOffset + 20;
    var currentId = null;

    for (var i = headingIds.length - 1; i >= 0; i--) {
      var el = document.getElementById(headingIds[i]);
      if (el && el.offsetTop <= scrollPos) {
        currentId = headingIds[i];
        break;
      }
    }

    // Update desktop TOC
    tocLinks.forEach(function (link) {
      if (link.getAttribute("data-target") === currentId) {
        link.classList.add("text-primary", "border-primary", "font-medium");
        link.classList.remove("text-base-content/60", "border-transparent");
      } else {
        link.classList.remove("text-primary", "border-primary", "font-medium");
        link.classList.add("text-base-content/60", "border-transparent");
      }
    });

    // Update mobile TOC
    tocLinksMobile.forEach(function (link) {
      if (link.getAttribute("data-target") === currentId) {
        link.classList.add("text-primary", "font-medium");
        link.classList.remove("text-base-content/70");
      } else {
        link.classList.remove("text-primary", "font-medium");
        link.classList.add("text-base-content/70");
      }
    });
  }

  // Initial highlight
  highlightCurrentSection();
});
