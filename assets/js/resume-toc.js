document.addEventListener("DOMContentLoaded", function () {
  const content = document.getElementById("resume-content");
  const toc = document.getElementById("toc");
  const tocMobile = document.getElementById("toc-mobile");

  if (!content || (!toc && !tocMobile)) {
    return;
  }

  // Find all headings in the content
  const headings = content.querySelectorAll("h1, h2, h3, h4, h5, h6");

  if (headings.length === 0) {
    return;
  }

  // Build hierarchical structure
  const tocStructure = [];
  const stack = [{ level: 0, children: tocStructure }];

  headings.forEach((heading, index) => {
    // Add ID to heading if it doesn't have one
    if (!heading.id) {
      heading.id = `heading-${index}`;
    }

    const level = parseInt(heading.tagName.charAt(1));
    const item = {
      id: heading.id,
      text: heading.textContent,
      level: level,
      heading: heading,
      children: [],
    };

    // Find the correct parent in the stack
    while (stack.length > 1 && stack[stack.length - 1].level >= level) {
      stack.pop();
    }

    // Add to parent's children
    stack[stack.length - 1].children.push(item);

    // Push current item to stack
    stack.push(item);
  });

  // Render TOC recursively
  function renderTocItem(item, isTopLevel = false) {
    const container = document.createElement("div");
    container.className = "toc-item";
    container.dataset.headingId = item.id;

    // Create link
    const link = document.createElement("a");
    link.href = `#${item.id}`;
    link.textContent = item.text;
    link.className = `block py-2.5 pr-3 text-sm rounded-lg transition-all duration-300 ease-out hover:bg-primary/15 hover:text-primary border-l-2 border-transparent`;

    // Style based on level
    if (item.level === 2) {
      link.style.paddingLeft = "0.75rem";
      link.classList.add("font-semibold", "text-base");
    } else if (item.level === 3) {
      link.style.paddingLeft = "1.5rem";
      link.classList.add("font-medium");
    } else if (item.level === 4) {
      link.style.paddingLeft = "2.5rem";
      link.classList.add("text-sm");
    } else if (item.level >= 5) {
      link.style.paddingLeft = "3.5rem";
      link.classList.add("text-xs", "opacity-70");
    }

    // Add toggle icon for items with children (only for h2)
    if (item.children.length > 0 && item.level === 2) {
      const toggleIcon = document.createElement("i");
      toggleIcon.className = "toc-toggle ph ph-caret-right inline-block mr-1.5";
      toggleIcon.style.fontSize = "0.9em";
      toggleIcon.style.transition = "transform 0.3s ease-out";
      toggleIcon.style.transformOrigin = "center";
      link.prepend(toggleIcon);

      // Toggle children on icon click
      toggleIcon.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        const childrenContainer = container.querySelector(".toc-children");
        if (childrenContainer) {
          const isExpanded = childrenContainer.dataset.expanded === "true";
          if (isExpanded) {
            collapseChildren(childrenContainer);
            toggleIcon.className =
              "toc-toggle ph ph-caret-right inline-block mr-1.5";
          } else {
            expandChildren(childrenContainer);
            toggleIcon.className =
              "toc-toggle ph ph-caret-down inline-block mr-1.5";
          }
        }
      });
    }

    container.appendChild(link);

    // Smooth scroll
    link.addEventListener("click", function (e) {
      // If clicking on the toggle icon, don't scroll
      if (e.target.classList.contains("toc-toggle")) {
        return;
      }

      e.preventDefault();

      // Close mobile drawer if open
      const mobileDrawer = document.getElementById("mobile-toc-drawer");
      if (mobileDrawer && mobileDrawer.checked) {
        mobileDrawer.checked = false;
      }

      // Disable scroll event handler during programmatic scroll
      isProgrammaticScroll = true;

      // Calculate offset for fixed header
      const headerOffset = 120;
      const elementPosition = item.heading.getBoundingClientRect().top;
      const offsetPosition =
        elementPosition + window.pageYOffset - headerOffset;

      window.scrollTo({
        top: offsetPosition,
        behavior: "smooth",
      });

      // Re-enable scroll handler after smooth scroll completes
      setTimeout(() => {
        isProgrammaticScroll = false;
      }, 2000);

      // Update URL without triggering scroll
      history.pushState(null, null, `#${item.id}`);

      // Expand the clicked item and its children immediately (only for h2)
      if (item.level === 2) {
        const childrenContainer = container.querySelector(".toc-children");
        if (childrenContainer) {
          expandChildren(childrenContainer);
          const toggleIcon = container.querySelector(".toc-toggle");
          if (toggleIcon) {
            toggleIcon.className =
              "toc-toggle ph ph-caret-down inline-block mr-1.5";
          }
        }
      }

      // Update active state (will also expand ancestors)
      updateActiveItem(item.id);
    });

    // Render children if any
    if (item.children.length > 0) {
      const childrenContainer = document.createElement("div");
      childrenContainer.className = "toc-children";

      // Only h2 items are collapsible, h3+ are always visible
      if (item.level === 2) {
        childrenContainer.style.display = "none"; // Initially collapsed
        childrenContainer.dataset.expanded = "false"; // Track expansion state
      } else {
        childrenContainer.style.display = "block"; // Always visible for h3+
        childrenContainer.dataset.expanded = "true";
      }

      item.children.forEach((child) => {
        childrenContainer.appendChild(renderTocItem(child));
      });

      container.appendChild(childrenContainer);
    }

    return container;
  }

  // Render all top-level items for desktop TOC
  if (toc) {
    tocStructure.forEach((item) => {
      toc.appendChild(renderTocItem(item, true));
    });
  }

  // Render all top-level items for mobile TOC
  if (tocMobile) {
    tocStructure.forEach((item) => {
      tocMobile.appendChild(renderTocItem(item, true));
    });
  }

  // Expand children with animation
  function expandChildren(childrenContainer) {
    if (!childrenContainer || childrenContainer.dataset.expanded === "true")
      return;

    childrenContainer.dataset.expanded = "true";
    childrenContainer.style.display = "block";
    childrenContainer.style.maxHeight = "0px";
    childrenContainer.style.opacity = "0";
    childrenContainer.style.overflow = "hidden";
    childrenContainer.style.transition =
      "max-height 0.3s ease-out, opacity 0.3s ease-out";

    // Trigger reflow
    childrenContainer.offsetHeight;

    // Animate to full height
    childrenContainer.style.maxHeight = childrenContainer.scrollHeight + "px";
    childrenContainer.style.opacity = "1";

    // Remove max-height after animation completes
    setTimeout(() => {
      if (childrenContainer.dataset.expanded === "true") {
        childrenContainer.style.maxHeight = "none";
        childrenContainer.style.overflow = "visible";
      }
    }, 300);
  }

  // Collapse children with animation
  function collapseChildren(childrenContainer) {
    if (!childrenContainer || childrenContainer.dataset.expanded === "false")
      return;

    childrenContainer.dataset.expanded = "false";
    childrenContainer.style.overflow = "hidden";
    childrenContainer.style.maxHeight = childrenContainer.scrollHeight + "px";
    childrenContainer.style.transition =
      "max-height 0.3s ease-out, opacity 0.3s ease-out";

    // Trigger reflow
    childrenContainer.offsetHeight;

    // Animate to 0 height
    childrenContainer.style.maxHeight = "0px";
    childrenContainer.style.opacity = "0";

    setTimeout(() => {
      if (childrenContainer.dataset.expanded === "false") {
        childrenContainer.style.display = "none";
      }
    }, 300);
  }

  // Update active item and expand its ancestors, collapse others
  function updateActiveItem(headingId) {
    // Remove all active states from both desktop and mobile TOC
    document.querySelectorAll("#toc a, #toc-mobile a").forEach((a) => {
      a.classList.remove(
        "bg-primary/20",
        "text-primary",
        "font-bold",
        "border-l-primary",
      );
      a.style.borderLeftColor = "transparent";
      a.style.borderLeftWidth = "2px";
    });

    // Find and highlight active item in both desktop and mobile TOC
    const activeContainers = document.querySelectorAll(
      `[data-heading-id="${headingId}"]`,
    );

    activeContainers.forEach((activeContainer) => {
      const activeLink = activeContainer.querySelector("a");
      if (activeLink) {
        activeLink.classList.add("bg-primary/20", "text-primary", "font-bold");
        activeLink.style.borderLeftColor = "hsl(var(--p))";
        activeLink.style.borderLeftWidth = "3px";
      }
    });

    if (activeContainers.length === 0) return;
    const activeContainer = activeContainers[0];

    // Process both desktop and mobile TOC
    activeContainers.forEach((activeContainer) => {
      // Find the TOC container (either #toc or #toc-mobile)
      let tocContainer = activeContainer.parentElement;
      while (
        tocContainer &&
        tocContainer.id !== "toc" &&
        tocContainer.id !== "toc-mobile"
      ) {
        tocContainer = tocContainer.parentElement;
      }
      if (!tocContainer) return;

      // Find the top-level h2 parent
      let h2Container = activeContainer;
      let parent = activeContainer.parentElement;
      while (parent && parent.id !== tocContainer.id) {
        if (parent.classList.contains("toc-item")) {
          h2Container = parent;
        }
        parent = parent.parentElement;
      }

      // Collapse all h2 children containers that are not in the active h2 section
      tocContainer
        .querySelectorAll(".toc-children")
        .forEach((childrenContainer) => {
          const parentContainer = childrenContainer.parentElement;
          const toggleIcon = parentContainer.querySelector(".toc-toggle");

          // Only collapse h2 sections (those with toggle icons)
          if (toggleIcon) {
            // Check if this is the active h2 section
            if (parentContainer === h2Container) {
              // Expand the active h2 section
              expandChildren(childrenContainer);
              toggleIcon.className =
                "toc-toggle ph ph-caret-down inline-block mr-1.5";
            } else {
              // Collapse other h2 sections
              collapseChildren(childrenContainer);
              toggleIcon.className =
                "toc-toggle ph ph-caret-right inline-block mr-1.5";
            }
          }
        });
    });
  }

  // Find which heading should be expanded based on its children
  function findActiveHeadingWithContext(headings, scrollPos) {
    let currentHeading = null;

    headings.forEach((heading) => {
      if (heading.offsetTop <= scrollPos) {
        currentHeading = heading;
      }
    });

    return currentHeading;
  }

  // Highlight active section on scroll
  let isScrolling = false;
  let isProgrammaticScroll = false;

  window.addEventListener("scroll", function () {
    if (isScrolling || isProgrammaticScroll) return;

    isScrolling = true;
    setTimeout(() => {
      const scrollPos = window.scrollY + 120;

      const currentHeading = findActiveHeadingWithContext(headings, scrollPos);

      if (currentHeading && currentHeading.id) {
        updateActiveItem(currentHeading.id);
      }

      isScrolling = false;
    }, 100);
  });

  // Initialize first item as active and expand its section
  if (tocStructure.length > 0) {
    updateActiveItem(tocStructure[0].id);
  }
});
