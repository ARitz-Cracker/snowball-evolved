export * from "./forms.js";
export * from "./css.js";
export * from "./elements.js";
export * from "./workarounds/index.js";

/**
 * Shorthand for `document.querySelector`
 */
export const q = /*#__PURE__*/ document.querySelector.bind(document);

/**
 * Shorthand for `document.querySelectorAll`
 */
export const qa = /*#__PURE__*/ document.querySelectorAll.bind(document);

/**
 * Shorthand for `document.addEventListener`
 */
export const on = /*#__PURE__*/ document.addEventListener.bind(document);

/**
 * Shorthand for `document.removeEventListener`
 */
export const off = /*#__PURE__*/ document.removeEventListener.bind(document);
