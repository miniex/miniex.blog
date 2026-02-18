document.addEventListener("DOMContentLoaded", function () {
  // Initialize highlight.js
  hljs.highlightAll();

  // Read i18n strings
  var i18nEl = document.getElementById("i18n-data");
  var i18n = i18nEl ? JSON.parse(i18nEl.textContent) : {};
  var copyText = i18n.code_copy || "Copy";
  var copiedText = i18n.code_copied || "Copied!";

  // Add copy buttons to all pre code blocks
  document.querySelectorAll("pre code").forEach(function (codeBlock) {
    var pre = codeBlock.parentElement;
    var button = document.createElement("button");
    button.className = "code-copy-button";
    button.textContent = copyText;

    button.addEventListener("click", function () {
      var code = codeBlock.innerText;
      navigator.clipboard.writeText(code).then(function () {
        button.textContent = copiedText;
        button.classList.add("copied");

        setTimeout(function () {
          button.textContent = copyText;
          button.classList.remove("copied");
        }, 2000);
      });
    });

    pre.appendChild(button);
  });
});
