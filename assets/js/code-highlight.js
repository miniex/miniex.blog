document.addEventListener("DOMContentLoaded", function () {
    // Initialize highlight.js
    hljs.highlightAll();

    // Add copy buttons to all pre code blocks
    document.querySelectorAll('pre code').forEach(function(codeBlock) {
        const pre = codeBlock.parentElement;
        const button = document.createElement('button');
        button.className = 'code-copy-button';
        button.textContent = 'Copy';

        button.addEventListener('click', function() {
            const code = codeBlock.innerText;
            navigator.clipboard.writeText(code).then(function() {
                button.textContent = 'Copied!';
                button.classList.add('copied');

                setTimeout(function() {
                    button.textContent = 'Copy';
                    button.classList.remove('copied');
                }, 2000);
            });
        });

        pre.appendChild(button);
    });
});
