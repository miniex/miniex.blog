var i18nEl = document.getElementById("i18n-data");
var i18n = i18nEl ? JSON.parse(i18nEl.textContent) : {};

document.addEventListener("DOMContentLoaded", function () {
  // Guestbook form submission
  document
    .getElementById("guestbook-form")
    .addEventListener("submit", async function (e) {
      e.preventDefault();

      var author = document.getElementById("author").value;
      var content = document.getElementById("content").value;
      var password = document.getElementById("password").value;

      if (!author.trim() || !content.trim()) {
        alert(
          i18n.guestbook_enter_both || "Please enter both name and message.",
        );
        return;
      }

      try {
        var response = await fetch("/api/guestbook", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            author: author,
            content: content,
            password: password || null,
          }),
        });

        if (response.ok) {
          document.getElementById("author").value = "";
          document.getElementById("content").value = "";
          document.getElementById("password").value = "";
          location.reload();
        } else {
          alert(
            i18n.guestbook_failed || "Failed to post entry. Please try again.",
          );
        }
      } catch (error) {
        console.error("Error:", error);
        alert(i18n.comments_error || "An error occurred. Please try again.");
      }
    });

  // Edit entry functionality
  document.querySelectorAll(".edit-btn").forEach(function (btn) {
    btn.addEventListener("click", async function () {
      var entryId = this.dataset.id;
      var entryContainer = this.closest(".group");
      var currentContent = entryContainer.querySelector(
        ".whitespace-pre-wrap",
      ).textContent;

      var password = prompt(
        i18n.comments_enter_password_edit || "Enter password to edit entry:",
      );
      if (!password) return;

      var newContent = prompt(
        i18n.comments_edit_prompt || "Edit your entry:",
        currentContent,
      );
      if (!newContent || newContent === currentContent) return;

      try {
        var response = await fetch("/api/guestbook/edit/" + entryId, {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            content: newContent,
            password: password,
          }),
        });

        if (response.ok) {
          var result = await response.json();
          if (result.success) {
            location.reload();
          } else {
            alert(
              i18n.comments_wrong_password ||
                "Wrong password or entry not found.",
            );
          }
        } else {
          alert(i18n.comments_failed_edit || "Failed to edit entry.");
        }
      } catch (error) {
        console.error("Error:", error);
        alert(i18n.comments_error || "An error occurred.");
      }
    });
  });

  // Delete entry functionality
  document.querySelectorAll(".delete-btn").forEach(function (btn) {
    btn.addEventListener("click", async function () {
      var entryId = this.dataset.id;

      var password = prompt(
        i18n.comments_enter_password_delete ||
          "Enter password to delete this entry:",
      );
      if (!password) return;

      if (
        confirm(
          i18n.comments_confirm_delete ||
            "Are you sure you want to delete this entry?",
        )
      ) {
        try {
          var response = await fetch("/api/guestbook/delete/" + entryId, {
            method: "DELETE",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({ password: password }),
          });

          if (response.ok) {
            var result = await response.json();
            if (result.success) {
              location.reload();
            } else {
              alert(
                i18n.comments_wrong_password ||
                  "Wrong password or entry not found.",
              );
            }
          } else {
            alert(i18n.comments_failed_edit || "Failed to delete entry.");
          }
        } catch (error) {
          console.error("Error:", error);
          alert(i18n.comments_error || "An error occurred.");
        }
      }
    });
  });
});
