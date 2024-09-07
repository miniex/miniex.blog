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
        extend: {
            typography: {
                DEFAULT: {
                    css: {
                        "img": {
                            maxWidth: "50%",
                            marginLeft: "auto",
                            marginRight: "auto",
                            borderRadius: "25px",
                        },
                    },
                },
            },
        },
    },
    plugins: [typography, daisyui],
    daisyui: {
        themes: ["cupcake"]
    }
}
