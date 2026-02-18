document.addEventListener("DOMContentLoaded", function () {
  var i18nEl = document.getElementById("i18n-data");
  var i18n = i18nEl ? JSON.parse(i18nEl.textContent) : {};

  var searchInput = document.getElementById("search-input");
  var searchResults = document.getElementById("search-results");
  var searchLangBadge = document.getElementById("search-lang-badge");
  var searchModal = document.getElementById("search-modal");

  if (!searchInput || !searchResults || !searchLangBadge) return;

  var langs = ["ko", "ja", "en"];
  var currentLang = searchLangBadge.dataset.lang || "en";
  var debounceTimer = null;
  var selectedIndex = -1;

  // Keyboard shortcut: Ctrl+K or /
  document.addEventListener("keydown", function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      searchModal.showModal();
      searchInput.focus();
    }
    if (
      e.key === "/" &&
      document.activeElement.tagName !== "INPUT" &&
      document.activeElement.tagName !== "TEXTAREA"
    ) {
      e.preventDefault();
      searchModal.showModal();
      searchInput.focus();
    }
  });

  // Arrow key navigation inside modal
  searchInput.addEventListener("keydown", function (e) {
    var items = searchResults.querySelectorAll("[data-search-item]");
    if (!items.length) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
      updateSelection(items);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      updateSelection(items);
    } else if (
      e.key === "Enter" &&
      selectedIndex >= 0 &&
      items[selectedIndex]
    ) {
      e.preventDefault();
      items[selectedIndex].click();
    }
  });

  function updateSelection(items) {
    items.forEach(function (item, i) {
      if (i === selectedIndex) {
        item.classList.add("bg-primary/10");
        item.classList.remove("hover:bg-base-200/80");
        item.scrollIntoView({ block: "nearest" });
      } else {
        item.classList.remove("bg-primary/10");
        item.classList.add("hover:bg-base-200/80");
      }
    });
  }

  // Reset on modal close
  searchModal.addEventListener("close", function () {
    searchInput.value = "";
    searchResults.classList.add("hidden");
    searchResults.textContent = "";
    selectedIndex = -1;
  });

  // Language badge click: cycle ko -> ja -> en
  searchLangBadge.addEventListener("click", function (e) {
    e.preventDefault();
    e.stopPropagation();
    var idx = langs.indexOf(currentLang);
    currentLang = langs[(idx + 1) % langs.length];
    searchLangBadge.textContent = currentLang.toUpperCase();
    searchLangBadge.dataset.lang = currentLang;

    if (searchInput.value.trim().length > 0) {
      performSearch(searchInput.value.trim());
    }
    searchInput.focus();
  });

  // Debounced search on input
  searchInput.addEventListener("input", function () {
    clearTimeout(debounceTimer);
    selectedIndex = -1;
    var query = searchInput.value.trim();

    if (query.length === 0) {
      searchResults.classList.add("hidden");
      searchResults.textContent = "";
      return;
    }

    debounceTimer = setTimeout(function () {
      performSearch(query);
    }, 300);
  });

  function performSearch(query) {
    fetch("/api/search?q=" + encodeURIComponent(query) + "&lang=" + currentLang)
      .then(function (res) {
        return res.json();
      })
      .then(function (results) {
        searchResults.classList.remove("hidden");
        searchResults.textContent = "";
        selectedIndex = -1;

        if (results.length === 0) {
          var emptyState = document.createElement("div");
          emptyState.className = "flex flex-col items-center py-10 px-4";

          var icon = document.createElement("i");
          icon.className =
            "ph ph-magnifying-glass text-4xl text-base-content/20 mb-3";
          emptyState.appendChild(icon);

          var msg = document.createElement("p");
          msg.className = "text-sm text-base-content/45 text-center";
          msg.textContent = i18n.search_no_results || "No results found.";
          emptyState.appendChild(msg);

          searchResults.appendChild(emptyState);
          return;
        }

        var list = document.createElement("div");
        list.className = "py-2";

        results.forEach(function (item, index) {
          var link = document.createElement("a");
          link.href = "/post/" + encodeURIComponent(item.slug);
          link.className =
            "flex items-start gap-3 px-4 py-3 hover:bg-base-200/80 transition-all duration-150 cursor-pointer";
          link.setAttribute("data-search-item", index);

          // Left icon
          var iconWrap = document.createElement("div");
          iconWrap.className =
            "flex-shrink-0 w-8 h-8 rounded-lg bg-gradient-to-br from-primary/10 to-secondary/10 flex items-center justify-center mt-0.5";
          var iconEl = document.createElement("i");
          var iconClass =
            item.post_type === "Blog"
              ? "ph-article"
              : item.post_type === "Review"
                ? "ph-star"
                : "ph-notebook";
          iconEl.className = "ph " + iconClass + " text-sm text-primary/70";
          iconWrap.appendChild(iconEl);
          link.appendChild(iconWrap);

          // Content
          var content = document.createElement("div");
          content.className = "flex-1 min-w-0";

          var titleRow = document.createElement("div");
          titleRow.className = "flex items-center gap-2 mb-0.5";

          var title = document.createElement("span");
          title.className = "font-medium text-sm text-base-content truncate";
          title.textContent = item.title;
          titleRow.appendChild(title);

          content.appendChild(titleRow);

          var desc = document.createElement("p");
          desc.className = "text-xs text-base-content/50 line-clamp-1";
          desc.textContent = item.description;
          content.appendChild(desc);

          // Tags row
          if (item.tags && item.tags.length > 0) {
            var tagsRow = document.createElement("div");
            tagsRow.className = "flex items-center gap-1.5 mt-1.5";

            var maxTags = Math.min(item.tags.length, 3);
            for (var t = 0; t < maxTags; t++) {
              var tagEl = document.createElement("span");
              tagEl.className =
                "text-[10px] px-1.5 py-0.5 rounded-md bg-base-200/80 text-base-content/50";
              tagEl.textContent = item.tags[t];
              tagsRow.appendChild(tagEl);
            }
            if (item.tags.length > 3) {
              var moreTag = document.createElement("span");
              moreTag.className = "text-[10px] text-base-content/40";
              moreTag.textContent = "+" + (item.tags.length - 3);
              tagsRow.appendChild(moreTag);
            }

            content.appendChild(tagsRow);
          }

          link.appendChild(content);

          // Right arrow
          var arrow = document.createElement("div");
          arrow.className =
            "flex-shrink-0 opacity-0 group-hover:opacity-100 self-center";
          var arrowIcon = document.createElement("i");
          arrowIcon.className =
            "ph ph-arrow-right text-sm text-base-content/30";
          arrow.appendChild(arrowIcon);
          link.appendChild(arrow);

          list.appendChild(link);
        });

        searchResults.appendChild(list);
      })
      .catch(function (err) {
        console.error("Search error:", err);
      });
  }
});
