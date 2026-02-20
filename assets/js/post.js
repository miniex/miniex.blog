// Scroll progress indicator
window.addEventListener(
  "scroll",
  function () {
    var winScroll =
      document.body.scrollTop || document.documentElement.scrollTop;
    var height =
      document.documentElement.scrollHeight -
      document.documentElement.clientHeight;
    var scrolled = (winScroll / height) * 100;
    document.getElementById("progress-bar").style.width = scrolled + "%";
  },
  { passive: true },
);

// i18n strings from server
var i18nEl = document.getElementById("i18n-data");
var i18n = i18nEl ? JSON.parse(i18nEl.textContent) : {};

// Get postId from DOM
var commentsSection = document.getElementById("comments-section");
var postId = commentsSection ? commentsSection.dataset.postId : null;

if (postId) {
  document.addEventListener("DOMContentLoaded", function () {
    loadComments();
  });

  document
    .getElementById("comment-form")
    .addEventListener("submit", async function (e) {
      e.preventDefault();

      var author = document.getElementById("comment-author").value;
      var content = document.getElementById("comment-content").value;
      var password = document.getElementById("comment-password").value;

      if (!author.trim() || !content.trim()) {
        alert(
          i18n.comments_enter_both || "Please enter both name and comment.",
        );
        return;
      }

      try {
        var response = await fetch("/api/comments", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            post_id: postId,
            author: author,
            content: content,
            password: password || null,
          }),
        });

        if (response.ok) {
          document.getElementById("comment-author").value = "";
          document.getElementById("comment-content").value = "";
          document.getElementById("comment-password").value = "";
          loadComments();
        } else {
          alert(i18n.comments_failed_create || "Failed to create comment.");
        }
      } catch (error) {
        console.error("Error:", error);
        alert(i18n.comments_error || "An error occurred.");
      }
    });

  // Like button functionality
  (function () {
    var cid = localStorage.getItem("blog_client_id");
    if (!cid) {
      cid = crypto.randomUUID();
      localStorage.setItem("blog_client_id", cid);
    }
    var likeBtn = document.getElementById("like-btn");
    var likeIcon = document.getElementById("like-icon");
    var likeCount = document.getElementById("like-count");
    if (!likeBtn) return;

    var likedSlugs = JSON.parse(
      localStorage.getItem("blog_liked_posts") || "{}",
    );
    if (likedSlugs[postId]) {
      likeIcon.className = "ph-fill ph-heart text-lg text-red-400";
      likeBtn.classList.add("text-red-400");
    }

    likeBtn.addEventListener("click", async function () {
      try {
        var resp = await fetch("/api/post/" + postId + "/like", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ client_id: cid }),
        });
        if (!resp.ok && resp.status !== 200) return;
        var text = await resp.text();
        if (!text) return;
        var data = JSON.parse(text);
        likeCount.textContent = data.like_count;
        if (data.liked) {
          likeIcon.className = "ph-fill ph-heart text-lg text-red-400";
          likeBtn.classList.add("text-red-400");
          likedSlugs[postId] = true;
        } else {
          likeIcon.className = "ph ph-heart text-lg";
          likeBtn.classList.remove("text-red-400");
          delete likedSlugs[postId];
        }
        localStorage.setItem("blog_liked_posts", JSON.stringify(likedSlugs));
        likeIcon.classList.add("like-bounce");
        setTimeout(function () {
          likeIcon.classList.remove("like-bounce");
        }, 400);
      } catch (e) {
        console.error("Like error:", e);
      }
    });
  })();
}

async function loadComments() {
  try {
    var response = await fetch("/api/comments/" + postId);
    var data = await response.json();
    var container = document.getElementById("comments-container");

    if (data.data.length === 0) {
      container.textContent = "";
      var p = document.createElement("p");
      p.className = "text-center text-base-content/70 py-8";
      p.textContent = i18n.comments_be_first || "Be the first to comment!";
      container.appendChild(p);
      return;
    }

    container.textContent = "";
    data.data.forEach(function (comment) {
      var card = document.createElement("div");
      card.className =
        "group relative p-6 bg-gradient-to-br from-base-100 to-base-200 rounded-xl border border-base-300/20 shadow-sm hover:shadow-md transition-all duration-300";

      var topBar = document.createElement("div");
      topBar.className =
        "absolute top-0 left-0 w-full h-0.5 bg-gradient-to-r from-primary/60 to-secondary/60 rounded-t-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300";
      card.appendChild(topBar);

      var header = document.createElement("div");
      header.className = "flex justify-between items-start mb-4";

      var userInfo = document.createElement("div");
      userInfo.className = "flex items-center gap-3";

      var avatar = document.createElement("div");
      avatar.className =
        "w-10 h-10 rounded-full bg-gradient-to-br from-primary/20 to-secondary/20 flex items-center justify-center border border-base-300/30";
      var avatarIcon = document.createElement("i");
      avatarIcon.className = "ph ph-user text-primary text-lg";
      avatar.appendChild(avatarIcon);
      userInfo.appendChild(avatar);

      var nameBlock = document.createElement("div");
      var nameEl = document.createElement("h4");
      nameEl.className = "font-semibold text-base-content text-lg";
      nameEl.textContent = comment.author;
      nameBlock.appendChild(nameEl);

      var dateEl = document.createElement("p");
      dateEl.className = "text-sm text-base-content/60 flex items-center gap-1";
      var clockIcon = document.createElement("i");
      clockIcon.className = "ph ph-clock text-xs";
      dateEl.appendChild(clockIcon);
      var dateText = document.createTextNode(
        " " + formatDate(comment.created_at),
      );
      dateEl.appendChild(dateText);
      nameBlock.appendChild(dateEl);
      userInfo.appendChild(nameBlock);
      header.appendChild(userInfo);

      var actions = document.createElement("div");
      actions.className =
        "flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-300";

      if (comment.password_hash) {
        var editBtn = document.createElement("button");
        editBtn.className =
          "btn btn-sm btn-ghost btn-circle edit-comment-btn hover:bg-primary/10 hover:text-primary";
        editBtn.dataset.id = comment.id;
        var editIcon = document.createElement("i");
        editIcon.className = "ph ph-pencil text-base";
        editBtn.appendChild(editIcon);
        actions.appendChild(editBtn);
      }

      var deleteBtn = document.createElement("button");
      deleteBtn.className =
        "btn btn-sm btn-ghost btn-circle delete-comment-btn hover:bg-error/10 hover:text-error";
      deleteBtn.dataset.id = comment.id;
      var deleteIcon = document.createElement("i");
      deleteIcon.className = "ph ph-trash text-base";
      deleteBtn.appendChild(deleteIcon);
      actions.appendChild(deleteBtn);
      header.appendChild(actions);
      card.appendChild(header);

      var body = document.createElement("div");
      body.className = "pl-13";
      var contentP = document.createElement("p");
      contentP.className =
        "text-base-content/90 whitespace-pre-wrap leading-relaxed";
      contentP.textContent = comment.content;
      body.appendChild(contentP);
      card.appendChild(body);

      container.appendChild(card);
    });

    // Add edit event listeners
    document.querySelectorAll(".edit-comment-btn").forEach(function (btn) {
      btn.addEventListener("click", async function () {
        var commentId = this.dataset.id;
        var commentContainer = this.closest(".group");
        var currentContent = commentContainer.querySelector(
          ".whitespace-pre-wrap",
        ).textContent;

        var password = prompt(
          i18n.comments_enter_password_edit ||
            "Enter password to edit comment:",
        );
        if (!password) return;

        var newContent = prompt(
          i18n.comments_edit_prompt || "Edit your comment:",
          currentContent,
        );
        if (!newContent || newContent === currentContent) return;

        try {
          var response = await fetch("/api/comments/edit/" + commentId, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ content: newContent, password: password }),
          });

          if (response.ok) {
            var result = await response.json();
            if (result.success) {
              loadComments();
            } else {
              alert(
                i18n.comments_wrong_password ||
                  "Wrong password or comment not found.",
              );
            }
          } else {
            alert(i18n.comments_failed_edit || "Failed to edit comment.");
          }
        } catch (error) {
          console.error("Error:", error);
          alert(i18n.comments_error || "An error occurred.");
        }
      });
    });

    // Add delete event listeners
    document.querySelectorAll(".delete-comment-btn").forEach(function (btn) {
      btn.addEventListener("click", async function () {
        var commentId = this.dataset.id;

        var password = prompt(
          i18n.comments_enter_password_delete ||
            "Enter password to delete this comment:",
        );
        if (!password) return;

        if (
          confirm(
            i18n.comments_confirm_delete ||
              "Are you sure you want to delete this comment?",
          )
        ) {
          try {
            var response = await fetch("/api/comments/delete/" + commentId, {
              method: "DELETE",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ password: password }),
            });

            if (response.ok) {
              var result = await response.json();
              if (result.success) {
                loadComments();
              } else {
                alert(
                  i18n.comments_wrong_password ||
                    "Wrong password or comment not found.",
                );
              }
            } else {
              alert(i18n.comments_failed_edit || "Failed to delete comment.");
            }
          } catch (error) {
            console.error("Error:", error);
            alert(i18n.comments_error || "An error occurred.");
          }
        }
      });
    });
  } catch (error) {
    console.error("Error loading comments:", error);
    var container = document.getElementById("comments-container");
    container.textContent = "";
    var p = document.createElement("p");
    p.className = "text-center text-error py-8";
    p.textContent = i18n.comments_error || "Failed to load comments.";
    container.appendChild(p);
  }
}

function formatDate(dateString) {
  var d = new Date(dateString);
  var Y = d.getFullYear();
  var M = String(d.getMonth() + 1).padStart(2, "0");
  var D = String(d.getDate()).padStart(2, "0");
  var h = String(d.getHours()).padStart(2, "0");
  var m = String(d.getMinutes()).padStart(2, "0");
  var offset = -d.getTimezoneOffset() / 60;
  var sign = offset >= 0 ? "+" : "";
  return Y + "/" + M + "/" + D + " " + h + ":" + m + " " + sign + offset;
}

// Reveal article content — fade out loading overlay
var contentRevealed = false;
function revealContent() {
  if (contentRevealed) return;
  contentRevealed = true;
  var overlay = document.getElementById("post-loading");
  if (!overlay) return;
  overlay.style.opacity = "0";
  setTimeout(function () {
    overlay.remove();
  }, 500);
}

// Render KaTeX math elements (katex loaded via defer in HTML)
function renderMath() {
  var mathEls = document.querySelectorAll(".math");
  if (!mathEls.length || typeof katex === "undefined") return;
  mathEls.forEach(function (el) {
    var isDisplay = el.classList.contains("math-display");
    katex.render(el.textContent, el, {
      displayMode: isDisplay,
      throwOnError: false,
    });
    el.classList.add("rendered");
  });
}

// Content reveal coordination: waits for KaTeX + graphs to finish
document.addEventListener("DOMContentLoaded", function () {
  var hasGraphs = document.querySelector(
    ".function-plot-target, .chart-js-target, .plotly-target",
  );
  var mathDone = false;
  var graphsDone = false;

  function checkReveal() {
    if (mathDone && (graphsDone || !hasGraphs)) revealContent();
  }

  if (hasGraphs) {
    document.addEventListener(
      "graphs:rendered",
      function () {
        graphsDone = true;
        checkReveal();
      },
      { once: true },
    );
    setTimeout(revealContent, 8000);
  }

  renderMath();
  mathDone = true;
  checkReveal();
});
