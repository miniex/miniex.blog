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
            img: {
              maxWidth: "50%",
              marginLeft: "auto",
              marginRight: "auto",
              borderRadius: "25px",
            },
            blockquote: {
              fontStyle: "normal",
              quotes: "none",
            },
            "blockquote p:first-of-type::before": {
              content: "none",
            },
            "blockquote p:last-of-type::after": {
              content: "none",
            },
          },
        },
      },
    },
  },
  plugins: [typography, daisyui],
  daisyui: {
    themes: ["cupcake", "dark"],
  },
};
