import typography from "@tailwindcss/typography";
import daisyui from "daisyui";

/** @type {import("tailwindcss").Config} */
export default {
    content: [
        "./assets/**/*.css",
        "!./assets/styles/tailwind.output.css",
        "./templates/**/*.html"
    ],
    theme: {
        extend: {},
    },
    plugins: [typography, daisyui],
}
