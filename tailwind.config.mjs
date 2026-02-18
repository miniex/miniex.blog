import typography from "@tailwindcss/typography";
import daisyui from "daisyui";

/** @type {import("tailwindcss").Config} */
export default {
  content: [
    "./assets/**/*.css",
    "!./assets/styles/tailwind.output.css",
    "./templates/**/*.html",
  ],
  theme: {
    extend: {
      typography: {
        DEFAULT: {
          css: {
            // Better line height for readability
            lineHeight: "1.8",

            // Improved heading styles
            h2: {
              fontWeight: "700",
              marginTop: "2.5em",
              marginBottom: "1em",
              paddingBottom: "0.3em",
              borderBottom: "2px solid hsl(var(--bc) / 0.08)",
            },
            h3: {
              fontWeight: "600",
              marginTop: "1.8em",
              marginBottom: "0.8em",
            },
            h4: {
              fontWeight: "600",
              marginTop: "1.5em",
              marginBottom: "0.6em",
              color: "hsl(var(--bc) / 0.9)",
            },

            // Better paragraph spacing
            p: {
              marginTop: "1.2em",
              marginBottom: "1.2em",
            },

            // Enhanced list styling
            ul: {
              listStyleType: "disc",
              paddingLeft: "1.5em",
            },
            "ul > li": {
              paddingLeft: "0.3em",
              marginTop: "0.5em",
              marginBottom: "0.5em",
            },
            "ul > li::marker": {
              color: "hsl(var(--p))",
            },

            // Better link styling
            a: {
              color: "hsl(var(--p))",
              textDecoration: "none",
              fontWeight: "500",
              transition: "all 0.2s ease",
              "&:hover": {
                color: "hsl(var(--pf))",
                textDecoration: "underline",
              },
            },

            // Enhanced code blocks
            code: {
              backgroundColor: "transparent",
              color: "hsl(var(--bc))",
              padding: "0.2em 0.4em",
              borderRadius: "0.25rem",
              fontSize: "0.9em",
              fontWeight: "500",
              fontFamily: '"JetBrains Mono", "Consolas", "Monaco", monospace',
            },
            "code::before": {
              content: "none",
            },
            "code::after": {
              content: "none",
            },

            // Pre code block styling
            pre: {
              backgroundColor: "#f5f0f2",
              padding: "1.5rem",
              borderRadius: "1.5rem",
              marginTop: "1.5rem",
              marginBottom: "1.5rem",
              overflowX: "auto",
            },
            "pre code": {
              backgroundColor: "transparent",
              padding: "0",
              fontSize: "0.875em",
              lineHeight: "1.7",
              color: "#3d353a",
              fontWeight: "500",
              fontFamily: '"JetBrains Mono", "Consolas", "Monaco", monospace',
            },

            // Syntax highlighting for light mode
            "pre code .hljs-keyword, pre code .hljs-selector-tag, pre code .hljs-type":
              {
                color: "#a8707e",
                fontWeight: "600",
              },
            "pre code .hljs-string, pre code .hljs-attr": {
              color: "#8a7090",
              fontWeight: "600",
            },
            "pre code .hljs-function, pre code .hljs-title": {
              color: "#96707e",
              fontWeight: "600",
            },
            "pre code .hljs-number, pre code .hljs-literal": {
              color: "#c08888",
              fontWeight: "600",
            },
            "pre code .hljs-comment": {
              color: "#a89098",
              fontStyle: "italic",
            },

            // Improved blockquote styling
            blockquote: {
              fontStyle: "normal",
              quotes: "none",
              backgroundColor: "hsl(var(--b2) / 0.5)",
              borderLeftColor: "hsl(var(--p))",
              borderLeftWidth: "4px",
              padding: "0.1em 1.25em",
              borderRadius: "0.375rem",
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
              color: "hsl(var(--bc) / 0.9)",
            },
            "blockquote p:first-of-type::before": {
              content: "none",
            },
            "blockquote p:last-of-type::after": {
              content: "none",
            },

            // Strong text styling
            strong: {
              fontWeight: "700",
              color: "hsl(var(--bc))",
            },

            // Horizontal rule styling
            hr: {
              borderColor: "hsl(var(--bc) / 0.1)",
              marginTop: "2.5em",
              marginBottom: "2.5em",
            },

            img: {
              maxWidth: "50%",
              marginLeft: "auto",
              marginRight: "auto",
              borderRadius: "25px",
            },
          },
        },
        sm: {
          css: {
            blockquote: {
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
            },
          },
        },
        base: {
          css: {
            blockquote: {
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
            },
          },
        },
        lg: {
          css: {
            blockquote: {
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
            },
          },
        },
        xl: {
          css: {
            blockquote: {
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
            },
          },
        },
        "2xl": {
          css: {
            blockquote: {
              marginTop: "0.4em",
              marginBottom: "0.4em",
            },
            "blockquote p": {
              marginTop: "0",
              marginBottom: "0",
            },
          },
        },
      },
    },
  },
  plugins: [typography, daisyui],
  daisyui: {
    themes: [
      {
        pastel: {
          primary: "#c9899e",
          "primary-content": "#ffffff",
          secondary: "#b096b8",
          "secondary-content": "#ffffff",
          accent: "#d4a89e",
          "accent-content": "#ffffff",
          neutral: "#3d353a",
          "neutral-content": "#e8e2e5",
          "base-100": "#fcfafb",
          "base-200": "#f5f0f2",
          "base-300": "#ebe4e7",
          "base-content": "#3d353a",
          info: "#b096b8",
          success: "#8db8a0",
          warning: "#d4c098",
          error: "#c98a8a",
        },
      },
      {
        "pastel-dark": {
          primary: "#d0a0b0",
          "primary-content": "#1c181a",
          secondary: "#baa0c0",
          "secondary-content": "#1c181a",
          accent: "#d4b0a8",
          "accent-content": "#1c181a",
          neutral: "#e0d8dc",
          "neutral-content": "#1c181a",
          "base-100": "#1c181a",
          "base-200": "#242022",
          "base-300": "#332d30",
          "base-content": "#e0d8dc",
          info: "#baa0c0",
          success: "#a0c0a8",
          warning: "#d4c8a0",
          error: "#c89090",
        },
      },
    ],
  },
};
