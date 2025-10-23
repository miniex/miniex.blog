document.addEventListener("DOMContentLoaded", function () {
  const content = document.getElementById("resume-content");
  const toc = document.getElementById("toc");

  if (!content || !toc) {
    return;
  }

  // Find all headings in the content
  const headings = content.querySelectorAll("h1, h2, h3, h4, h5, h6");

  if (headings.length === 0) {
    return;
  }

  // Generate TOC
  headings.forEach((heading, index) => {
    // Add ID to heading if it doesn't have one
    if (!heading.id) {
      heading.id = `heading-${index}`;
    }

    // Create TOC item
    const level = parseInt(heading.tagName.charAt(1));
    const link = document.createElement("a");
    link.href = `#${heading.id}`;
    link.textContent = heading.textContent;
    link.className = `block py-2 pr-3 text-sm rounded-lg transition-colors duration-200 hover:bg-primary/10 hover:text-primary`;

    // Indent based on heading level with inline styles (h2 starts at 0)
    if (level === 2) {
      link.style.paddingLeft = "0rem";
      link.classList.add("font-semibold");
    } else if (level === 3) {
      link.style.paddingLeft = "1rem";
    } else if (level === 4) {
      link.style.paddingLeft = "2rem";
      link.classList.add("text-sm");
    } else if (level >= 5) {
      link.style.paddingLeft = "3rem";
      link.classList.add("text-xs", "opacity-70");
    }

    toc.appendChild(link);

    // Smooth scroll
    link.addEventListener("click", function (e) {
      e.preventDefault();
      heading.scrollIntoView({ behavior: "smooth", block: "start" });

      // Update URL without triggering scroll
      history.pushState(null, null, `#${heading.id}`);

      // Highlight active link
      document
        .querySelectorAll("#toc a")
        .forEach((a) =>
          a.classList.remove("bg-primary/10", "text-primary", "font-semibold"),
        );
      link.classList.add("bg-primary/10", "text-primary", "font-semibold");
    });
  });

  // Highlight active section on scroll
  let isScrolling = false;
  window.addEventListener("scroll", function () {
    if (isScrolling) return;

    isScrolling = true;
    setTimeout(() => {
      const scrollPos = window.scrollY + 100;

      let currentHeading = null;
      headings.forEach((heading) => {
        if (heading.offsetTop <= scrollPos) {
          currentHeading = heading;
        }
      });

      if (currentHeading) {
        document.querySelectorAll("#toc a").forEach((a) => {
          a.classList.remove("bg-primary/10", "text-primary", "font-semibold");
          if (a.getAttribute("href") === `#${currentHeading.id}`) {
            a.classList.add("bg-primary/10", "text-primary", "font-semibold");
          }
        });
      }

      isScrolling = false;
    }, 100);
  });

  // Highlight first item initially
  const firstLink = toc.querySelector("a");
  if (firstLink) {
    firstLink.classList.add("bg-primary/10", "text-primary", "font-semibold");
  }
});
