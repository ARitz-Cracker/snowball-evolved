export * from "./forms.js";
export * from "./css.js";
export * from "./elements.js";

/**
 * Shorthand for `document.querySelector`
 */
export const q = document.querySelector.bind(document);

/**
 * Shorthand for `document.querySelectorAll`
 */
export const qa = document.querySelectorAll.bind(document);
