/**
 * Shorthand for `document.querySelector`
 */
export const q = /*#__PURE__*/ document.querySelector.bind(/*#__PURE__*/ document);

/**
 * Shorthand for `document.querySelectorAll`
 */
export const qa = /*#__PURE__*/ document.querySelectorAll.bind(/*#__PURE__*/ document);

/**
 * Shorthand for `document.addEventListener`
 */
export const on = /*#__PURE__*/ document.addEventListener.bind(/*#__PURE__*/ document);

/**
 * Shorthand for `document.removeEventListener`
 */
export const off = /*#__PURE__*/ document.removeEventListener.bind(/*#__PURE__*/ document);

/**
 * Just call's preventDefault on the event.
 * Useful for `thing.addEventListener("something", preventDefault);`
 */
function preventDefault(ev: Event) {
	ev.preventDefault();
}
